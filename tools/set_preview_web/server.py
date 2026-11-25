from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
from pathlib import Path
import hashlib,json,subprocess,threading,uuid,time,urllib.parse,sys
import scenarios
WEB=Path(__file__).resolve().parent;ROOT=WEB.parents[1];P=ROOT/'build/web-set-preview';SNAP=json.loads((P/'snapshot.json').read_text());META=json.loads(subprocess.check_output([str(P/'solver'),'--meta']));SKILLS={s['id']:s for s in META['skills']};REV=META['revision'];JOBS={};LOCK=threading.Lock()
PROFILE={k:SNAP[k]for k in('hr','gender','hunter','weapon_slots','captured_at')};PROFILE['charms']=len(SNAP['charms']);PROFILE['charms_captured_at']=SNAP.get('charms_captured_at',SNAP['captured_at'])
assert META['snapshot_hash']==hashlib.sha256((P/'snapshot.json').read_bytes()).hexdigest(),'Rebuild the preview for this snapshot'
META.update(profile=PROFILE,revision=REV,search_version="builder-ui-v1",armor_count=sum(map(len,SNAP['armor'])),jewel_count=len(SNAP['jewels']))
def weapon_profile(value):
 if not isinstance(value,dict):raise ValueError('Invalid weapon profile')
 p=dict(weapon=14,element=0,status=0,slots=SNAP['weapon_slots']);p.update(value)
 if set(p)!={'weapon','element','status','slots'}:raise ValueError('Unknown weapon setting')
 if any(type(p[k])is not int for k in ('weapon','element','status','slots')):raise ValueError('Invalid weapon settings')
 if not 0<=p['weapon']<=14 or p['element']not in (0,27,28,29,30,31)or p['status']not in (0,1,2,3,92)or not 0<=p['slots']<=3:raise ValueError('Invalid weapon settings')
 if p['weapon']!=14 and (p['weapon']>=11)!=bool(SNAP['hunter']):raise ValueError('Weapon class differs from this captured armor availability. Use a matching character capture.')
 return p

def search_goals(value):
 if not isinstance(value,list)or len(value)>6 or any(type(g)is not int or g not in SKILLS for g in value):raise ValueError('Choose up to six optional skills')
 if len({SKILLS[g]['tree']for g in value})!=len(value):raise ValueError('Choose distinct skill trees')
 return list(value)

def search_settings(value):
 if not isinstance(value,dict):raise ValueError('Invalid settings')
 result=dict(order=[0,1,2,3],skills={},village=None,hub=None,owned_only=False);result.update(value)
 if set(result)!={'order','skills','village','hub','owned_only'}:raise ValueError('Unknown setting')
 if not isinstance(result['order'],list)or any(type(n)is not int for n in result['order'])or sorted(result['order'])!=[0,1,2,3]:raise ValueError('Invalid category order')
 for name,maximum in [('village',10),('hub',11)]:
  n=result[name]
  if n is not None and (type(n)is not int or not 0<=n<=maximum):raise ValueError('Invalid stars')
 if (result['village']is None)!=(result['hub']is None):raise ValueError('Set both star limits for a planning scenario')
 if type(result['owned_only'])is not bool or not isinstance(result['skills'],dict):raise ValueError('Invalid equipment or skill settings')
 seen=set()
 for category,trees in result['skills'].items():
  if category not in ('0','1','2','3')or not isinstance(trees,list)or len(trees)>205:raise ValueError('Invalid skill order')
  for tree in trees:
   if type(tree)is not int or tree not in {s['tree']for s in SKILLS.values()}or tree in seen:raise ValueError('Invalid/duplicate skill tree')
   seen.add(tree)
 return result

def command(goals,profile,settings):
 boosts=','.join(f'{tree}:{len(trees)-i}'for trees in settings['skills'].values()for i,tree in enumerate(trees))
 scenario=str(scenarios.prepare(ROOT,settings['village'],settings['hub']))if settings['village']is not None else ''
 return ['nice','-n','10',str(P/'solver'),','.join(map(str,goals)),str(profile['weapon']),str(profile['element']),str(profile['status']),str(profile['slots']),','.join(map(str,settings['order'])),boosts,scenario,str(int(settings['owned_only']))]

def validate(d,goals):
 assert d.get('source_hash')==META['source_hash']
 assert len(d['builds'])<=24
 assert [b['ranking_key']for b in d['builds']]==sorted(b['ranking_key']for b in d['builds'])
 seen=set();skill_sets=set()
 for b in d['builds']:
  assert b['active_count']==sum(bool(s['active'])for s in b['skills'])
  signature=tuple((s['tree'],s['threshold'])for s in b['skills']if s['active']);assert signature not in skill_sets;skill_sets.add(signature)
  points={s['tree']:s['points']for s in b['skills']}
  for g in goals:assert points.get(SKILLS[g]['tree'],0)>=SKILLS[g]['points']
  used=[0]*7
  for j in b['jewels']:assert 0<=j['gear']<7 and 1<=j['slots']<=3;used[j['gear']]+=j['slots']
  assert used==b['used'];assert all(0<=a<=c<=3 for a,c in zip(used,b['capacity']))
 if d.get("search_version")=="builder-ui-v1":
  assert d["variations"]==len(skill_sets)
  if d["phase"]=="variations":assert d["batch_variations"]==5 and d["elapsed"]<60.2
  if d["phase"]=="timeout":assert 60<=d["elapsed"]<61
 return d

def latest():
 for f in (P/'latest.json',P/'attack-results.jsonl'):
  if not f.exists():continue
  for line in reversed(f.read_text().splitlines()):
   try:
    d=json.loads(line);d.setdefault('goals',[]);
    if d.get('search_version')!='builder-ui-v1' or d.get('source_hash')!=META['source_hash']:continue
    d['can_resume']=bool(d.get('job')in JOBS and JOBS[d['job']].get('process')and JOBS[d['job']]['process'].poll()is None and d['phase']in('variations','timeout'));return validate(d,d['goals'])
   except (ValueError,KeyError,AssertionError):continue
 return {'builds':[],'count':0,'phase':'ready','visited':0}

def worker(jid,goals,profile,settings):
 j=JOBS[jid]
 try:
  process=subprocess.Popen(command(goals,profile,settings),stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,bufsize=1);j['process']=process
  for line in process.stdout:
   d=validate(json.loads(line),goals);d['goals']=goals;d['weapon_profile']=profile;d['settings']=settings;d['job']=jid;d['can_resume']=d['phase']in('variations','timeout');j['data']=d
   tmp=P/('result-'+jid+'.tmp');tmp.write_text(json.dumps(d));tmp.replace(P/'latest.json')
  error=process.stderr.read();code=process.wait()
  if j.get('cancelled'):j['data']['phase']='cancelled'
  elif code:j['data'].update(phase='error',error='Search stopped unexpectedly. '+error[-300:])
 except Exception as e:j['data'].update(phase='error',error=str(e));p=j.get('process');p and p.poll()is None and p.kill()

class Handler(BaseHTTPRequestHandler):
 def send(self,code,value,typ='application/json'):
  body=value if isinstance(value,bytes)else json.dumps(value).encode();self.send_response(code);self.send_header('Content-Type',typ);self.send_header('Content-Length',str(len(body)));self.send_header('Cache-Control','no-store');self.send_header('X-Content-Type-Options','nosniff');self.end_headers();self.wfile.write(body)
 def do_GET(self):
  path=urllib.parse.urlsplit(self.path).path
  if path=='/':return self.send(200,(WEB/'index.html').read_bytes(),'text/html; charset=utf-8')
  if path in ('/app.js','/style.css'):return self.send(200,(WEB/path[1:]).read_bytes(),'text/javascript'if path.endswith('.js')else'text/css')
  if path=='/api/effects':return self.send(200,(P/'ui-effects.json').read_bytes())
  if path=='/api/categories':
   try:
    raw={k:int(v[0])for k,v in urllib.parse.parse_qs(urllib.parse.urlsplit(self.path).query).items()};profile=weapon_profile(raw);cmd=command([],profile,search_settings({}));cmd[4]='--meta';return self.send(200,json.loads(subprocess.check_output(cmd)))
   except Exception as e:return self.send(400,{'error':str(e)})
  if path=='/font.ttf':return self.send(200,(ROOT/'build/data/game-ui/native.ttf').read_bytes(),'font/ttf')
  if path=='/api/meta':return self.send(200,META)
  if path=='/api/latest':return self.send(200,latest())
  if path.startswith('/api/jobs/'):
   jid=path.rsplit('/',1)[-1];return self.send(200,JOBS[jid]['data'])if jid in JOBS else self.send(404,{'error':'Unknown search'})
  return self.send(404,{'error':'Not found'})
 def do_POST(self):
  origin=self.headers.get('Origin');host=self.headers.get('Host')
  if origin and urllib.parse.urlsplit(origin).netloc!=host:return self.send(403,{'error':'Local preview requests only'})
  path=urllib.parse.urlsplit(self.path).path
  if path=='/api/search':
   try:
    n=int(self.headers.get('Content-Length',0));assert 0<n<4096;request=json.loads(self.rfile.read(n));goals=search_goals(request['goals']);profile=weapon_profile(request.get('weapon_profile',{}));settings=search_settings(request.get('settings',{}))
   except Exception:return self.send(400,{'error':'Choose up to six optional distinct skills and valid weapon settings matching this captured hunter class'})
   with LOCK:
    if sum(j['data']['phase']in ('starting','searching','variations','timeout')for j in JOBS.values())>=2:return self.send(429,{'error':'Two searches are already running'})
    jid=uuid.uuid4().hex;JOBS[jid]={'started':time.time(),'data':{'phase':'starting','builds':[],'visited':0,'goals':goals,'weapon_profile':profile,'settings':settings}}
   threading.Thread(target=worker,args=(jid,goals,profile,settings),daemon=True).start();return self.send(200,{'job':jid})
  if path.endswith('/resume'):
   jid=path.split('/')[-2]
   with LOCK:
    j=JOBS.get(jid);p=j.get('process')if j else None
    if not p or p.poll()is not None or j['data']['phase']not in('variations','timeout'):return self.send(409,{'error':'This search cannot be resumed. Start a new search.'})
    j['data']=dict(j['data'],phase='searching',can_resume=False,elapsed=0,batch_variations=0)
    p.stdin.write('resume\n');p.stdin.flush()
   return self.send(200,{'job':jid})
  if path.endswith('/cancel'):
   jid=path.split('/')[-2]
   if jid not in JOBS:return self.send(404,{'error':'Unknown search'})
   j=JOBS[jid];j['cancelled']=True;p=j.get('process');p and p.poll()is None and p.terminate();j['data']['phase']='cancelled';j['data']['can_resume']=False;return self.send(200,{'ok':True})
  return self.send(404,{'error':'Not found'})
 def log_message(self,*args):pass
if __name__=='__main__':
 port=int(sys.argv[1])if len(sys.argv)>1 else 8780
 print(f'Set Builder web preview: http://127.0.0.1:{port}',flush=True)
 try:ThreadingHTTPServer(('127.0.0.1',port),Handler).serve_forever()
 finally:
  for j in JOBS.values():
   p=j.get('process')
   if p and p.poll()is None:p.terminate()
