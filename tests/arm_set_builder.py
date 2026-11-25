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
preview_node=next(node for node in ast.parse(source.read_text()).body if isinstance(node,ast.FunctionDef) and node.name=='preview')
exec(compile(ast.Module(body=[preview_node],type_ignores=[]),str(source),'exec'),n)
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

    fixture_before=bytes(u.mem_read(fixture,0x100000));writes=[]
    u.hook_add(UC_HOOK_MEM_WRITE,lambda uc,access,address,size,value,ctx:writes.append((address,size)),begin=fixture,end=fixture+0xfffff)
    n['call'](n['symbols']['forge_onInit'])
    def render():return n['render']()
    def press(bits):
        n['hid'](0);render();n['hid'](bits);render();n['hid'](0);return render()
    def click(text,index=0):
        op=[o for o in n['drawops']if o[0]=='text'and o[2][1]==text][index]
        n['touch']=((op[1][0]+3)/1.125,(op[1][1]+3-45)/1.125);render();n['touch']=None;return render()
    press(1<<10);press((1<<10)|(1<<12))
    for _ in range(5):out=press(1<<7)
    assert 'Navigation'in out and 'Open Set Builder'in out and 'Required skills (optional)'not in out,out
    out=press(1<<6);assert 'Armor sets'in out and 'Navigation'in out,out
    out=press(1<<7);assert 'Open Set Builder'in out and 'Required skills (optional)'not in out,out
    out=press(1<<7);assert 'Item list'in out and 'Navigation'in out,out
    out=press(1<<6);assert 'Open Set Builder'in out and 'Required skills (optional)'not in out,out
    n['preview']('native-set-builder-entry.png')
    out=press(1)
    assert 'Required skills (optional)'in out and 'Saved builds'in out and 'Navigation'not in out,out
    assert any('3 owned charms'in t for t in out),out
    # Dedicated tabs use LB/RB, retaining the ordinary Utility on Back.
    out=press(1<<7);assert 'Set Builder settings'in out,out
    ctrl=[True]
    def ctrl_key(uc,at,size,ctx):
        if n['imports'].get(at)=='forge_input_isCtrlDown':uc.reg_write(UC_ARM_REG_R0,int(ctrl[0]))
    u.hook_add(UC_HOOK_CODE,ctrl_key,begin=n['STUB'],end=n['STUB']+0xffff)
    n['key_pressed'].add(9);out=render();n['key_pressed'].clear();ctrl[0]=False;render()
    assert 'Typing searches directly'not in out,'Settings shortcut opened an invisible search editor'
    if '--priority-only'in sys.argv:
        out=click('Skill Priorities')
        storage=path.with_suffix('.set-builder.txt');before=storage.read_text()if storage.exists()else None
        out=press(1<<14)
        after=storage.read_text()if storage.exists()else None
        assert after==before,'Left/Right must not swap whole priorities'
        print('PASS: numbered priority slots stay fixed')
        sys.exit(0)
    if '--search-only'not in sys.argv and '--recommendation-only'not in sys.argv:
        # Settings have separate Requirements and numeric Priority 1..4 pages.
        out=click('Requirements');assert 'Village stars'in out and 'Hub stars'in out,out
        assert 'Use automatic native unlock checks'not in out and '<'not in out and '>'not in out,out
        n['preview']('native-set-builder-requirements.png')
        out=click('Village stars');assert 'Village progression'in out and 'Automatic'in out,out
        n['preview']('native-set-builder-dropdown.png')
        out=click('8 stars');assert 'Requirements'in out,out
        storage=path.with_suffix('.set-builder.txt');prefs=storage.read_text();assert 'village=8\n'in prefs and 'hub=255\n'in prefs,prefs
        out=click('Hub stars');out=click('4 stars');prefs=storage.read_text();assert 'village=8\n'in prefs and 'hub=4\n'in prefs,prefs
        out=click('Village stars');out=click('Automatic');prefs=storage.read_text();assert 'village=255\n'in prefs and 'hub=4\n'in prefs,prefs
        out=click('Hub stars');out=press(1<<15);out=press(2);assert 'Requirements'in out,out
        assert 'hub=4\n'in storage.read_text(),'Cancel changed the option'
        out=click('Hub stars');out=click('Automatic');assert 'hub=255\n'in storage.read_text()
        out=click('Owned armor only');assert 'owned_only=1\n'in storage.read_text()
        out=click('Owned armor only');assert 'owned_only=0\n'in storage.read_text()
        out=press(2);assert 'Set Builder settings'in out,out
        out=click('Skill Priorities');assert all('Priority '+str(i)in out for i in range(1,5)),out
        assert 'Damage'not in out and 'Weapon support'not in out,out
        n['preview']('native-set-builder-priorities.png')
        out=click('Priority 1');before=[t for t in out if re.match(r'^\d+\. ',t)];assert len(before)>=4,out
        before_order=storage.read_text();press(2);out=press(1<<14)
        assert storage.read_text()==before_order,'Priority slots moved as whole sections'
        out=click('Priority 1')
        names=lambda rows:[r.split('. ',1)[1]for r in rows]
        out=click('Move to priority...',0);out=click('Priority 4');prefs=storage.read_text();assert 'groups=default'not in prefs,prefs
        out=press(2);assert 'Skill Priorities'in out,out
        out=click('Priority 4');assert names(before)[0]in names([t for t in out if re.match(r'^\d+\. ',t)]),out
        out=press(2);out=click('Priority 2');second=[t for t in out if re.match(r'^\d+\. ',t)];moved=names(second)[0]
        out=click('Move to priority...',0);out=click('Priority 1');out=press(2);out=click('Priority 1')
        assert moved in names([t for t in out if re.match(r'^\d+\. ',t)]),out
        out=press(2);out=press(2);assert 'Set Builder settings'in out,out
        out=click('Reset Set Builder Settings');prefs=storage.read_text();assert 'groups=default\n'in prefs and 'trees=default\n'in prefs,prefs
        assert 'village=255\n'in prefs and 'hub=255\n'in prefs,prefs
        n['preview']('native-set-builder-settings.png')
        print('PASS: separate Requirements/Priorities, independent Automatic dropdowns, cancel, checkbox, skill reassignment and standalone Reset',flush=True)
        if '--settings-only'in sys.argv:
            assert not writes and bytes(u.mem_read(fixture,0x100000))==fixture_before
            sys.exit(0)
    out=press(1<<7);assert 'Saved builds'in out,out
    out=press(2);assert 'Required skills (optional)'in out,out
    out=press(2);assert 'Navigation'in out and 'Open Set Builder'in out,out
    # Mouse confirmation uses the same entry as B/Enter.
    out=click('Open Set Builder')
    assert 'Required skills (optional)'in out and 'Navigation'not in out,out
    if '--navigation-only'in sys.argv:
        assert not writes and bytes(u.mem_read(fixture,0x100000))==fixture_before
        print('PASS: LB/RB visits and leaves landing page without opening builder; B and mouse confirmation enter; Back restores landing; native memory unchanged')
        sys.exit(0)
    if '--recommendation-only'in sys.argv:
        assert any('No required skills:'in t for t in out),out
        n['preview']('native-set-builder-optional-skills.png')
        press(1<<2)
        for frame in range(2500):
            out=render()
            if any(t.startswith('Build 1 /')for t in out):break
        else:raise AssertionError(('No unconstrained recommendation',out))
        assert 'Pieces, sockets & jewels'in out and 'Skills from this build'in out,out
        n['preview']('native-set-builder-recommendations.png')
        assert not writes and bytes(u.mem_read(fixture,0x100000))==fixture_before
        print('PASS: no required skills finds native rendered recommendations; SVC clock executes; character memory unchanged')
        sys.exit(0)
    press(1);press(1<<2)
    for frames in range(2500):
        out=render()
        if any(t.startswith('Build 1 /')for t in out):break
    else:raise AssertionError(('No build',out))
    press(1<<2)
    out=render();assert 'Pieces, sockets & jewels'in out and 'Skills from this build'in out,out
    assert 'Weapon'in out and 'Charm'in out and 'Missing'in out,out
    assert any('Acrobat'in t and '/ 10'in t for t in out),out
    # Prompts belong to their controls, with no detached footer legend.
    assert all(key in out for key in ('LB','RB','LT','RT')),out
    # Labels are positioned inside actual native shoulder artwork, not a new legend.
    for key in ('LB','RB','LT','RT'):
        pos=next(o[1] for o in n['drawops'] if o[0]=='text' and o[2][1]==key)
        pixels=[o for o in n['drawops'] if o[0]=='ImDrawList_AddRectFilled'
                and abs(o[1][0]-pos[0])<30 and abs(o[1][1]-pos[1])<30
                and 0<o[1][2]-o[1][0]<30 and 0<o[1][3]-o[1][1]<30]
        assert len(pixels)>5,(key,pixels)
    assert not any('LT: save' in t or 'workspace tabs' in t or t.startswith('R3  ') for t in out),out
    n['preview']('native-set-builder-results.png')
    print('PASS: dedicated navigation and complete result rendering',flush=True)
    # The visible gear/skill view must never enter the obsolete hidden jewel mode.
    for _ in range(5):press(1<<15)
    out=press(1);assert 'Skills from this build'in out and 'Pieces, sockets & jewels'in out,out
    out=press(2)
    for _ in range(5):press(1<<13)
    out=press(1);assert 'Crafting materials'in out,out
    out=press(2);assert 'Pieces, sockets & jewels'in out,out
    out=press(1<<5);rows=path.read_text().splitlines();assert len(rows)==5 and all(r.startswith('armor:')for r in rows),rows
    press(1<<5);assert path.read_text().splitlines()==rows,'Wishlist add was not idempotent'
    out=press(1<<9);assert any('pinned'in t for t in out),out
    out=click('Compare');assert 'Compare complete loadouts'in out,out
    out=click('Back to result');assert 'Pieces, sockets & jewels'in out,out
    # Persist, reopen and rename through actual ARM filesystem adapters.
    out=press(1<<8);assert any('Build saved'in t for t in out),out
    storage=path.with_suffix('.set-builder.txt');original=storage.read_text();assert 'builds=1'in original,original
    out=click('Saved builds');assert 'Acrobat'in out,out
    out=click('Rename');assert 'Rename saved build'in out,out
    n['key_pressed'].add(4);render();n['key_pressed'].clear();render() # append a to name
    n['key_pressed'].add(40);render();n['key_pressed'].clear();render() # finish editing without changing name
    out=click('Save name');assert 'Saved builds'in out and 'Acrobata'in out,out
    out=click('Open');assert 'Pieces, sockets & jewels'in out,out
    assert any('Saved loadout'in t for t in out),out
    # Current requirement drafts do not modify the stored socket layout.
    out=click('Settings');out=click('Requirements');out=click('Weapon slots');out=click('0 sockets')
    out=click('Saved builds');out=click('Open');assert 'Pieces, sockets & jewels'in out,out
    n['preview']('native-set-builder-saved.png')
    out=press(2);assert 'Saved builds'in out,out
    out=click('Settings');out=click('Reset Set Builder Settings')
    assert 'builds=1\n'in storage.read_text(),'Reset deleted saved builds'
    # Every memory write went to plugin/host scratch, not native character/equipment.
    assert bytes(u.mem_read(fixture,0x100000))==fixture_before,(writes[:10],out)
    assert not writes,writes[:10]
    print('PASS: compiled ARM dedicated navigation, combined slots/jewels/skills, saved persistence/rename/reopen, frozen sockets, planning settings; character memory unchanged')
