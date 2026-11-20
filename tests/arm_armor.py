"""Navigate the compiled armor browser and persist a mixed weapon/armor wishlist."""
import ast,os,re,sys,tempfile,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
weapons=(ROOT/'build/generated/weapon_data.rs').read_text()
key=re.search(r'Weapon\{key:(\d+)',weapons)[1]
with tempfile.TemporaryDirectory(prefix='compendium-armor-') as directory:
    wishlist=Path(directory)/'wishlist.txt';wishlist.write_text(f'{key} 0\n')
    os.environ['COMPENDIUM_PREVIEW_WISHLIST']=str(wishlist)
    sys.path.insert(0,str(ROOT/'tools'))
    import preview_app as preview
    ns=preview.ns
    def press(bits):
        preview.frame({'kind':'button','bits':bits})
        return ns['texts'][:]
    for _ in range(4):press(1<<7)
    assert 'Armor' in ns['texts'] and 'Chest' in ns['texts'],ns['texts']
    press(1)
    def type_query(query):
        for character in query:
            preview.frame({'kind':'key','key':44 if character==' ' else ord(character)-ord('a')+4})
        return ns['texts'][:]
    out=type_query('leather')
    assert 'Leather X' in out and 'Leather S' in out,out
    assert all(label in out for label in ('HR  Blademaster','HR  Gunner','GR  Blademaster','GR  Gunner')),out
    assert any(text=='Search: leather|' for text in out),out
    preview.frame({'kind':'key','key':80})
    assert 'Search: leathe|r' in ns['texts'],ns['texts']
    preview.frame({'kind':'key','key':79})
    assert 'Search: leather|' in ns['texts'],ns['texts']
    # Physical arrows leave the editor and move the result in the same frame.
    preview.frame({'kind':'key','key':81})
    assert 'Search: leather' in ns['texts'] and 'Search: leather|' not in ns['texts'],ns['texts']
    out=press(1)
    assert 'Full set stats' in out and 'Full set skills' in out and 'Leather Cap S' in out,out
    database=Path(os.environ['MHGU_DATABASE']).resolve()
    cx=sqlite3.connect(database.as_uri()+'?mode=ro',uri=True)
    family=cx.execute('select family from armor join items using(_id) where name=?',('Leather Cap S',)).fetchone()[0]
    stats=cx.execute('select sum(defense),sum(max_defense),count(*),sum(num_slots) from armor where family=?',(family,)).fetchone()
    assert f'{stats[0]} / {stats[1]} max' in out,out
    assert f'{stats[2]} pieces    {stats[3]} slots total' in out,out
    for name,points in cx.execute('select t.name,sum(s.point_value) from item_to_skill_tree s join skill_trees t on t._id=s.skill_tree_id where s.item_id in(select _id from armor where family=?) group by t.name',(family,)):
        assert name in out and f'{points:+}' in out,(name,points,out)
    assert 'Gathering +1' in out and "Spirit's Whim" in out,out
    assert 'Unadorned set; no talisman or gems' in out,out
    assert all(slot in out for slot in ('Full set','Head','Chest','Arms','Waist','Legs')),out
    out=press(1<<14)
    assert 'Leather Cap S' in out and 'Stats & skill points' in out and 'Crafting materials' in out,out
    press(2);press(2);press(1)
    type_query('leather headgear')
    preview.frame({'kind':'key','key':82})
    assert 'Search: leather headgear' in ns['texts'] and 'Search: leather headgear|' not in ns['texts'],ns['texts']
    out=press(1)
    assert 'Full set stats' in out and 'Full set skills' in out,out
    out=press(1<<14)
    assert 'Leather Headgear' in out and 'Full set preview' in out and 'Stats & skill points' in out and 'Crafting materials' in out,out
    assert 'Leather Vest' in out and 'Leather Trousers' in out,out
    if os.environ.get('COMPENDIUM_SCREENSHOTS'):
        source=ROOT/'tests/arm_smoke.py'
        node=next(n for n in ast.parse(source.read_text()).body if isinstance(n,ast.FunctionDef) and n.name=='preview')
        exec(compile(ast.Module(body=[node],type_ignores=[]),str(source),'exec'),ns)
        ns['preview']('armor-details.png')
    press(1<<5)
    assert wishlist.read_text()==f'{key} 0\narmor:1310721 0\n',wishlist.read_text()
    # Open a material, return, then navigate a set piece without changing equipment.
    assert 'ITEM SOURCES' in press(1),ns['texts']
    assert 'Leather Headgear' in press(2),ns['texts']
    assert 'Leather Vest' in press(1<<14),ns['texts']
    press(2);press(1<<6)
    assert 'Wishlist' in ns['texts'] and 'Leather Headgear' in ns['texts'],ns['texts']
    # Up/Down reaches the armor's materials directly, without opening its details.
    weapon_count=next(line for line in weapons.splitlines() if line.startswith('Weapon{')).split('Recipe{',1)[1].split('Recipe{')[0].count('Material{')
    for _ in range(weapon_count):press(1<<15)
    assert 'ITEM SOURCES' in press(1),ns['texts']
    press(2);press(1<<5)
    assert wishlist.read_text()==f'{key} 0\n',wishlist.read_text()
    # The same item view exposes monster conditions, ranks, quantities and rates.
    press(1<<7)  # Armor
    press(1<<7)  # Set Builder
    press(1<<7)  # Items
    for character in 'rathalos scale':
        preview.frame({'kind':'key','key':44 if character==' ' else ord(character)-ord('a')+4})
    preview.frame({'kind':'key','key':40})
    out=press(1)
    assert 'Hyper Rathalos Scale' in out and 'Quest rewards' in out and 'HR' in out,out
    press(2);press(1<<15);out=press(1)
    assert 'Rathalos' in out and 'Body Carve' in out and 'LR' in out,out
    if os.environ.get('COMPENDIUM_SCREENSHOTS'):ns['preview']('monster-material-sources.png')
    print('ARM armor browsing, set links, material sources and mixed wishlist persistence passed')
