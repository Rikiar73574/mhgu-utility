"""Validate planning settings before starting any local search process."""
import importlib.util
from pathlib import Path
import sys
import unittest
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools/set_preview_web'))
from scenarios import quest_in_stars

class QuestPlanningTests(unittest.TestCase):
 def test_village_and_hub_are_independent_and_g_rank_is_separate(self):
  self.assertTrue(quest_in_stars('Village',1,8,8,4))
  self.assertTrue(quest_in_stars('Guild',1,4,8,4))
  self.assertFalse(quest_in_stars('Guild',1,5,10,4))
  self.assertFalse(quest_in_stars('Village',1,9,8,7))
  self.assertFalse(quest_in_stars('Guild',2,1,10,7))
  self.assertTrue(quest_in_stars('Guild',2,1,1,8))
  self.assertFalse(quest_in_stars('Guild',2,2,1,8))
  self.assertFalse(quest_in_stars('Event',0,1,10,11))

@unittest.skipUnless((ROOT/'build/web-set-preview/solver').exists(),'private preview inputs not configured')
class PreviewSettingsTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  spec=importlib.util.spec_from_file_location('set_preview_server',ROOT/'tools/set_preview_web/server.py')
  cls.server=importlib.util.module_from_spec(spec);spec.loader.exec_module(cls.server)
 def test_settings_reject_invalid_or_partial_progression_and_duplicate_priorities(self):
  for value in ({'order':[0,0,2,3]},{'village':8},{'village':11,'hub':4},{'village':8,'hub':12},{'owned_only':'yes'},{'skills':{'0':[18,18]}},{'skills':{'0':[999]}}):
   with self.subTest(value=value),self.assertRaises(ValueError):self.server.search_settings(value)
  settings=self.server.search_settings({'village':8,'hub':4,'order':[2,0,1,3],'skills':{'0':[71,18]}})
  self.assertEqual(settings['order'],[2,0,1,3]);self.assertEqual(settings['hub'],4)
 def test_zero_required_skills_is_valid_but_invalid_requests_stay_rejected(self):
  self.assertEqual(self.server.search_goals([]),[])
  for value in (None,'',{},[True],[-1],[9999],[33]*7,[33,33]):
   with self.subTest(value=value),self.assertRaises(ValueError):self.server.search_goals(value)
 def test_command_carries_priorities_slots_and_owned_only_to_real_solver(self):
  p=self.server.weapon_profile({'weapon':0,'slots':2});s=self.server.search_settings({'order':[2,0,1,3],'skills':{'0':[71,18]},'owned_only':True})
  cmd=self.server.command([33],p,s)
  self.assertEqual(cmd[5:10],['0','0','0','2','2,0,1,3'])
  self.assertEqual(cmd[10],'71:2,18:1');self.assertEqual(cmd[-1],'1')

if __name__=='__main__':unittest.main()
