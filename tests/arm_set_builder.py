"""Isolated compiled-ARM Set Builder integration; never opens a running game/save.

The real NSO is loaded read-only by arm_smoke's AST prefix. Native resource /
quest calls have deterministic read-only stubs: this tests adapter, solver, HID,
UI and wishlist integration, not the correctness of the native game routines.
"""
import ast
import re
import sys
import tempfile
from pathlib import Path
from unicorn import UC_HOOK_CODE, UC_HOOK_MEM_WRITE
from unicorn.arm_const import UC_ARM_REG_R0, UC_ARM_REG_LR, UC_ARM_REG_PC

ROOT = Path(__file__).resolve().parents[1]
source = ROOT / 'tests/arm_smoke.py'
prefix = []
for node in ast.parse(source.read_text()).body:
    if (isinstance(node, ast.Expr) and isinstance(node.value, ast.Call)
            and isinstance(node.value.func, ast.Name) and node.value.func.id == 'call'):
        break
    prefix.append(node)
n = {'__file__': str(source), '__name__': 'set_builder_integration'}
exec(compile(ast.Module(body=prefix, type_ignores=[]), str(source), 'exec'), n)
sys.path.insert(0, str(ROOT / 'tools'))
import preview_storage

with tempfile.TemporaryDirectory(prefix='compendium-set-builder-') as folder:
    path = Path(folder) / 'wishlist.txt'
    path.write_text('')
    preview_storage.install(n, path)
    u, M, put, get = n['u'], n['MAIN'], n['put'], n['get']
    fixture = 0xc000000
    u.mem_map(fixture, 0x100000)
    save, box, gc, manager, actor, pd = [fixture + x for x in
                                       (0, 0x10000, 0x30000, 0x31000, 0x32000, 0x35000)]
    root, table, resource, progress = [fixture + x for x in (0x37000, 0x38000, 0x39000, 0x40000)]
    for off, pointer in ((0x1897f78, save), (0x1856fe8, box), (0x188ca48, gc),
                         (0x1885b58, manager), (0x1856b18, root), (0x1884c58, progress)):
        put(M + off, pointer)
    put(save, M + 0x176af48)
    put(box, M + 0x173e3d0)
    put(manager + 0x1c, actor)
    put(actor + 0x13c4, pd)
    put(pd + 0x14c, 0)  # Great Sword / blademaster, male, town scene zero.
    u.mem_write(pd + 0x554, (999).to_bytes(2, 'little'))
    u.mem_write(pd + 0x1c, bytes((7, 0, 1, 0)))  # Weapon tag and nonzero ID.
    put(save + 0x958, (1 << 21) | (1 << 22))
    put(save + 0x3614, 0xffffffff)  # Full 2,000-record physical box.
    u.mem_write(save + 0x16c8, b'\xff' * (5 * 0x360))  # Remembered recipes.
    u.mem_write(progress + 0x500, b'\xff' * 0x800)
    put(root + 0x50, table)
    put(table + 0x64, 2)
    put(resource, M + 0x176af48)
    put(resource + 4, 2)  # Native resource ID, not a data pointer.
    put(M + 0x176af48 + 0xd8, M + 0x100)
    row = fixture + 0x39600
    put(row, M + 0x176b148)
    put(row + 4, 2)  # Inline row metadata, not a data pointer.
    put(M + 0x176b148 + 0x38, M + 0x200)
    for offset, opcode in ((0, 0xe5d00012), (4, 0xe12fff1e)):
        put(M + 0x200 + offset, opcode)
    # Deliberately no item stock singleton: missing stock must not block search.
    put(M + 0x1858438, 0)
    put(M + 0x18584b8, 0)
    for index, tree, points in ((0, 18, 1), (999, 19, 1), (1900, 191, 10)):
        record = box + 0x1c + index * 44
        u.mem_write(record + 4, bytes((6, 0, 1, 0)))
        u.mem_write(record + 0x10, bytes((tree, 0, points, 0, 0)))

    native_calls = {}
    def native(offset, result):
        def stub(uc, at, size, ctx):
            native_calls[offset] = native_calls.get(offset, 0) + 1
            uc.reg_write(UC_ARM_REG_R0, result)
            uc.reg_write(UC_ARM_REG_PC, uc.reg_read(UC_ARM_REG_LR))
        u.hook_add(UC_HOOK_CODE, stub, begin=M + offset, end=M + offset)
    native(0x13f03c, resource)  # Pure equipment resource resolution.
    native(0x100, row)  # Pure upgrade-level lookup, inline row has zero slots.
    native(0x54af8c, 1)  # Quest appearance; all modeled sources accessible.
    native(0x54ce08, 1)  # Entry eligibility.
    native(0x3ba50c, 0)  # No downloaded quest resource: stays unknown.
    fixture_before = bytes(u.mem_read(fixture, 0x100000))
    writes = []
    u.hook_add(UC_HOOK_MEM_WRITE, lambda uc, access, address, size, value, ctx:
               writes.append((address, size)), begin=fixture, end=fixture + 0xfffff)
    n['call'](n['symbols']['forge_onInit'])
    def render():
        return n['render']()
    def press(bits):
        n['hid'](0); render(); n['hid'](bits)
        out = render(); n['hid'](0)
        return out
    press(1 << 10)
    press((1 << 10) | (1 << 12))
    for _ in range(5):
        out = press(1 << 7)
    assert 'Choose activated skills' in out, out
    assert any('3 owned charms' in text for text in out), out
    out = press(1 << 2)
    assert 'Select at least one skill.' in out, out
    out = press(1)  # Alphabetically first positive activation: Acrobat.
    assert '1 / 6 selected' in out and 'Acrobat' in out, out
    out = press(1 << 2)
    assert 'Checking progression availability (not crafting stock)...' in out, out
    out = press(1 << 2)  # Pause during preparation and resume.
    assert 'Y  Search / resume' in out, out
    out = press(1 << 2)
    for frames in range(2500):
        out = render()
        if any(text.startswith('Build 1 /') for text in out):
            break
        assert not any('capacity exceeded' in text.lower() or 'snapshot or search data is invalid' in text for text in out), out
    else:
        raise AssertionError(('No result before frame budget', out))
    # Pause so browsing and wishlist assertions use a stable best-ranked solution.
    if 'Y  Pause search' in out:
        out = press(1 << 2)
    assert 'No talisman required' in out, out
    assert any(re.fullmatch(r'Acrobat  (\d+) / 10', text) and int(re.fullmatch(r'Acrobat  (\d+) / 10', text)[1]) >= 10 for text in out), out
    assert out.count('Missing') == 5, out
    assert any(re.fullmatch(r'Base defense \d+   Slots: 0 used / \d+ total', text) for text in out), out
    assert any(text.startswith('B  All skills (') for text in out), out
    out = press(1 << 5)
    rows = path.read_text().splitlines()
    assert len(rows) == 5 and all(row.startswith('armor:') for row in rows), rows
    assert len(set(rows)) == 5, rows
    press(1 << 5)
    assert path.read_text().splitlines() == rows, 'Adding missing armor was not idempotent'
    # Resuming keeps exploring and preserves or improves the requested fit.
    # This real-catalog fixture need not fill the retained pool quickly.
    initial_points = int(next(re.fullmatch(r'Acrobat  (\d+) / 10', text)[1]
                              for text in out if re.fullmatch(r'Acrobat  (\d+) / 10', text)))
    press(1 << 2)
    for _ in range(500):
        out = render()
        summary = next((text for text in out if ' builds found    ' in text), '')
        if summary and int(summary.split()[0]) >= 2:
            break
    else:
        raise AssertionError(('Search never produced a second distinct build', out))
    for _ in range(32):
        out = render()
    summary = next(text for text in out if ' builds found    ' in text)
    pool_size, steps = map(int, re.fullmatch(r'(\d+) builds found    (\d+) search steps', summary).groups())
    assert 2 <= pool_size <= 24, out
    assert 'Y  Pause search' in out, out
    before = steps
    for _ in range(8):
        out = render()
    summary = next(text for text in out if ' builds found    ' in text)
    assert int(summary.split()[3]) > before, out
    out = press(1 << 2)
    best_points = int(next(re.fullmatch(r'Acrobat  (\d+) / 10', text)[1]
                           for text in out if re.fullmatch(r'Acrobat  (\d+) / 10', text)))
    assert best_points <= initial_points, (initial_points, best_points, out)
    out = press(1 << 14)
    assert any(text.startswith('Build 2 /') for text in out), out
    out = press(1 << 12)
    assert any(text.startswith('Build 1 /') for text in out), out
    out = press(1)
    assert 'Crafting materials' in out, out
    out = press(2)
    assert 'No talisman required' in out, out
    slot_summary = next(re.fullmatch(r'Base defense \d+   Slots: (\d+) used / (\d+) total', text)
                        for text in out if text.startswith('Base defense '))
    for _ in range(5):
        press(1 << 15)
    out = press(1)
    assert 'Decoration placement' in out and 'No decorations required.' in out, out
    placement = next(text for text in out if all(part in text for part in ('Head', 'Chest', 'Arms', 'Waist', 'Legs', 'Charm', 'Weapon')))
    per_piece = re.findall(r'(Head|Chest|Arms|Waist|Legs|Charm|Weapon) ?(\d+)/(\d+)', placement)
    assert len(per_piece) == 7, out
    assert all(int(used) <= int(capacity) for _, used, capacity in per_piece), out
    assert sum(int(used) for _, used, _ in per_piece) == int(slot_summary[1]), out
    assert sum(int(capacity) for _, _, capacity in per_piece) == int(slot_summary[2]), out
    for bits in (1 << 15, 1 << 14, 1 << 13, 1 << 12):
        press(bits)  # Empty jewel list is bounded in every direction.
    out = press(2)
    assert 'No talisman required' in out, out
    # The seventh result card exposes every contributing tree, including
    # unrequested inactive/negative points that the compact summary omits.
    press(1 << 15)
    out = press(1)
    assert 'All skills' in out and 'Skill tree' in out and 'Activated skill' in out and 'Points' in out, out
    assert 'Nimbleness' in out and 'Acrobat' in out, out
    assert 'Not active' in out, out
    assert any(re.fullmatch(r'[+-]\d+', text) for text in out), out
    page = next(text for text in out if re.fullmatch(r'\d+ active skills   \d+ skill trees   Page \d+ / \d+', text))
    trees = int(page.split()[3])
    assert trees > 1, out
    pages = int(page.rsplit(' ', 1)[1])
    for _ in range(pages + 1):
        out = press(1 << 14)
    assert any(text.endswith(f'Page {pages} / {pages}') for text in out), out
    for _ in range(pages + 1):
        out = press(1 << 12)
    assert any(text.endswith(f'Page 1 / {pages}') for text in out), out
    out = press(2)
    assert any(text.startswith('Build 1 /') for text in out), out
    out = press(2)
    assert 'Choose activated skills' in out and '1 / 6 selected' in out, out
    out = press(2)
    assert 'MHGU Utility' not in out, out
    press(1 << 10)
    out = press((1 << 10) | (1 << 12))
    assert 'Choose activated skills' in out, out
    out = press(1 << 6)
    assert 'Armor sets' in out and 'Choose activated skills' not in out, out
    assert not writes, writes
    assert bytes(u.mem_read(fixture, 0x100000)) == fixture_before
    assert native_calls.get(0x13f03c, 0) >= 4 and native_calls.get(0x54ce08, 0) > 0, native_calls
    print('ARM Set Builder passed: town capture, 3 charms in full physical box, skill/search/pause, '
          'missing stock, result pagination/details/back, empty jewels/slot allocation, all skill trees/activation/paging, 5 missing wishlist pieces/idempotence, '
          'no fixture writes; preparation frames:', frames + 1)
