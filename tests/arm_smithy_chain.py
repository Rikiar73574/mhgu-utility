"""Compiled ARM smithy chaining, with a peer that requires a stock entry.

The peer models the published smithy mods' prologue guard. The hook adapter
models Recomp forge.c: each original trampoline calls the previous detour.
No game process or third-party plugin is executed.
"""
import ast, gc, struct
from pathlib import Path
from unicorn import UC_HOOK_CODE
from unicorn.arm_const import UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2, UC_ARM_REG_R3

source = Path(__file__).with_name('arm_smoke.py')
prefix = []
for node in ast.parse(source.read_text()).body:
    if isinstance(node, ast.Expr) and isinstance(node.value, ast.Call) and isinstance(node.value.func, ast.Name) and node.value.func.id == 'call':
        break
    prefix.append(node)
fixture = compile(ast.Module(body=prefix, type_ignores=[]), str(source), 'exec')


def scenario(order, word=0xe92d41f0, destination=None, accepted=True, fail=False, ready=True, lazy=False):
    ns = {'__file__': str(source)}
    exec(fixture, ns)
    u, put, get, call = (ns[n] for n in ['u', 'put', 'get', 'call'])
    main, stub, heap = (ns[n] for n in ['MAIN', 'STUB', 'HEAP'])
    entry = main + 0x6f1ae8
    # A peer detour inside Recomp's actual plugin arena, not the fixture's BASE.
    peer = 0x03208000
    u.mem_map(0x03200000, 0x10000)
    stock, inert = stub + 0x8000, stub + 0x8004
    for at in [stock, inert]:
        u.mem_write(at, bytes.fromhex('1eff2fe1'))
    # push {r4,lr}; ldr r4,[pc,#4]; blx r4; pop {r4,pc}; .word stock
    u.mem_write(peer, struct.pack('<5I', 0xe92d4010, 0xe59f4004, 0xe12fff34, 0xe8bd8010, stock))
    counts = {'peer': 0, 'stock': 0, 'create': 0}
    installed = {}
    hints = []
    hint = heap + 0x4000

    def observe(uc, at, size, ctx):
        if at == peer:
            counts['peer'] += 1
        elif at == stock:
            counts['stock'] += 1
        elif at == inert and uc.reg_read(UC_ARM_REG_R0) == hint:
            hints.append(get(hint + 0x230))

    u.hook_add(UC_HOOK_CODE, observe, begin=peer, end=peer)
    u.hook_add(UC_HOOK_CODE, observe, begin=stock, end=stock)
    u.hook_add(UC_HOOK_CODE, observe, begin=inert, end=inert)

    def imports(uc, at, size, ctx):
        if ns['imports'].get(at) != 'forge_hook_create':
            return False
        out, target, detour, original = [uc.reg_read(r) for r in [UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2, UC_ARM_REG_R3]]
        if target == entry:
            counts['create'] += 1
            if fail:
                uc.reg_write(UC_ARM_REG_R0, 0)
                return True
            # Recomp's fnhook_add_trampoline dispatches the previous detour.
            put(original, installed.get(target, stock))
            installed[target] = detour
            put(target, 0xe51ff004)
            put(target + 4, detour)
        else:
            put(original, inert)
        uc.reg_write(UC_ARM_REG_R0, out)
        return True

    ns['storage_callback'] = imports
    if not ready:
        put(main + 0x86d34c, 0)  # Reject incompatible input before any native hooks.
    put(entry, word)
    if destination is not None:
        put(entry + 4, destination)

    def peer_init():
        assert (get(entry), get(entry + 4)) == (0xe92d41f0, 0xe24dd018), 'peer rejects already-patched smithy entry'
        installed[entry] = peer
        put(entry, 0xe51ff004)
        put(entry + 4, peer)

    for plugin in order:
        if plugin == 'utility':
            call(ns['symbols']['forge_onInit'])
            assert counts['create'] == 0, 'Utility must wait until all plugin initializers finish'
        elif not lazy:
            peer_init()
    # One complete callback round lets first-update peers inspect stock code.
    for plugin in order:
        if plugin == 'utility':
            call(ns['symbols']['forge_onUpdate'], 0)
            assert counts['create'] == 0, 'Utility must allow all first-update callbacks to finish'
        elif lazy:
            peer_init()
    call(ns['symbols']['forge_onUpdate'], 0)
    assert counts['create'] == int(accepted), (word, destination, counts)
    for _ in range(2):
        call(ns['symbols']['forge_onUpdate'], 0)
    assert counts['create'] == int(accepted), 'installation must be attempted only once'
    if accepted and not fail:
        assert installed.get(entry) not in [None, peer]
        # Real Utility callback -> peer -> stock; capture remains active afterward.
        put(hint, main + 0x1777c60)
        put(hint + 0x4c, 0x34)
        put(hint + 0x50, 33)
        for i in range(4):
            put(hint + 0x228 + 8 * i, 0xffffffff)
        put(hint + 0x228, 15)  # Native Help occupies one custom row.
        call(ns['symbols']['compendium_native_hint_refresh'], hint, 0x34, 33, 1)
        hints.clear()
        obj = heap + 0x8000
        data = bytearray(0x3218)
        data[4:6] = bytes([7, 25])
        struct.pack_into('<H', data, 0x5c, 10)
        struct.pack_into('<H', data, 0x1f78, 1)
        struct.pack_into('<HH', data, 0x28d4, 8, 1)
        data[0x28dd] = 4
        struct.pack_into('<H', data, 0x2240, 110)
        u.mem_write(obj, bytes(data))
        call(installed[entry], obj)
        assert counts['stock'] == 1 and counts['peer'] == int('peer' in order), counts
        assert hints == [132], 'selection capture and native wishlist hint must survive the chain'
    elif fail:
        assert entry not in installed, 'failed hook must not replace native code'
    del ns, u
    gc.collect()


scenario(['utility', 'peer'])
scenario(['peer', 'utility'])
scenario(['utility', 'peer'], lazy=True)
scenario(['peer', 'utility'], lazy=True)
scenario(['utility'])
scenario(['utility'], fail=True)
scenario(['utility'], accepted=False, ready=False)
for word, destination, accepted in [
    (0xe51ff004, 0x03200000, True),
    (0xe51ff004, 0x03dffffc, True),
    (0xe51ff004, 0x031ffffc, False),
    (0xe51ff004, 0x03e00000, False),
    (0xe51ff004, 0x03200001, False),
    (0xe51ff004, 0, False),
    (0xea000000, 0x03200000, False),
    (0xe92d4010, None, False),
]:
    scenario(['utility'], word, destination, accepted)
print('PASS: both init/first-update orders, previous-detour execution, stock path, disabled input, one-shot failure, Forge/foreign entry guards')
