//! Pure translation helpers. Game strings are read by slot; no game text is stored here.
extern crate alloc;
use alloc::{borrow::Cow, collections::BTreeMap, string::{String,ToString}, vec::Vec};

pub fn hash(text:&str)->u64 { text.bytes().fold(0xcbf29ce484222325,|h,b|(h^u64::from(b)).wrapping_mul(0x100000001b3)) }

pub fn gmd(data:&[u8])->Option<Vec<&str>> {
 if data.len()<41||&data[..4]!=b"GMD\0" {return None;}
 let offset=u32::from_le_bytes(data[36..40].try_into().ok()?)as usize;
 let bytes=data.get(41usize.checked_add(offset)?..)?;
 if bytes.last()!=Some(&0){return None;}
 bytes[..bytes.len()-1].split(|b|*b==0).map(|b|core::str::from_utf8(b).ok()).collect()
}

pub struct Translator { pub names:BTreeMap<u64,String>, pub labels:BTreeMap<String,String> }
impl Translator {
 pub fn new()->Self {Self{names:BTreeMap::new(),labels:BTreeMap::new()}}
 pub fn exact<'a>(&'a self,text:&'a str)->&'a str {
  self.labels.get(text).or_else(||self.names.get(&hash(text))).map(String::as_str).unwrap_or(text)
 }
 // Translate complete tokens/phrases, longest first. Never substitute inside a word.
 pub fn text<'a>(&self,text:&'a str)->Cow<'a,str> {
  let exact=self.exact(text);if exact!=text{return Cow::Owned(exact.to_string());}
  if self.names.is_empty()&&self.labels.is_empty(){return Cow::Borrowed(text);}
  let mut output=String::new();let mut at=0;let mut changed=false;
  while at<text.len(){
   let rest=&text[at..];let mut found=None;
   if at==0||!text[..at].chars().next_back().unwrap().is_alphanumeric(){
    for (end,_) in rest.char_indices().chain(core::iter::once((rest.len(),'\0'))).rev(){
     if end==0||end>256 {continue;}
     if end<rest.len()&&rest[end..].chars().next().unwrap().is_alphanumeric(){continue;}
     let part=&rest[..end];let translated=self.exact(part);
     if part!=translated{found=Some((end,translated));break;}
    }
   }
   if let Some((len,value))=found{output.push_str(value);at+=len;changed=true;}
   else{let ch=rest.chars().next().unwrap();output.push(ch);at+=ch.len_utf8();}
  }
  if changed{Cow::Owned(output)}else{Cow::Borrowed(text)}
 }
 pub fn load_labels(&mut self,input:&str)->Result<(),usize>{
  let mut labels=BTreeMap::new();
  for (i,line) in input.lines().enumerate(){
   if line.is_empty()||line.starts_with('#'){continue;}
   let Some((key,value))=line.split_once('\t')else{return Err(i+1)};
   if key.is_empty()||value.is_empty()||key.len()>512||value.len()>2048{return Err(i+1);}
   if key!=value{labels.insert(key.to_string(),value.replace("\\n","\n"));}
  }
  self.labels=labels;Ok(())
 }
}

pub fn matches(name:&str,query:&str)->bool {
 if query.is_empty(){return true;}
 fn normalized(s:&str)->String{s.chars().filter(|c|c.is_alphanumeric()).flat_map(char::to_lowercase).collect()}
 normalized(name).contains(&normalized(query))
}

pub fn korean(text:&str,map:&[u16])->String {
 text.chars().map(|c|{
  let code=c as usize;
  if (0x7a00..0x7a00+map.len()).contains(&code){char::from_u32(u32::from(map[code-0x7a00])).unwrap_or(c)}else{c}
 }).collect()
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn gmd_bounds_and_utf8(){let mut b=alloc::vec![0;41];b[..4].copy_from_slice(b"GMD\0");b.extend_from_slice("名前\0Description\0".as_bytes());assert_eq!(gmd(&b).unwrap()[0],"名前");b.pop();assert!(gmd(&b).is_none());b[36..40].copy_from_slice(&u32::MAX.to_le_bytes());assert!(gmd(&b).is_none());}
 #[test]fn malformed_gmd_rejected(){
  assert!(gmd(b"GMD").is_none());
  let mut b=alloc::vec![0;41];b[..4].copy_from_slice(b"GMD\0");
  b.extend_from_slice(&[0xff,0]);assert!(gmd(&b).is_none());
 }
 #[test]fn identity_labels_keep_fast_path(){let mut t=Translator::new();t.load_labels("Items\tItems\n").unwrap();assert!(t.labels.is_empty());assert!(matches("anything",""));}
 #[test]fn phrase_translation_and_boundaries(){let mut t=Translator::new();t.names.insert(hash("Iron Ore"),"鉄鉱石".into());t.load_labels("Attack Up (L)\t攻撃力UP【大】\n").unwrap();assert_eq!(t.text("Iron Ore ×2"),"鉄鉱石 ×2");assert_eq!(t.text("Attack Up (L) 20 / 20"),"攻撃力UP【大】 20 / 20");assert_eq!(t.text("Iron Ores"),"Iron Ores");assert_eq!(t.text("Unknown"),"Unknown");}
 #[test]fn invalid_labels_are_atomic(){let mut t=Translator::new();t.load_labels("Items\tアイテム").unwrap();assert!(t.load_labels("broken").is_err());assert_eq!(t.exact("Items"),"アイテム");}
 #[test]fn unicode_search(){assert!(matches("鉄鉱石","鉱石"));assert!(!matches("鉄鉱石","植物"));assert!(matches("Attack Up (L)","attackup"));assert!(matches("Épée","épée"));}
 #[test]fn korean_only_maps_configured_range(){assert_eq!(korean("稀稁日本",&[0xac00,0xac01]),"가각日本");}
}
