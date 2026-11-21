//! Bounded Pareto filtering for seed searches only. The broad pass keeps all gear.
use crate::set_solver::{Catalog,Gear};
#[derive(Clone,Copy)]pub struct Mask{bits:[[u64;19];7],counts:[u16;7],kind:usize,row:u32,other:u32,pub ready:bool}
impl Mask{
 pub const fn new()->Self{Self{bits:[[0;19];7],counts:[0;7],kind:0,row:0,other:0,ready:false}}
 pub fn count(&self,k:usize)->u32{u32::from(self.counts[k])}
 pub fn index(&self,k:usize,mut n:u32)->u32{for(i,&bits)in self.bits[k].iter().enumerate(){let count=bits.count_ones();if n>=count{n-=count;continue;}let mut value=bits;for _ in 0..n{value&=value-1;}return i as u32*64+value.trailing_zeros();}unreachable!()}
 fn keep(&mut self){self.bits[self.kind][self.row as usize/64]|=1<<(self.row%64);self.counts[self.kind]+=1;self.row+=1;self.other=0;}
 pub fn tick(&mut self,c:&impl Catalog,goals:usize,pareto:bool,budget:u32)->u32{
  let mut work=0;while !self.ready&&work<budget{work+=1;
   if self.kind==5{self.kind=6;continue;}
   let count=if self.kind<5{c.armor_count(self.kind)}else{c.jewel_count()};
   if self.row==count{self.kind+=1;self.row=0;self.other=0;if self.kind==7{self.ready=true;}continue;}
   if !c.verified(self.kind,self.row){self.row+=1;self.other=0;continue;}
   if self.kind==6{let j=c.jewel(self.row);if !pareto||j.extra!=[0;2]||j.points[..goals].iter().any(|&p|p>0){self.keep();}else{self.row+=1;}continue;}
   if !pareto{self.keep();continue;}
   let a=c.armor(self.kind,self.row);
   if a.torso_up||a.extra!=[0;2]||self.other==count{self.keep();continue;}
   let j=self.other;self.other+=1;if j==self.row||!c.verified(self.kind,j){continue;}
   if dominates(c.armor(self.kind,j),a,goals,j<self.row){self.row+=1;self.other=0;}
  }work
 }
}
fn dominates(b:Gear,a:Gear,n:usize,earlier:bool)->bool{!b.torso_up&&b.extra==[0;2]&&b.slots>=a.slots&&b.points[..n].iter().zip(&a.points).all(|(b,a)|b>=a)&&(b.slots>a.slots||b.points[..n]!=a.points[..n]||earlier)}
