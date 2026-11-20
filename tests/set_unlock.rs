#[path="../mod/src/set_unlock.rs"]mod set_unlock;
use set_unlock::*;use std::collections::BTreeMap;
#[derive(Default)]struct Fake{words:BTreeMap<u32,u32>,bytes:BTreeMap<u32,u8>}
impl Memory for Fake{fn word(&self,a:u32)->Option<u32>{self.words.get(&a).copied()}fn byte(&self,a:u32)->Option<u8>{self.bytes.get(&a).copied()}}
const SAVE:u32=0x08000000;const PROGRESS:u32=0x09000000;
fn setup()->Fake{let mut f=Fake::default();f.words.insert(0x1897f78,SAVE);f.words.insert(0x1884c58,PROGRESS);f}
#[test]fn progression_is_not_rank_or_rarity(){let mut f=setup();for g in 0..4{assert_eq!(gate(&f,0,g),Some(true));}f.words.insert(0x162ea7c+20*4,387);f.bytes.insert(PROGRESS+0x500+387/8,0);assert_eq!(gate(&f,0,20),Some(false));f.bytes.insert(PROGRESS+0x500+387/8,1<<(387&7));assert_eq!(gate(&f,0,20),Some(true));assert_eq!(gate(&f,0,112),None);}
#[test]fn special_unlocks_read_exact_save_bits(){let mut f=setup();f.bytes.insert(SAVE+0x3614+27/8,1<<(27&7));assert_eq!(gate(&f,0,89),Some(true));assert_eq!(gate(&f,0,88),Some(false));assert_eq!(gate(&f,0,105),None);}
#[test]fn missing_memory_never_means_locked_or_available(){let f=Fake::default();assert_eq!(gate(&f,0,20),None);assert_eq!(recipe_available(Some(true),&[1,2],|i|if i==1{Some(true)}else{None}),None);}
#[test]fn sources_ignore_quantities_and_historical_key_ownership(){assert_eq!(recipe_available(Some(true),&[1,2],|_|Some(true)),Some(true));assert_eq!(recipe_available(Some(false),&[1],|_|Some(true)),Some(false));assert_eq!(recipe_available(Some(true),&[1,2],|i|Some(i==1)),Some(false));}
#[test]fn alternative_recipes_keep_proven_positive_and_unknown(){assert_eq!(alternatives([None,Some(true)]),Some(true));assert_eq!(alternatives([Some(false),None]),None);assert_eq!(alternatives([Some(false),Some(false)]),Some(false));assert_eq!(alternatives([]),None);}
#[test]fn special_quest_gate_is_an_or_and_missing_metadata_is_unknown(){let mut f=setup();let manager=0x0a000000;let resource=0x0b000000;let rows=0x0c000000;f.words.extend([(0x188ad90,manager),(manager+0xb4,resource),(resource+0x64,6),(resource+0x6c,rows)]);for(i,q)in [0x3ed,0x2c89,0x2cc2,0x2cc3,0x2cc4].into_iter().enumerate(){f.words.insert(rows+(i as u32+1)*12+4,q);}f.bytes.insert(SAVE+0x42c,0);assert_eq!(gate(&f,0,110),Some(false));f.bytes.insert(SAVE+0x42c,1<<4);assert_eq!(gate(&f,0,110),Some(true));f.words.remove(&(rows+5*12+4));assert_eq!(gate(&f,0,110),Some(true));f.bytes.insert(SAVE+0x42c,0);assert_eq!(gate(&f,0,110),None);}
#[test]fn monster_sources_keep_low_high_g_and_actual_monster_access_separate(){
 // Proven sample ingredients: Rathalos Scale/Scale+/Shard are monster1 at
 // LR/HR/GR; Neset's Purple Cocoon is monster66 at GR. No rarity shortcut.
 assert!(source_matches(0,15,&[1],0,0,1,1));assert!(!source_matches(0,15,&[1],0,1,1,1));assert!(!source_matches(0,15,&[1],0,2,1,1));
 assert!(source_matches(1,15,&[1],0,1,1,1));assert!(!source_matches(1,15,&[1],0,2,1,1));
 assert!(source_matches(2,15,&[1],0,2,1,1));assert!(!source_matches(2,15,&[1],0,2,66,1));assert!(source_matches(2,15,&[66],808,2,66,1));
 assert!(source_matches(2,15,&[],808,2,808,2));assert!(!source_matches(2,15,&[],808,2,1278,2));assert!(!source_matches(2,15,&[1],0,2,257,1));
}
#[test]fn known_locked_ingredient_wins_even_after_unknown(){assert_eq!(recipe_available(None,&[1,2],|i|if i==1{None}else{Some(false)}),Some(false));}
#[test]fn acquisition_history_overrides_missing_or_inaccessible_quest_sources(){assert_eq!(ingredient_available(Some(false),Some(true)),Some(true));assert_eq!(ingredient_available(None,Some(true)),Some(true));assert_eq!(ingredient_available(Some(true),None),Some(true));assert_eq!(recipe_available(Some(false),&[1],|_|ingredient_available(Some(false),Some(true))),Some(false));}
#[test]fn no_accessible_modeled_source_is_unknown_without_complete_source_proof(){assert_eq!(ingredient_available(Some(false),Some(false)),None);assert_eq!(ingredient_available(Some(false),None),None);assert_eq!(ingredient_available(None,Some(false)),None);assert_eq!(recipe_available(Some(true),&[1],|_|ingredient_available(Some(false),Some(false))),None);assert_eq!(recipe_available(Some(false),&[1],|_|ingredient_available(Some(false),Some(false))),Some(false));}
