"""Check metadata against owned tables and fallback font coverage."""
from pathlib import Path
import os,struct,sys,tempfile,unittest
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
from localization import generate,hash_name,TABLES
from game_names import strings
class LocalizationTests(unittest.TestCase):
 def test_generated_slots_and_korean_mapping(self):
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp);table=root/'table';table.mkdir()
   for name in TABLES:(table/f'{name}_eng.gmd').write_bytes(b'GMD\0'+bytes(37)+b'Name\0Description\0')
   output=root/'slots.rs';generate(table,output)
   raw=output.with_suffix('.bin').read_bytes();self.assertEqual(raw[:8],b'UNAM\x01\0\0\0')
   records=list(struct.iter_unpack('<QBH',raw[8:]));self.assertEqual(len(records),len(TABLES)+1)
   self.assertEqual(records[0],(hash_name('Name'),0,0))
   text=output.read_text();self.assertNotIn('Description',text);self.assertNotIn('Name',text)
   self.assertEqual(bytes([0xb0,0xa1]).decode('euc_kr'),'가')
 def test_fonts_cover_japanese_and_korean(self):
  from fontTools.ttLib import TTFont
  for locale,text in [('jpn','鉄鉱石あいうアイウ'),('kor','가각힣공격력')]:
   cmap=TTFont(ROOT/f'assets/fonts/{locale}.ttf').getBestCmap()
   self.assertTrue(all(ord(ch)in cmap for ch in text),locale)
 def test_owned_italian_tables_keep_slot_layout(self):
  root=Path(os.environ.get('MHGU_GAME_DIR','/nonexistent'))/'romfs/nativeNX'
  if not root.exists():self.skipTest('owned game dump not configured')
  for name in TABLES:
   en=strings(root/f'eng/table/{name}_eng.gmd');it=strings(root/f'ita/table/{name}_ita.gmd')
   self.assertEqual(len(en),len(it),name)
  en=strings(root/'eng/table/itemData_eng.gmd');it=strings(root/'ita/table/itemData_ita.gmd')
  index=en.index('Iron Ore');self.assertNotEqual(en[index],it[index]);self.assertTrue(it[index])
if __name__=='__main__':unittest.main()
