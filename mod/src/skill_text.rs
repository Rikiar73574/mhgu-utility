//! Allocation-free wrapping; every line remains a valid slice of the source.
pub fn lines(text:&str,width:f32,measure:impl Fn(char)->f32,mut emit:impl FnMut(usize,&str))->usize{
 let mut count=0;
 for paragraph in text.split('\n'){
  let mut rest=paragraph.trim();
  if rest.is_empty(){emit(count,"");count+=1;continue;}
  while !rest.is_empty(){
   let mut used=0.0;let mut end=0;let mut space=None;
   for (i,ch) in rest.char_indices(){
    let next=used+measure(ch);
    if next>width&&end>0{break;}
    used=next;end=i+ch.len_utf8();if ch.is_whitespace(){space=Some(i);}
   }
   if end<rest.len(){if let Some(i)=space.filter(|&i|i>0){end=i;}}
   emit(count,rest[..end].trim_end());count+=1;rest=rest[end..].trim_start();
  }
 }
 count
}
#[cfg(test)]mod tests{
 use super::*;
 fn wrap(s:&str,w:f32)->Vec<String>{let mut out=Vec::new();lines(s,w,|_|1.0,|_,s|out.push(s.into()));out}
 #[test]fn wraps_words_and_preserves_paragraphs(){assert_eq!(wrap("one two\n\nthree",5.0),["one","two","","three"]);}
 #[test]fn splits_long_words_at_unicode_boundaries(){assert_eq!(wrap("é猫word",2.0),["é猫","wo","rd"]);}
 #[test]fn always_advances_even_for_tiny_width(){assert_eq!(wrap("猫é",0.0),["猫","é"]);assert_eq!(wrap("",5.0),[""]);}
}

/// Decode one generated effect, never the whole catalog or a heap allocation.
pub fn decode<'a>(encoded:&[u8],pairs:&[(u8,u8);256],out:&'a mut [u8])->Option<&'a str>{
 let mut len=0;let mut stack=[0u8;256];let mut pending=0;
 for &byte in encoded{
  stack[pending]=byte;pending+=1;
  while pending>0{
   pending-=1;let token=stack[pending];let (a,b)=pairs[token as usize];
   if (a,b)==(0,0){*out.get_mut(len)?=token;len+=1;}
   else{if pending+2>stack.len(){return None;}stack[pending]=b;stack[pending+1]=a;pending+=2;}
  }
 }
 core::str::from_utf8(&out[..len]).ok()
}
#[cfg(test)]mod decode_tests{
 use super::*;
 #[test]fn expands_nested_pairs_and_preserves_utf8(){let mut pairs=[(0,0);256];pairs[1]=(b'a',b'b');pairs[2]=(1,b'c');let mut out=[0;12];assert_eq!(decode(&[2,0xc3,0xa9],&pairs,&mut out),Some("abcé"));assert_eq!(decode(&[2,2,2,2,2],&pairs,&mut out),None);}
 #[test]fn rejects_corrupt_cycles(){let mut pairs=[(0,0);256];pairs[1]=(1,1);assert_eq!(decode(&[1],&pairs,&mut[0;32]),None);}
}
