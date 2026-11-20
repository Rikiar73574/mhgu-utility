"""Skill-effect data contracts shared by native UI and browser preview."""
import json,os,re,sys,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
import skill_effects

class SkillEffectsTests(unittest.TestCase):
 def test_numeric_mechanics_and_native_compound_identities(self):
  native={name:''for name in skill_effects.BASE}
  native.update({'Acrobat':'Combines the effects of "Constitution +1" and "Evasion +1".',
                 'Iron Wall':'Combines the effects of "Defense Up (M)" and "Iron Skin".',
                 'Pack Rat':'Combines the effects of "Charm Chaser" and "Gathering +1".'})
  # These families are generated from reviewed numeric formulas.
  native.update({name:''for name in ['Defense Up (M)','Defense Up (L)','Defense Down (M)','Critical Eye +3','Fire Atk +2']})
  descriptions=skill_effects.descriptions(native)
  self.assertIn('1.06',descriptions['Defense Up (L)'][1])
  self.assertIn('0.95',descriptions['Defense Down (M)'][1])
  self.assertIn('10 frames',descriptions['Acrobat'][1])
  self.assertIn('x0.75',descriptions['Acrobat'][1])
  self.assertIn('Defense Up (M)',descriptions['Iron Wall'][1])
  self.assertNotIn('Defense Up (L)',descriptions['Iron Wall'][1])
  self.assertIn('Gathering +1',descriptions['Pack Rat'][1])
  self.assertIn('50% chance',descriptions['Halve Stun'][1])
  self.assertIn('not verified',descriptions['Chance Seizer'][0])
 def test_unknown_or_cyclic_skill_cannot_fall_back_to_vague_native_text(self):
  with self.assertRaises(KeyError):skill_effects.descriptions({'Unknown':'Greatly increases damage.'})
  with self.assertRaises(ValueError):skill_effects.descriptions({'Loop':'Combines the effects of "Loop".'})
 def test_packed_effects_round_trip_utf8_and_repeated_tokens(self):
  texts=['the '*200,'é 猫 '+('the '*100),'the '*200]
  blob,pairs,locations=skill_effects.pack(texts)
  def expand(token):
   a,b=pairs[token]
   return bytes([token])if(a,b)==(0,0)else expand(a)+expand(b)
  self.assertEqual(locations[0],locations[2])
  for original,(offset,length)in zip(texts,locations):
   self.assertEqual(b''.join(expand(t)for t in blob[offset:offset+length]).decode(),original)
 def test_generated_activation_effects_are_complete(self):
  path=ROOT/'build/generated/set_data.rs'
  if not path.is_file():self.skipTest('No local generated game data')
  text=path.read_text();activations=re.findall(r'Activation\{tree:\d+,points:-?\d+,name:("(?:\\.|[^"\\])*")\}',text)
  self.assertGreater(len(activations),300)
  section=text.split('pub static EFFECTS:')[1].split(';')[0]
  effects=re.findall(r'\((\d+),(\d+)\)',section)
  self.assertEqual(len(effects),len(activations))
  blob=bytes(map(int,text.split('pub static EFFECT_TEXT:&[u8]=&[')[1].split(']')[0].split(',')))
  pairs=[tuple(map(int,p))for p in re.findall(r'\((\d+),(\d+)\)',text.split('pub static EFFECT_PAIRS:')[1].split(';')[1])]
  self.assertEqual(len(pairs),256)
  def decode(value):
   a,b=pairs[value]
   return bytes([value])if(a,b)==(0,0)else decode(a)+decode(b)
  from game_names import strings
  source=Path(os.environ.get('MHGU_GAME_DIR','/nonexistent'))/'romfs/nativeNX/eng/table/skillData_eng.gmd'
  expected=None
  if source.is_file():
   rows=strings(source);expected=skill_effects.descriptions({rows[i]:' '.join(rows[i+1].split())for i in range(0,len(rows),2)if rows[i]and rows[i+1].strip()})
  for name,(offset,length)in zip(activations,effects):
   out=b''.join(decode(b)for b in blob[int(offset):int(offset)+int(length)]).decode()
   self.assertTrue(out,json.loads(name))
   if expected:self.assertEqual(out,expected[json.loads(name)][1])
