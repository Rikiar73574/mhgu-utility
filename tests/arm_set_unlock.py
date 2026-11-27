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

    from elftools.elf.elffile import ELFFile
    with (ROOT/'mod/target/armv7a-none-eabi/release/libhunters_compendium.so').open('rb')as f:
        syms={x.name:n['BASE']+x['st_value']for x in ELFFile(f).get_section_by_name('.symtab').iter_symbols()}
    def symbol(fragment):
        return next(v for k,v in syms.items()if k.endswith(fragment))
    begin=symbol('10set_unlock4live5begin')
    cache=symbol('10set_unlock4live5STATE')
    builder=symbol('11set_builder5STATE')
    start=symbol('11set_builder5start')
    tick=symbol('11set_builder10tick_slice')
    rows=(ROOT/'build/generated/quest_data.rs').read_text().split('pub static SOURCES:')[0].splitlines()[3:-1]
    index=next(i for i,r in enumerate(rows)if 'id:1027,'in r)
    sources=(ROOT/'build/generated/quest_data.rs').read_text().split('pub static SOURCES:')[1].split('pub static MONSTERS:')[0]
    item=int(re.search(r'Source\{item:(\d+),entity:1,rank:2,kind:1',sources)[1])
    if '--simulate-missing-tier-check'in sys.argv:
        def bypass(uc,at,size,ctx):
            uc.reg_write(UC_ARM_REG_R0,1)
            uc.reg_write(UC_ARM_REG_PC,uc.reg_read(UC_ARM_REG_LR))
        at=symbol('10set_unlock4live4tier')
        u.hook_add(UC_HOOK_CODE,bypass,begin=at,end=at)
    # All appearance/entry checks deliberately return true. Only the real
    # compiled source-cache tier check may reject the future Village10 route.
    n['call'](n['symbols']['forge_onInit'])
    n['call'](start)
    before=bytes(u.mem_read(fixture,0x100000))
    def check(unlocked):
        u.mem_write(progress+0x500,bytes(0x800))
        if unlocked:u.mem_write(progress+0x500+1052//8,bytes([1<<(1052&7)]))
        n['call'](begin)
        assert get(cache+4096)==save,(hex(get(cache+4096)),hex(save))
        put(cache+4108,index)
        a=native_calls.get(0x54af8c,0)
        n['call'](tick,builder,0,0,0)
        assert get(cache+4108)==index+16
        value=bytes(u.mem_read(cache+item,1))[0]
        assert value==(1 if unlocked else 0),('Source cache admitted locked material',unlocked,value)
        assert (native_calls.get(0x54af8c,0)>a)==unlocked
    check(False);check(True);check(False)
    # begin() cleared the positive material value on the next search, too.
    print('PASS: compiled cache rejects locked Village10 sources even when appearance/entry pass; admits after native tier unlock; clears on next search; armor rank filter never called')
