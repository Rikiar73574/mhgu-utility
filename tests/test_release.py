"""Small release-tool checks; never contacts GitHub or runs the game/build."""
import hashlib
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
import release
from test_package import nro


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.nro = Path(self.temporary.name) / 'tested.nro'
        self.nro.write_bytes(nro())

    def tearDown(self):
        self.temporary.cleanup()

    def test_package_matches_existing_binary(self):
        with tempfile.TemporaryDirectory() as tmp:
            archive, sums = release.prepare(self.nro, Path(tmp))
            release.verify_package(archive, self.nro)
            self.assertEqual(sums.read_text(), hashlib.sha256(archive.read_bytes()).hexdigest() + '  hunters-compendium.zip\n')

    def test_corrupt_download_never_publishes_draft(self):
        with tempfile.TemporaryDirectory() as tmp:
            archive, sums = release.prepare(self.nro, Path(tmp))
            commands = []
            def run(*args, **kwargs):
                commands.append(args)
                if args[:3] == ('gh', 'api', 'repos/Rikiar73574/mhgu-utility/releases?per_page=100'):
                    return '[]'
                if args[:3] == ('gh', 'release', 'download'):
                    dest = Path(args[args.index('--dir') + 1])
                    (dest / archive.name).write_bytes(b'corrupted upload')
                    (dest / sums.name).write_bytes(sums.read_bytes())
                return ''
            with patch.object(release, 'run', side_effect=run):
                with self.assertRaisesRegex(ValueError, 'differs'):
                    release.publish('v9.9.9', 'f' * 40, 'notes', archive, sums)
            self.assertFalse(any(c[:3] == ('gh', 'release', 'edit') for c in commands))
            self.assertTrue(any(c[:3] == ('gh', 'release', 'create') for c in commands))
            self.assertFalse(any('cargo' in c or 'pytest' in c for c in commands))

    def test_verified_draft_becomes_latest(self):
        with tempfile.TemporaryDirectory() as tmp:
            archive, sums = release.prepare(self.nro, Path(tmp))
            commands = []
            def run(*args):
                commands.append(args)
                if args[:2] == ('gh', 'api'):
                    if args[-1].endswith('/latest'):
                        return '{"tag_name":"v9.9.9","draft":false,"prerelease":false,"html_url":"https://example.test/release"}'
                    return '[]'
                if args[:3] == ('gh', 'release', 'download'):
                    dest = Path(args[args.index('--dir') + 1])
                    for source in (archive, sums):
                        (dest / source.name).write_bytes(source.read_bytes())
                return ''
            with patch.object(release, 'run', side_effect=run):
                url = release.publish('v9.9.9', 'f' * 40, 'notes', archive, sums)
            self.assertEqual(url, 'https://example.test/release')
            order = [c[:3] for c in commands]
            self.assertLess(order.index(('gh', 'release', 'download')), order.index(('gh', 'release', 'edit')))
            self.assertIn(('git', 'push', '--atomic'), order)

    def test_different_push_destination_is_rejected(self):
        def run(*args):
            return ('git@github.com:someone/another-repo.git' if '--push' in args
                    else 'git@github.com:Rikiar73574/mhgu-utility.git')
        with patch.object(release, 'run', side_effect=run):
            with self.assertRaisesRegex(ValueError, 'push destination'):
                release.check_remote()

    def test_existing_release_is_not_overwritten(self):
        def run(*args, **kwargs):
            if args[:2] == ('gh', 'api'):
                return '[{"tag_name":"v9.9.9"}]'
            raise AssertionError(('unexpected mutation', args))
        with patch.object(release, 'run', side_effect=run):
            with self.assertRaisesRegex(ValueError, 'already exists'):
                release.publish('v9.9.9', 'f' * 40, 'notes', Path('unused.zip'), Path('unused.sums'))


if __name__ == '__main__':
    unittest.main()
