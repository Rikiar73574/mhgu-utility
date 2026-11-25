const $ = id => document.getElementById(id);
const esc = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const categories = ['Damage', 'Weapon support', 'Survival & comfort', 'Gathering & rewards'];
const descriptions = ['Attack, affinity and matching element/status bonuses.', 'Skills that suit how the selected weapon works.', 'Defense, health, guarding, evasion and resistance.', 'Gathering, rewards and other general utility.'];
const weapons = ['Great Sword','Long Sword','Sword & Shield','Dual Blades','Hammer','Hunting Horn','Lance','Gunlance','Switch Axe','Charge Blade','Insect Glaive','Light Bowgun','Heavy Bowgun','Bow','Unknown'];
const pieces = ['Head','Chest','Arms','Waist','Legs','Charm','Weapon'];
const key = 'mhgu-builder-preview-v1';
const clone = value => JSON.parse(JSON.stringify(value));
let meta, effects = {}, groups = [], view = 'search', goals = [], data = {builds:[]}, job = null, paused = null, timer;
let chosen = new Set(), detailId = null, categoryToken = 0, resultToken = '';
let local = {saved:[], plans:[], config:null};
try { local = {...local, ...JSON.parse(localStorage.getItem(key) || '{}')}; } catch { $('notice').textContent = 'Stored settings could not be read. Defaults are in use.'; }
if (!Array.isArray(local.saved)) local.saved = [];
if (!Array.isArray(local.plans)) local.plans = [];
let config;

function persist() {
  local.config = config;
  try { localStorage.setItem(key, JSON.stringify(local)); }
  catch { throw Error('Browser storage is unavailable or full. Export your builds before closing this page.'); }
  $('saved-count').textContent = local.saved.length;
}
function notice(message) { $('notice').textContent = message; }
function fail(error) { $('error').textContent = error.message || String(error); }
async function api(path, body) {
  const response = await fetch(path, body === undefined ? {} : {method:'POST', headers:{'Content-Type':'application/json'}, body:JSON.stringify(body)});
  const value = await response.json();
  if (!response.ok) throw Error(value.error || 'The preview request failed.');
  return value;
}
function download(name, value) {
  const url = URL.createObjectURL(new Blob([JSON.stringify(value, null, 2)], {type:'application/json'}));
  const a = document.createElement('a'); a.href = url; a.download = name; a.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
}
function show(next) {
  view = next;
  for (const name of ['utility','search','settings','saved','detail','comparison']) $(name).hidden = name !== view;
  $('tabs').hidden = view === 'utility';
  $('back').hidden = view === 'utility';
  $('back').textContent = view === 'search' ? 'Back to Utility' : view === 'detail' ? 'Back to saved builds' : 'Back to search';
  $('heading').textContent = view === 'utility' ? 'Utility' : 'Set Builder';
  document.querySelectorAll('#tabs button').forEach(b => b.setAttribute('aria-current', b.dataset.view === view ? 'page' : 'false'));
  if (view === 'saved') renderSaved();
  if (view === 'settings') renderSettings();
  window.scrollTo({top:0});
}
function back() { show(view === 'search' ? 'utility' : view === 'detail' ? 'saved' : 'search'); }
document.addEventListener('keydown', event => {
  if (event.key !== 'Escape') return;
  if (event.target.matches('input,select,textarea')) { event.target.blur(); return; }
  if (view !== 'utility') { event.preventDefault(); back(); }
});
$('back').onclick = back;
document.addEventListener('click', event => { const b = event.target.closest('[data-view]'); if (b) show(b.dataset.view); });

function caption(name) {
  const summary = effects.SKILL_SUMMARIES?.[name] || effects.SKILL_EFFECTS?.[name] || '';
  const full = effects.SKILL_FULL?.[name] || summary;
  return `<span class="effect">${esc(summary)}</span>${full !== summary ? `<details><summary class="small">Full effect</summary><span class="effect">${esc(full)}</span></details>` : ''}`;
}
function selected() {
  $('goals').innerHTML = goals.map(id => {
    const skill = meta.skills.find(s => s.id === id);
    return `<div class="goal"><div>${esc(skill.name)}${caption(skill.name)}</div><button data-remove="${id}" aria-label="Remove ${esc(skill.name)}" ${job?'disabled':''}>×</button></div>`;
  }).join('') || '<p class="muted">No required skills. Recommend builds using your weapon and skill priorities.</p>';
  $('run').disabled = !!job;
  $('stop').hidden = !job;
  $('resume').hidden = !paused || !!job || JSON.stringify(goals) !== JSON.stringify(data.goals) || JSON.stringify(config) !== JSON.stringify({weapon_profile:data.weapon_profile, settings:data.settings});
  suggestions();
}
function suggestions() {
  const query = $('skill-search').value.trim().toLowerCase();
  $('suggestions').innerHTML = (query ? meta.skills.filter(s => s.name.toLowerCase().includes(query)) : []).slice(0,12).map(s => `<button data-add="${s.id}" ${job?'disabled':''}>${esc(s.name)} <span class="small">${s.points} pts</span>${caption(s.name)}</button>`).join('');
}
$('skill-search').oninput = suggestions;
$('skill-search').onkeydown = event => { if (event.key === 'Enter') { const first = $('suggestions').querySelector('button'); if (first) first.click(); else if (!job) run(); } };
$('suggestions').onclick = event => {
  const button = event.target.closest('[data-add]'); if (!button || job) return;
  const skill = meta.skills.find(s => s.id === Number(button.dataset.add));
  const at = goals.findIndex(id => meta.skills.find(s => s.id === id).tree === skill.tree);
  if (at >= 0) goals[at] = skill.id; else if (goals.length < 6) goals.push(skill.id);
  $('skill-search').value = ''; selected(); $('skill-search').focus();
};
$('goals').onclick = event => { const b = event.target.closest('[data-remove]'); if (b && !job) { goals = goals.filter(id => id !== Number(b.dataset.remove)); selected(); } };

function context() {
  const p = config.weapon_profile, s = config.settings;
  $('query-context').innerHTML = `<p>${esc(weapons[p.weapon])} / ${p.slots} weapon slots</p><p>${meta.profile.charms} captured owned charms</p><p>${s.village === null ? 'Captured native progression' : `Planning: Village ${s.village} / Hub ${s.hub}`}</p><p>${s.owned_only ? 'Owned armor only' : 'All available armor'}</p><p class="small muted">${s.order.map(i=>esc(categories[i])).join(' / ')}</p>`;
}
function options(id, values, current) { $(id).innerHTML = values.map(([value,label]) => `<option value="${value}" ${value===current?'selected':''}>${esc(label)}</option>`).join(''); }
function renderSettings() {
  const p = config.weapon_profile, s = config.settings;
  options('weapon-type', weapons.map((n,i)=>[i,n]), p.weapon);
  [...$('weapon-type').options].forEach(o => { if (+o.value !== 14 && (+o.value >= 11) !== Boolean(meta.profile.hunter)) o.disabled = true; });
  options('weapon-element', [[0,'None / unknown'],[27,'Fire'],[28,'Water'],[29,'Thunder'],[30,'Ice'],[31,'Dragon']], p.element);
  options('weapon-status', [[0,'None / unknown'],[1,'Poison'],[2,'Paralysis'],[3,'Sleep'],[92,'Blast']], p.status);
  options('weapon-slots', [0,1,2,3].map(n=>[n,String(n)]), p.slots);
  options('village-stars', Array.from({length:11},(_,n)=>[n,n?`${n} stars`:'No Village quests']), s.village ?? 1);
  options('hub-stars', Array.from({length:12},(_,n)=>[n,n>7?`G ${n-7} stars`:n?`${n} stars`:'No Hub quests']), s.hub ?? Math.min(meta.profile.hr,7));
  $('override-stars').checked = s.village !== null;
  $('village-stars').disabled = $('hub-stars').disabled = s.village === null;
  $('progression-note').textContent = s.village === null ? 'Automatic: uses the captured native unlock checks. Village stars were not recorded separately in this snapshot.' : 'Planning scenario: selected star limits change the candidate material sources. Recipe unlocks remain unverified.';
  $('owned-only').checked = s.owned_only;
  renderCategories();
}
async function refreshCategories() {
  const token = ++categoryToken;
  const value = await api('/api/categories?'+new URLSearchParams(config.weapon_profile));
  if (token !== categoryToken) return;
  const trees = new Map();
  for (const skill of value.skills) {
    if (!trees.has(skill.tree)) trees.set(skill.tree,{tree:skill.tree,name:skill.tree_name,category:skill.category,weight:skill.weight,activations:[]});
    const group=trees.get(skill.tree);if(group.category!==skill.category){group.mixed=true;group.category=Math.max(group.category,skill.category);}group.activations.push(skill.name+' ('+(skill.category?categories[4-skill.category]:'not prioritized')+')');
  }
  groups = [...trees.values()]; renderCategories();
}
function orderedSkills(category) {
  const saved = config.settings.skills[String(category)] || [];
  return groups.filter(g => g.category === 4-category).sort((a,b) => {
    if (saved.length) { const x=saved.indexOf(a.tree),y=saved.indexOf(b.tree); return (x<0?999:x)-(y<0?999:y) || a.name.localeCompare(b.name); }
    return b.weight-a.weight || a.name.localeCompare(b.name);
  });
}
const expanded = new Set();
function renderCategories() {
  $('categories').innerHTML = config.settings.order.map((category, position) => {
    const skills = orderedSkills(category);
    return `<div class="priority-category"><div class="category-heading"><div><button class="category-toggle" data-expand="${category}" aria-expanded="${expanded.has(category)}">${expanded.has(category)?'▾':'▸'} ${position+1}. ${esc(categories[category])} <span class="small">(${skills.length})</span></button><p class="category-description">${esc(descriptions[category])}</p></div><div class="move-controls"><button data-category="${category}" data-step="-1" aria-label="Move ${esc(categories[category])} up" ${position===0?'disabled':''}>↑</button><button data-category="${category}" data-step="1" aria-label="Move ${esc(categories[category])} down" ${position===3?'disabled':''}>↓</button></div></div><div class="skill-orders" ${expanded.has(category)?'':'hidden'}>${skills.map((s,i)=>`<div class="skill-order"><span>${esc(s.name)}<small>${s.activations.map(esc).join(', ')}</small></span><div class="move-controls"><button data-tree="${s.tree}" data-group="${category}" data-step="-1" aria-label="Move ${esc(s.name)} up" ${i===0?'disabled':''}>↑</button><button data-tree="${s.tree}" data-group="${category}" data-step="1" aria-label="Move ${esc(s.name)} down" ${i===skills.length-1?'disabled':''}>↓</button></div></div>`).join('')}</div></div>`;
  }).join('')+`<p class="small muted">${groups.filter(g=>g.category===0).length} skill trees do not receive a recommendation for this weapon profile. They can still be explicitly requested.</p>`;
}
$('categories').onclick = event => {
  const expand = event.target.closest('[data-expand]');
  if (expand) { const id=+expand.dataset.expand; expanded.has(id)?expanded.delete(id):expanded.add(id); renderCategories(); return; }
  const button = event.target.closest('[data-step]'); if (!button) return;
  const step=+button.dataset.step;
  if (button.dataset.category !== undefined) {
    const order=config.settings.order, at=order.indexOf(+button.dataset.category);
    [order[at],order[at+step]]=[order[at+step],order[at]];
  } else {
    const category=+button.dataset.group, order=orderedSkills(category).map(s=>s.tree), at=order.indexOf(+button.dataset.tree);
    [order[at],order[at+step]]=[order[at+step],order[at]]; config.settings.skills[String(category)]=order;
  }
  try { persist(); context(); selected(); renderCategories(); notice('Priority order saved. Start a new search to apply it.'); } catch(e) { fail(e); }
};
$('settings').addEventListener('change', async event => {
  const p=config.weapon_profile,s=config.settings;
  p.weapon=+$('weapon-type').value; p.element=+$('weapon-element').value; p.status=+$('weapon-status').value; p.slots=+$('weapon-slots').value;
  const override=$('override-stars').checked;
  s.village=override?+$('village-stars').value:null; s.hub=override?+$('hub-stars').value:null; s.owned_only=$('owned-only').checked;
  try { persist(); context(); selected(); if (event.target.id.startsWith('weapon-')) await refreshCategories(); renderSettings(); notice('Settings saved. Start a new search to apply them.'); } catch(e) { fail(e); }
});
$('reset-settings').onclick = async () => { config=defaults(); try { persist(); await refreshCategories(); renderSettings(); context(); selected(); notice('Settings reset to the captured character profile.'); } catch(e) { fail(e); } };

function sockets(total, used) { return `<span class="sockets" aria-label="${used} of ${total} slots used">${Array.from({length:total},(_,i)=>`<i class="socket ${i<used?'used':''}"></i>`).join('')||'<span class="muted small">None</span>'}</span>`; }
function charmName(build) { return build.charm ? build.charm.skills.map(s=>`${s.name} ${s.points>0?'+':''}${s.points}`).join(', ') : 'No charm required'; }
function card(build, source, savedId=null) {
  const stale=!!savedId&&(source.snapshot!==meta.snapshot_hash||source.source!==meta.source_hash);
  const p=source.weapon_profile, planning=source.settings?.village!==null && source.settings?.village!==undefined;
  const active=build.skills.filter(s=>s.active || (s.points<0 && s.activation));
  const inactive=build.skills.filter(s=>!active.includes(s));
  const rows=[...build.armor.map(a=>({name:a.name,ownership:a.owned?'Owned':'Missing',owned:a.owned})),{name:charmName(build),ownership:build.charm?'Owned':'Not needed',owned:true},{name:weapons[p.weapon]+' profile',ownership:'Preview setting',owned:true}];
  const reference=savedId?`saved:${savedId}`:`result:${build.rank}`;
  return `<article class="build"><div class="build-head"><div><h3>${savedId?esc(local.saved.find(s=>s.id===savedId)?.name):'Build '+build.rank}</h3><p class="small muted">${build.active_count} active skills / priority ${(source.settings?.order||[0,1,2,3]).map(c=>categories[c]).map(esc).join(' / ')}</p></div><span class="badge ${!build.verified||planning||stale?'provisional':''}">${stale?'Saved snapshot: revalidation needed':planning?'Planning scenario: recipe unlocks unverified':build.verified?'Native availability verified':'Unlocks unconfirmed'}</span></div><div class="build-grid"><section class="equipment"><h4>Pieces, slots & jewels</h4>${rows.map((a,i)=>`<div class="gear-row"><span class="gear-label">${pieces[i]}</span><div class="gear-name">${esc(a.name)}<small class="${a.owned?'owned':'missing'}">${esc(a.ownership)}${i===5&&build.charm?' / Equipment box #'+build.charm.box_index:''}</small></div>${sockets(build.capacity[i],build.used[i])}<ul class="jewel-list">${build.jewels.filter(j=>j.gear===i).map(j=>`<li>${esc(j.name)} / ${j.slots} socket${j.slots>1?'s':''}</li>`).join('')||`<li class="muted">${build.capacity[i]?'No jewels placed':'No sockets'}</li>`}</ul></div>`).join('')}</section><section class="skills"><h4>Skills from this build</h4>${active.map(s=>`<div class="activation ${s.requested?'requested':''} ${s.points<0?'negative':''}"><div class="activation-title"><span>${esc(s.activation)}</span><span>${s.points}/${s.threshold}</span></div><span class="category-caption">${s.category?esc(categories[4-s.category]):'Not prioritized for this weapon'}${s.requested?' / Required':''}</span>${caption(s.activation)}</div>`).join('')}<h4>Other skill points</h4><div class="points">${inactive.map(s=>`<span class="point ${s.points<0?'bad':''}">${esc(s.name)} ${s.points>0?'+':''}${s.points} / ${s.points<0?-Math.abs(s.threshold):s.threshold}</span>`).join('')||'<span class="muted small">No additional skill points.</span>'}</div></section></div><div class="build-summary"><span>Base defense ${build.defense}</span><span>${build.used.reduce((a,b)=>a+b,0)} / ${build.capacity.reduce((a,b)=>a+b,0)} sockets used</span><span>${build.armor.filter(a=>!a.owned).length} armor pieces missing</span></div><div class="build-actions">${savedId?'':`<input class="save-name" type="text" maxlength="80" aria-label="Name for build ${build.rank}" placeholder="Name this build"><button data-save="${build.rank}">Save build</button><label class="check"><input type="checkbox" data-compare="${build.rank}" ${chosen.has(build.rank)?'checked':''}>Compare</label>`}<button data-plan="${reference}">Plan missing armor & jewels</button></div></article>`;
}
function renderResults(force=false) {
  const token=JSON.stringify([data.builds,$('result-filter').value,$('verified').checked,[...chosen]]); if (!force && token===resultToken) return; resultToken=token;
  const query=$('result-filter').value.toLowerCase().trim();
  const builds=data.builds.filter(b=>(!$('verified').checked || (b.verified && data.settings?.village==null))&&(!query || JSON.stringify(b).toLowerCase().includes(query)));
  $('results').innerHTML=builds.map(b=>card(b,data)).join('')||'<p class="empty">'+(job?'Searching armor, charms and jewels…':data.builds.length?'No builds match these display filters.':'Start a search; required skills are optional.')+'</p>';
  $('compare').hidden=chosen.size<2;
}
$('result-filter').oninput=()=>renderResults(true); $('verified').onchange=()=>renderResults(true);
async function run() {
  if (job) return;
  try {
    if (paused) { await api('/api/jobs/'+paused+'/cancel',{}); paused=null; }
    $('error').textContent=''; notice(''); chosen.clear(); data={builds:[]}; renderResults(true);
    const reply=await api('/api/search',{goals,...clone(config)}); job=reply.job; selected(); renderResults(true); poll();
  } catch(e) { fail(e); }
}
async function poll() {
  if (!job) return;
  try {
    data=await api('/api/jobs/'+job); data.builds ||= [];
    const phase=data.phase, terminal=['timeout','variations','complete','cancelled','error'].includes(phase);
    $('progress').textContent=`${data.builds.length} variations / ${Number(data.visited||0).toLocaleString()} search steps / ${Number(data.elapsed||0).toFixed(1)} seconds`;
    $('search-status').textContent=phase==='timeout'?'60-second checkpoint reached. Continue to keep searching.':phase==='variations'?'Five new skill variations found. Continue to keep searching.':phase==='complete'?'Search exhausted all candidates.':phase==='cancelled'?'Search stopped.':phase==='error'?'Search failed.':'Searching armor, owned charms and jewels…';
    renderResults();
    if (terminal) { paused=data.can_resume?job:null; job=null; selected(); if (data.error) throw Error(data.error); }
    else timer=setTimeout(poll,600);
  } catch(e) { job=null; selected(); fail(e); }
}
$('run').onclick=run;
$('resume').onclick=async()=>{try{const id=paused;if(!id||job)return;await api('/api/jobs/'+id+'/resume',{});job=id;paused=null;selected();poll();}catch(e){fail(e)}};
$('stop').onclick=async()=>{try{if(job)await api('/api/jobs/'+job+'/cancel',{});}catch(e){fail(e)}};
$('export-results').onclick=()=>download('mhgu-build-results.json',{profile:meta.profile,...data});

document.addEventListener('change',event=>{
  if (!event.target.matches('[data-compare]')) return;
  const rank=+event.target.dataset.compare;
  if (event.target.checked) { if(chosen.size>=3){event.target.checked=false;notice('Compare up to three builds at once.');return;} chosen.add(rank); } else chosen.delete(rank);
  $('compare').hidden=chosen.size<2;
});
$('compare').onclick=()=>{
  const builds=data.builds.filter(b=>chosen.has(b.rank));
  const rows=[['Skills',b=>b.skills.filter(s=>s.active).map(s=>esc(s.activation)+' '+s.points+'/'+s.threshold).join('<br>')],...pieces.slice(0,5).map((n,i)=>[n,b=>esc(b.armor[i].name)+(b.armor[i].owned?' (Owned)':' (Missing)')]),['Charm',b=>esc(charmName(b))],['Jewels',b=>b.jewels.map(j=>esc(j.name)+' / '+pieces[j.gear]).join('<br>')],['Sockets',b=>b.used.reduce((a,n)=>a+n,0)+' / '+b.capacity.reduce((a,n)=>a+n,0)],['Base defense',b=>b.defense]];
  $('comparison-content').innerHTML='<div class="comparison-scroll"><table class="comparison-table"><thead><tr><th>Loadout</th>'+builds.map(b=>'<th>Build '+b.rank+'</th>').join('')+'</tr></thead><tbody>'+rows.map(([n,f])=>'<tr><th>'+n+'</th>'+builds.map(b=>'<td>'+f(b)+'</td>').join('')+'</tr>').join('')+'</tbody></table></div>';show('comparison');
};
document.addEventListener('click',event=>{
  const button=event.target.closest('[data-save],[data-plan]'); if(!button)return;
  try {
    if(button.dataset.save){const build=data.builds.find(b=>b.rank===+button.dataset.save);if(!build)return;const name=button.parentElement.querySelector('input[type=text]').value.trim()||build.skills.filter(s=>s.active).map(s=>s.activation).join(' + ');local.saved.push({id:crypto.randomUUID(),name:name.slice(0,80),created:Date.now(),snapshot:meta.snapshot_hash,source:meta.source_hash,goals:clone(data.goals),build:clone(build),weapon_profile:clone(data.weapon_profile),settings:clone(data.settings)});persist();notice('Build saved. Open Saved builds to find it.');}
    else{const[type,id]=button.dataset.plan.split(':');const item=type==='saved'?local.saved.find(s=>s.id===id):{build:data.builds.find(b=>b.rank===+id)};if(!item?.build)return;local.plans.push({id:crypto.randomUUID(),created:Date.now(),armor:item.build.armor.map((a,slot)=>({...a,slot})).filter(a=>!a.owned),jewels:clone(item.build.jewels),note:'Armor ownership from captured character. Jewel ownership unknown. This is not the in-game wishlist.'});persist();notice('Wishlist plan saved locally. It has not changed your in-game wishlist.');}
  } catch(e){fail(e)}
});
function renderSaved() {
  $('saved-count').textContent=local.saved.length;
  $('saved-list').innerHTML=local.saved.length?'<div class="saved-grid">'+local.saved.map(item=>`<article class="saved-card"><label for="name-${item.id}">Build name<input id="name-${item.id}" type="text" maxlength="80" value="${esc(item.name)}" data-rename="${item.id}"></label><p>${item.build.skills.filter(s=>s.active).map(s=>esc(s.activation)).join(' / ')}</p><p class="small muted">${new Date(item.created).toLocaleString()} / ${esc(weapons[item.weapon_profile.weapon])}</p><p class="small ${item.snapshot===meta.snapshot_hash&&item.source===meta.source_hash?'owned':'missing'}">${item.snapshot===meta.snapshot_hash&&item.source===meta.source_hash?'Same captured character and search version':'Needs revalidation against the current character/search version'}</p><div class="actions"><button data-rename-build="${item.id}">Rename</button><button data-open="${item.id}">Open build</button><button data-research="${item.id}">Use search settings</button><button data-delete="${item.id}">Delete</button></div></article>`).join('')+'</div>':'<p class="empty">No builds saved yet. Choose Save build on a search result.</p>';
  $('wishlist-plans').innerHTML=local.plans.map(p=>`<p>${p.armor.length} missing armor pieces / ${p.jewels.length} planned jewel placements <button data-export-plan="${p.id}">Export plan</button></p>`).join('')||'<p class="small muted">No plans saved.</p>';
}
$('saved').onchange=event=>{const id=event.target.dataset.rename;if(!id)return;try{local.saved.find(s=>s.id===id).name=event.target.value.trim().slice(0,80)||'Unnamed build';persist();notice('Build renamed.');}catch(e){fail(e)}};
$('saved').onclick=async event=>{
  const button=event.target.closest('[data-open],[data-delete],[data-research],[data-export-plan],[data-rename-build]');if(!button)return;
  try{
    if(button.dataset.renameBuild){const item=local.saved.find(s=>s.id===button.dataset.renameBuild);item.name=button.closest('.saved-card').querySelector('input').value.trim().slice(0,80)||'Unnamed build';persist();notice('Build renamed.');return;}
    if(button.dataset.exportPlan){download('mhgu-wishlist-plan.json',local.plans.find(p=>p.id===button.dataset.exportPlan));return;}
    const id=button.dataset.open||button.dataset.delete||button.dataset.research,item=local.saved.find(s=>s.id===id);if(!item)return;
    if(button.dataset.delete){local.saved=local.saved.filter(s=>s.id!==id);persist();renderSaved();notice('Saved build deleted.');}
    else if(button.dataset.research){config={weapon_profile:clone(item.weapon_profile),settings:clone(item.settings)};goals=item.goals||item.build.skills.filter(s=>s.requested).map(s=>meta.skills.find(a=>a.tree===s.tree&&a.points===s.threshold)?.id).filter(v=>v!==undefined);persist();await refreshCategories();context();selected();show('search');notice('Saved search settings loaded. Start a new search to revalidate.');}
    else{detailId=id;$('saved-detail').innerHTML=card(item.build,item,id);show('detail');}
  }catch(e){fail(e)}
};
$('export-saved').onclick=()=>download('mhgu-saved-builds.json',{version:1,builds:local.saved,plans:local.plans});
function defaults(){return {weapon_profile:{weapon:14,element:0,status:0,slots:meta.profile.weapon_slots},settings:{order:[0,1,2,3],skills:{},village:null,hub:null,owned_only:false}};}

async function boot(){
  try{
    [meta,effects]=await Promise.all([api('/api/meta'),api('/api/effects')]);config=local.config||defaults();
    const latest=await api('/api/latest');
    if(latest.builds?.length){data=latest;goals=latest.goals;paused=latest.can_resume?latest.job:null;if(!local.config)config={weapon_profile:latest.weapon_profile,settings:latest.settings};}
    await refreshCategories();context();selected();renderResults(true);renderSaved();
    $('capture').textContent=`HR ${meta.profile.hr} / ${meta.profile.hunter===0?'Blademaster':'Gunner'} / ${meta.profile.gender===0?'Male':'Female'}. Charms captured ${new Date(meta.profile.charms_captured_at*1000).toLocaleDateString()}.`;
    $('provenance').textContent=`Search ${meta.search_version} / source ${meta.source_hash.slice(0,8)} / ${meta.armor_count} captured armor candidates / ${meta.jewel_count} decorations.`;
    $('saved-count').textContent=local.saved.length;show('search');
    if(data.builds.length){$('progress').textContent=`${data.builds.length} result variations / previous checkpoint`;$('search-status').textContent=paused?'Previous search checkpoint. Continue to keep searching.':'Previous result snapshot. Start a new search to recompute.';}
  }catch(e){fail(e)}
}
boot();
