"""Native set facts: reject namespace drift and preserve signed thresholds."""
import json
import os
from pathlib import Path
import sqlite3
import struct
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import set_data


class SetDataTests(unittest.TestCase):
    def test_saturated_thresholds_keep_first_crossing_for_both_signs(self):
        row = struct.pack('<IB7H', 0, 0, 1, 1, 2, 3, 3, 4, 4)
        self.assertEqual(set_data.activations([row], ['', 'severe', 'mild', 'good', 'better']),
                         [(0, -15, 'severe'), (0, -10, 'mild'), (0, 10, 'good'), (0, 20, 'better')])

    def test_zero_activation_is_absent(self):
        self.assertEqual(set_data.activations([bytes(19)], ['']), [])

    def test_ambiguous_activation_sign_rejected(self):
        row = struct.pack('<IB7H', 0, 0, 1, 1, 1, 1, 1, 1, 1)
        with self.assertRaisesRegex(ValueError, 'both signs'):
            set_data.activations([row], ['', 'invalid'])

    def test_truncated_records_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'test.native'
            path.write_bytes(b'abcd' + struct.pack('<I', 2) + b'12345')
            with self.assertRaisesRegex(ValueError, 'record mismatch'):
                set_data.records(path, 5, b'abcd')

    @unittest.skipUnless(os.environ.get('MHGU_DATABASE') and os.environ.get('MHGU_GAME_DIR'), 'owned native inputs not configured')
    def test_all_real_native_jewels_and_activations_match_database(self):
        root = Path(__file__).resolve().parents[1]
        db = sqlite3.connect(Path(os.environ['MHGU_DATABASE']).resolve().as_uri() + '?mode=ro', uri=True)
        catalog = json.loads((root / 'build/data/catalog.json').read_text())
        trees, mapping, active, jewels, points, recipes, materials = set_data.facts(db, catalog, Path(os.environ['MHGU_GAME_DIR']))
        expected = set(db.execute('select t.name,s.required_skill_tree_points,s.name from skills s join skill_trees t on t._id=s.skill_tree_id'))
        self.assertEqual({(trees[t], p, n) for t,p,n in active}, expected)
        self.assertEqual(len(jewels), db.execute('select count(*) from decorations').fetchone()[0])
        self.assertEqual(len(set(j[0] for j in jewels)), len(jewels))
        self.assertTrue(any(v < 0 for _,v in points))
        self.assertTrue(all(1 <= j[2] <= 3 and j[4] and j[6] for j in jewels))
        self.assertTrue(all(g >= 0 and count > 0 for g,_,count in recipes))
        self.assertEqual((trees[203], trees[204], trees[205]), ('Torso Up', 'Secret Arts', 'Talisman Boost'))
        self.assertEqual(len(mapping), 187)
        # Rough compiled static-data estimate: ARM32 fat string pointers8,
        # Activation12, Decoration14, Recipe6, aligned tuple4; names UTF8.
        estimated = sum(len(n.encode()) for n in trees) + len(trees)*8 + len(mapping) + sum(len(n.encode()) for _,_,n in active) + len(active)*12 + len(jewels)*14 + len(points)*2 + len(recipes)*6 + len(materials)*4
        self.assertLess(estimated, 60*1024)
        armor_materials = []
        armor = set_data.armor_recipes(catalog, Path(os.environ['MHGU_GAME_DIR']), armor_materials)
        self.assertEqual(len(armor), 2720)
        head = {r[2]: r for r in armor if r[5] == 0}
        # Native LR/HR/G versions have the SAME opaque gate zero. Rarity
        # must never be turned into fabricated progression metadata.
        self.assertEqual([head[i][1] for i in (69,215,436,506)], [0,0,0,0])
        self.assertEqual([head[i][0][0] for i in (69,215,436,506)], [150,446,916,1066])
        entries = sorted(catalog['items'].values(), key=lambda r: r['name'])
        def material_names(row):
            return [entries[i]['name'] if i != 65535 else 'UNKNOWN' for i in armor_materials[row[3]:row[3]+row[4]]]
        self.assertEqual(set(material_names(head[69])), {'Rathalos Scale','Rathalos Shell','Rath Wingtalon','Dragonite Ore'})
        self.assertEqual(set(material_names(head[506])), {'Ahtal-Ka Purple Cocoon','Ahtal-Ka Fine Silk','Ahtal-Ka Pheromone','S.Queen Moonlash'})
        self.assertEqual(head[101][-2:], (13,5))
        self.assertNotIn('Rathalos Scrap', material_names(head[69]))
        self.assertTrue(65535 in armor_materials)
        self.assertLess(len(armor)*20+len(armor_materials)*2, 75*1024)
        db.close()


if __name__ == '__main__':
    unittest.main()
