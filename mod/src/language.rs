//! Optional runtime localization; read-only RomFS files and safe English fallback.
extern crate alloc;
use alloc::{collections::{BTreeMap,BTreeSet},ffi::CString,format,string::{String,ToString},vec::Vec};
use core::ffi::{c_char,c_void};
use crate::{Local,localization::{self,Translator}};
mod slots {include!(concat!(env!("OUT_DIR"),"/name_slots.rs"));}
#[derive(Clone,Copy)]#[repr(C)]struct Handle(u32);
#[repr(C)]struct LanguageCode{bytes:[u8;8]}
unsafe extern "C" {
 #[link_name="_ZN2nn2oe18GetDesiredLanguageEv"]fn desired_language()->LanguageCode;
 #[link_name="_ZN2nn2fs8OpenFileEPNS0_10FileHandleEPKci"]fn open(h:*mut Handle,p:*const c_char,m:i32)->u32;
 #[link_name="_ZN2nn2fs11GetFileSizeEPxNS0_10FileHandleE"]fn size(n:*mut i64,h:Handle)->u32;
 #[link_name="_ZN2nn2fs8ReadFileENS0_10FileHandleExPvj"]fn read(h:Handle,o:i64,p:*mut c_void,n:usize)->u32;
 #[link_name="_ZN2nn2fs9CloseFileENS0_10FileHandleE"]fn close(h:Handle);
}
pub fn file(relative:&str,max:usize)->Option<Vec<u8>>{
 let path=CString::new(format!("{}:/{}",forge::fs::romfs::mount_point(),relative)).ok()?;
 let mut h=Handle(0);if unsafe{open(&mut h,path.as_ptr(),1)}!=0{return None;}
 let mut n=0i64;let result=unsafe{size(&mut n,h)};
 let bytes=if result==0&&n>0&&n<=max as i64{
  let mut bytes=Vec::new();if bytes.try_reserve_exact(n as usize).is_err(){unsafe{close(h)};return None;}bytes.resize(n as usize,0);
  if unsafe{read(h,0,bytes.as_mut_ptr().cast(),bytes.len())}==0{Some(bytes)}else{None}
 }else{None};unsafe{close(h)};bytes
}
struct State{translator:Translator,cache:BTreeMap<String,String>,locale:String,cjk:bool,fonts:BTreeMap<String,Vec<u8>>}
static STATE:Local<Option<State>>=Local(core::cell::UnsafeCell::new(None));
// Called once during the ImGui initialization callback, before any UI text.
pub fn init(){unsafe{
 if (*STATE.0.get()).is_some(){return;}
 let config=file("nativeNX/utility/language.ini",4096).and_then(|b|String::from_utf8(b).ok()).unwrap_or_default();
 let get=|key:&str|config.lines().filter_map(|l|l.split_once('=')).find(|(k,_)|k.trim()==key).map(|(_,v)|v.trim());
 let selected=get("language").unwrap_or("auto");
 let locale=if ["eng","jpn","kor","fre","ger","ita","spa"].contains(&selected){selected}else{
  let code=desired_language().bytes;
  match &code[..2]{b"ja"=>"jpn",b"ko"=>"kor",b"fr"=>"fre",b"de"=>"ger",b"it"=>"ita",b"es"=>"spa",_=>{
   if forge::fs::file_exists(&format!("{}:/nativeNX/eng/table/itemData_eng.gmd",forge::fs::romfs::mount_point())){"eng"}else{"jpn"}
  }}
 }.to_string();
 let korean=locale=="kor"&&get("korean_patch")==Some("true");
 let table_locale=if locale=="kor"{"jpn"}else{&locale};
 let mut translator=Translator::new();let mut ambiguous=BTreeSet::new();let mut loaded=0;
 let mapping=file("nativeNX/utility/name_slots.bin",1024*1024).filter(|b|b.len()>=8&&&b[..8]==b"UNAM\x01\0\0\0"&&(b.len()-8)%11==0).unwrap_or_default();
 for (table,name) in slots::TABLES.iter().enumerate(){
  let bytes=file(&format!("nativeNX/{table_locale}/table/{name}_{table_locale}.gmd"),2*1024*1024)
   .or_else(||file(&format!("nativeNX/table/{name}_{table_locale}.gmd"),2*1024*1024));
  let Some(bytes)=bytes else{continue;};let Some(rows)=localization::gmd(&bytes)else{continue;};loaded+=1;
  for record in mapping.get(8..).unwrap_or(&[]).chunks_exact(11).filter(|r|r[8]as usize==table){
   let key=u64::from_le_bytes(record[..8].try_into().unwrap());let index=u16::from_le_bytes(record[9..11].try_into().unwrap());let Some(value)=rows.get(index as usize).filter(|v|!v.is_empty())else{continue;};
   let value=if korean{localization::korean(value,slots::HANGUL)}else{value.to_string()};
   if localization::hash(&value)==key{continue;}
   if translator.names.get(&key).is_some_and(|old|old!=&value){ambiguous.insert(key);}
   translator.names.insert(key,value);
  }
 }
 for key in ambiguous{translator.names.remove(&key);}
 if let Some(bytes)=file(&format!("nativeNX/utility/locales/{locale}.tsv"),512*1024){
  if let Ok(text)=core::str::from_utf8(&bytes){if let Err(line)=translator.load_labels(text){forge::log::prelude::warn!("Utility translation rejected at line {line}");}}
 }
 let cjk=locale=="jpn"||locale=="kor";
 forge::log::prelude::info!("Utility language {locale}: {loaded} native name tables, {} names, {} UI labels",translator.names.len(),translator.labels.len());
 *STATE.0.get()=Some(State{translator,cache:BTreeMap::new(),locale,cjk,fonts:BTreeMap::new()});
}}
pub fn font_data(native:bool)->Option<(*mut u8,usize)>{unsafe{
 let state=(*STATE.0.get()).as_mut()?;if !native&&!state.cjk{return None;}
 let name=if native{"field-guide"}else{&state.locale}.to_string();
 if !state.fonts.contains_key(&name){
  let data=file(&format!("nativeNX/utility/fonts/{name}.ttf"),24*1024*1024)?;
  state.fonts.insert(name.clone(),data);
 }
 let bytes=state.fonts.get_mut(&name)?;Some((bytes.as_mut_ptr(),bytes.len()))
}}
pub fn translated<R>(text:&str,draw:impl FnOnce(&str)->R)->R{
 let value=unsafe{(*STATE.0.get()).as_mut().and_then(|state|{
  if state.translator.names.is_empty()&&state.translator.labels.is_empty(){return None;}
  if let Some(value)=state.cache.get(text){return Some(value.clone());}
  let value=state.translator.text(text);
  let value=value.into_owned();if state.cache.len()<2048{state.cache.insert(text.to_string(),value.clone());}Some(value)
 })};
 draw(value.as_deref().unwrap_or(text))
}

pub fn matches(text:&str,query:&str)->bool{
 localization::matches(text,query)||translated(text,|value|localization::matches(value,query))
}
