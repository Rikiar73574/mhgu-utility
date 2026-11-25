"""Offline star-based material access, never proof of native recipe unlocks."""
import ast
from collections import defaultdict
from functools import lru_cache
import re


def array(text, name):
    raw = re.search(r'pub (?:static|const) '+name+r'[^=]*=&\[(.*?)\];', text, re.S)[1]
    return ast.literal_eval('['+raw+']')


def records(text, kind):
    for row in re.findall(r'^'+kind+r'\{(.*?)\},?$', text, re.M):
        yield {k: int(v) for k, v in re.findall(r'(\w+):(-?\d+)', row)}, row


def quest_in_stars(hub, rank, stars, village, guild):
    if hub == 'Village':
        return stars <= village
    if hub == 'Guild':
        return (rank < 2 and stars <= min(guild, 7)) or (rank == 2 and guild >= 8 and stars <= guild-7)
    return False


@lru_cache(maxsize=32)
def prepare(root, village, guild):
    """Return a candidate file with provisional bits set on every non-owned piece."""
    generated = root/'build/generated'
    snapshot = __import__('json').loads((root/'build/web-set-preview/snapshot.json').read_text())
    quests = (generated/'quest_data.rs').read_text()
    indexed, sources = set(), defaultdict(set)
    for source, _ in records(quests, 'Source'):
        indexed.add(source['item'])
        sources[source['kind'], source['rank'], source['entity']].add(source['item'])
    accessible = set()
    for index, (q, raw) in enumerate(records(quests, 'Quest')):
        hub = re.search(r'hub:"([^"]+)"', raw)[1]
        if not quest_in_stars(hub, q['rank'], q['stars'], village, guild):
            continue
        accessible.update(sources[0, q['rank'], q['loc']])
        accessible.update(sources[2, q['rank'], index])
        monsters = ast.literal_eval('['+re.search(r'monsters:&\[(.*?)\]', raw)[1]+']')
        for monster in monsters:
            accessible.update(sources[1, q['rank'], monster])
    def materials_possible(items):
        # Missing source coverage remains provisional, never silently "locked".
        return all(item not in indexed or item in accessible for item in items)
    sets = (generated/'set_data.rs').read_text()
    materials = array(sets, 'ARMOR_MATERIALS')
    possible = set()
    for recipe, raw in records(sets, 'ArmorRecipe'):
        items = materials[recipe['material']:recipe['material']+recipe['material_count']]
        if materials_possible(items):
            for native in ast.literal_eval(re.search(r'native:(\[.*?\])', raw)[1]):
                possible.add((recipe['slot'], native))
    owned = {tuple(a) for a in snapshot['owned']}
    armor = [[] for _ in range(5)]
    for index, (a, _) in enumerate(records((generated/'armor_data.rs').read_text(), 'Armor')):
        if a['native'] == 65535 or a['hunter'] not in (2, snapshot['hunter']) or a['gender'] not in (2, snapshot['gender']):
            continue
        key = a['slot'], a['native']
        if key in owned or key in possible:
            armor[a['slot']].append(index if key in owned else index | 0x8000)
    materials = array(sets, 'MATERIALS')
    recipes = [r for r, _ in records(sets, 'Recipe')]
    jewels = []
    for index, (d, _) in enumerate(records(sets, 'Decoration')):
        alternatives = recipes[d['recipe']:d['recipe']+d['recipe_count']]
        if any(materials_possible(i for i, _ in materials[r['material']:r['material']+r['material_count']]) for r in alternatives):
            jewels.append(index | 0x8000)
    path = root/f'build/web-set-preview/scenario-v{village}-h{guild}.txt'
    path.write_text('\n'.join(','.join(map(str, values)) for values in armor+[jewels])+'\n')
    return path
