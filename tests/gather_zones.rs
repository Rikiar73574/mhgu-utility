mod catalog{pub static ENTRIES:[u8;4]=[0;4];}
mod world{
 pub struct Gather{pub loc:u8,pub rank:u8,pub zone:u8,pub supply:bool,pub entry:u16}
 pub static GATHER:&[Gather]=&[
 Gather{loc:1,rank:0,zone:2,supply:false,entry:1},Gather{loc:1,rank:0,zone:5,supply:false,entry:1},
 Gather{loc:1,rank:1,zone:3,supply:false,entry:1},Gather{loc:2,rank:0,zone:4,supply:false,entry:1},
 Gather{loc:1,rank:0,zone:6,supply:true,entry:2},Gather{loc:1,rank:0,zone:32,supply:false,entry:3},
 Gather{loc:1,rank:0,zone:1,supply:false,entry:65535}];
}
#[path="../mod/src/gather_zones.rs"]mod zones;
#[test]fn cache_preserves_sources_and_refreshes_on_map_or_rank_change(){
 for _ in 0..120{assert_eq!(zones::zones(1,1,0),(1<<2)|(1<<5));}
 assert_eq!(zones::zones(2,1,0),0);assert_eq!(zones::zones(3,1,0),0);assert_eq!(zones::zones(65535,1,0),0);
 assert_eq!(zones::zones(1,1,1),1<<3);assert_eq!(zones::zones(1,2,0),1<<4);assert_eq!(zones::zones(1,1,0),(1<<2)|(1<<5));
}
