#!/usr/bin/env python3
"""Publish an existing tested NRO. No build, test suite, or game launch."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import struct
import subprocess
import tempfile
import tomllib
import zipfile

from package import ROOT, PLUGIN_PATH, REQUIRED_FILES, package, validate_nro

REPO = 'Rikiar73574/mhgu-utility'


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def run(*args):
    return subprocess.run(args, cwd=ROOT, check=True, capture_output=True,
                          text=True, timeout=90).stdout.strip()


def verify_package(archive, nro):
    with zipfile.ZipFile(archive) as z:
        names = z.namelist()
        expected = set(REQUIRED_FILES) | {PLUGIN_PATH, 'SHA256SUMS'}
        if len(names) != len(expected) or set(names) != expected:
            raise ValueError('ZIP runtime allowlist mismatch')
        image = z.read(PLUGIN_PATH)
        validate_nro(image)
        if image != nro.read_bytes():
            raise ValueError('Packaged NRO differs from the supplied tested build')
        footprint = (len(image) + struct.unpack_from('<I', image, 0x38)[0] + 65535) & ~65535
        if footprint > 9 * 1024 * 1024 + 65536:
            raise ValueError('NRO exceeds the shared plugin memory budget')
        sums = dict(line.split('  ', 1)[::-1] for line in z.read('SHA256SUMS').decode().splitlines())
        if set(sums) != expected - {'SHA256SUMS'}:
            raise ValueError('ZIP checksum allowlist mismatch')
        for name, digest in sums.items():
            if sha256(z.read(name)) != digest:
                raise ValueError(f'ZIP checksum mismatch: {name}')


def prepare(nro, output):
    archive = output / 'hunters-compendium.zip'
    package(nro, archive)
    verify_package(archive, nro)
    sums = output / 'SHA256SUMS'
    sums.write_text(f'{sha256(archive.read_bytes())}  {archive.name}\n')
    return archive, sums


def check_remote():
    allowed = {f'git@github.com:{REPO}'.lower(), f'https://github.com/{REPO}'.lower()}
    fetch = run('git', 'remote', 'get-url', 'origin').removesuffix('.git').lower()
    if fetch not in allowed:
        raise ValueError('origin does not point to the Utility repository')
    destinations = run('git', 'remote', 'get-url', '--push', '--all', 'origin').splitlines()
    if len(destinations) != 1 or destinations[0].removesuffix('.git').lower() not in allowed:
        raise ValueError('origin push destination must be the Utility repository only')


def publish(tag, head, notes, archive, sums):
    # Fail before pushing if an existing public release or draft uses this tag.
    releases = json.loads(run('gh', 'api', f'repos/{REPO}/releases?per_page=100'))
    if any(r['tag_name'] == tag for r in releases):
        raise ValueError(f'Release {tag} already exists; it will not be overwritten')
    local_tag = run('git', 'tag', '--list', tag)
    if local_tag:
        if run('git', 'rev-parse', f'{tag}^{{commit}}') != head:
            raise ValueError(f'Tag {tag} points to another commit; refusing to move it')
    else:
        run('git', 'tag', tag, head)
    run('git', 'push', '--atomic', 'origin', 'HEAD:refs/heads/main', f'refs/tags/{tag}')
    with tempfile.TemporaryDirectory(prefix='mhgu-release-') as temporary:
        directory = Path(temporary)
        notes_file = directory / 'notes.md'
        notes_file.write_text(notes)
        run('gh', 'release', 'create', tag, str(archive), str(sums), '--repo', REPO,
            '--verify-tag', '--draft', '--title', f'MHGU Utility {tag}', '--notes-file', str(notes_file))
        # Verify the actual uploaded bytes while the release is still a draft.
        run('gh', 'release', 'download', tag, '--repo', REPO, '--dir', str(directory),
            '--pattern', archive.name, '--pattern', sums.name)
        for source in (archive, sums):
            if (directory / source.name).read_bytes() != source.read_bytes():
                raise ValueError(f'Uploaded {source.name} differs; release left as a draft')
    run('gh', 'release', 'edit', tag, '--repo', REPO, '--draft=false', '--latest')
    latest = json.loads(run('gh', 'api', f'repos/{REPO}/releases/latest'))
    if latest['tag_name'] != tag or latest['draft'] or latest['prerelease']:
        raise ValueError('Published release could not be verified as latest; inspect GitHub')
    return latest['html_url']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('tag', nargs='?', help='default: v + mod/Cargo.toml version')
    parser.add_argument('--notes-file', type=Path, help='release notes; required when publishing')
    parser.add_argument('--nro', type=Path, default=ROOT / 'build/hunters_compendium.nro',
                        help='existing tested binary; never rebuilt')
    parser.add_argument('--sha256', help='optional previously recorded hash of the tested NRO')
    parser.add_argument('--dry-run', action='store_true', help='local packaging/checksums only; no GitHub or git writes')
    args = parser.parse_args()
    try:
        version = tomllib.loads((ROOT / 'mod/Cargo.toml').read_text())['package']['version']
        tag = args.tag or f'v{version}'
        if not re.fullmatch(r'v\d+\.\d+\.\d+', tag) or tag != f'v{version}':
            raise ValueError('Tag must match the version already set in mod/Cargo.toml')
        digest = sha256(args.nro.read_bytes())
        if args.sha256 and digest != args.sha256.lower():
            raise ValueError('NRO differs from the recorded tested SHA-256')
        archive, sums = prepare(args.nro, ROOT / 'dist')
        print(f'Packaging/checksums OK. Existing NRO SHA-256: {digest}', flush=True)
        if args.dry_run:
            print(f'Dry run: {tag}, {archive}, {sums}. Nothing pushed or published.')
            return
        if not args.notes_file or not args.notes_file.read_text().strip():
            raise ValueError('Provide nonempty release notes using --notes-file')
        if not shutil.which('gh'):
            raise ValueError('Install GitHub CLI (gh) and run gh auth login once before publishing')
        try:
            run('gh', 'auth', 'status', '--hostname', 'github.com')
        except subprocess.CalledProcessError as error:
            raise ValueError('GitHub sign-in missing or expired; run gh auth login --hostname github.com once') from error
        if run('git', 'branch', '--show-current') != 'main':
            raise ValueError('Publish from main')
        if run('git', 'status', '--porcelain', '--untracked-files=no'):
            raise ValueError('Commit tracked source changes first; the script never commits for you')
        if run('git', 'ls-files', '--others', '--exclude-standard', 'mod', 'compat', 'tools'):
            raise ValueError('Commit untracked build/tool sources before publishing')
        check_remote()
        head = run('git', 'rev-parse', 'HEAD')
        url = publish(tag, head, args.notes_file.read_text(), archive, sums)
        print(f'Published and verified: {url}')
    except (OSError, ValueError, subprocess.SubprocessError, zipfile.BadZipFile) as error:
        parser.exit(1, f'release: {error}\n')


if __name__ == '__main__':
    main()
