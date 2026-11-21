"""Private-snapshot integration: run the production adapter, then audit its outputs."""
import ast,hashlib,json,os,re,subprocess,unittest
from pathlib import Path
from collections import defaultdict
R=Path(__file__).resolve().parents[1];P=R/'build/web-set-preview'
def array(source,name):return ast.literal_eval('['+re.search(r'pub (?:static|const) '+name+r'[^=]*=&\[(.*?)\];',source,re.S)[1]+']')
class SetPreviewTests(unittest.TestCase):
 @unittest.skipUnless((P/'build-info.json').is_file(),'private native snapshot not configured')
 def test_production_search_finds_and_independently_validates_five_skill_variations(self):
  proof=json.loads((P/'build-info.json').read_text())
  for name,digest in proof.items():
   if name not in ('revision','source_hash'):self.assertEqual(hashlib.sha256((R/name).read_bytes()).hexdigest(),digest,name)
  meta=json.loads(subprocess.check_output([str(P/'solver'),'--meta']))
  self.assertEqual(meta['snapshot_hash'],hashlib.sha256((P/'snapshot.json').read_bytes()).hexdigest())
  goal=next(s['id']for s in meta['skills']if s['name']=='Attack Up (L)')
  out=subprocess.check_output([str(P/'solver'),str(goal)],env=os.environ|{'MHGU_PREVIEW_TEST_FRAMES':'3602'},text=True,timeout=90)
  d=json.loads(out.splitlines()[-1]);self.assertEqual(d['search_version'],'native-v5');self.assertEqual(d['source_hash'],meta['source_hash'])
  self.assertEqual(d['count'],5);self.assertEqual(d['phase'],'variations');self.assertLess(d['elapsed'],60)
  result=d
  a=(R/'build/generated/armor_data.rs').read_text();s=(R/'build/generated/set_data.rs').read_text()
  armor=[]
  for row in re.findall(r'^Armor\{(.*?)\},$',a,re.M):
   d={k:int(v)for k,v in re.findall(r'(\w+):(-?\d+)',row)};d['name']=json.loads(re.search(r'name:("(?:\\.|[^"])*")',row)[1]);armor.append(d)
  armpts=array(a,'SKILLS');mapping=array(s,'ARMOR_TREES');decepts=array(s,'POINTS')
  decos=[{k:int(v)for k,v in re.findall(r'(\w+):(-?\d+)',row)}for row in re.findall(r'^Decoration\{(.*?)\},$',s,re.M)]
  entries=[json.loads(m)for m in re.findall(r'^Entry\{name:("(?:\\.|[^"])*")', (R/'build/generated/catalog.rs').read_text(),re.M)]
  # Entry vector offsets must match Rust's real catalog indexing.
  assert entries[decos[0]['entry']]=='Antidote Jwl 1'
  snap=json.loads((P/'snapshot.json').read_text())
  d=result
  witnesses=[]
  for b in d['builds']:
   pts=defaultdict(int);used=[0]*7;cap=[0]*7
   for slot,piece in enumerate(b['armor']):
    record=next(x for x in armor if x['slot']==slot and x['native']==piece['native'] and x['name']==piece['name'])
    ix=armor.index(record);assert any((v&0x7fff)==ix for v in snap['armor'][slot]);cap[slot]=record['slots']
    for tree,value in armpts[record['skill']:record['skill']+record['skill_count']]:pts[mapping[tree]]+=value
   if b['charm']:
    c=next(c for c in snap['charms']if c['box_index']+1==b['charm']['box_index']);cap[5]=c['slots']
    for t,p in zip(c['trees'],c['points']):
     if t:pts[t]+=p
   cap[6]=snap['weapon_slots']
   for j in b['jewels']:
    deco=next(x for x in decos if entries[x['entry']]==j['name']);ix=decos.index(deco);assert any((v&0x7fff)==ix for v in snap['jewels']);assert deco['slots']==j['slots'];used[j['gear']]+=j['slots']
    for t,p in decepts[deco['skill']:deco['skill']+deco['skill_count']]:pts[t]+=p
   # Current returned fixtures use ordinary additive gear. Fail instead of silently
   # applying incorrect arithmetic to special multiplier equipment.
   assert not any(pts[t]for t in (203,204,205)), 'special gear requires native multiplier evaluator'
   assert used==b['used']and cap==b['capacity'];assert all(u<=c for u,c in zip(used,cap))
   expected={t:((p+128)%256)-128 for t,p in pts.items()if p}
   reported={x['tree']:x['points']for x in b['skills']if x['points']}
   assert expected==reported,(b['rank'],expected,reported)
   assert expected[18]>=20
   assert not any(x['active']and 45<=x['tree']<=69 for x in b['skills'])
   if expected.get(85,0)>=10 and b['verified']and not b['penalties']:witnesses.append(b['rank'])
  assert witnesses,'Attack L alone still misses verified Guard +1'
  assert len({tuple((s['tree'],s['threshold'])for s in b['skills']if s['active'])for b in d['builds']})==5
  assert any(any(s['active']and s['activation']=='Gathering +1'for s in b['skills'])for b in d['builds'])
  assert [b['active_count']for b in d['builds']]==sorted([b['active_count']for b in d['builds']],reverse=True)
