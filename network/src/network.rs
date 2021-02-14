use actix::prelude::*;


pub struct SubNetwork {
    pub name: String,
    pub addr: Addr<SubNetwork>,
}

impl Actor for SubNetwork {
    type Context = Context<SubNetwork>;
}

pub struct Manager_SubNetwork {
    pub name: String,
}


impl Manager_SubNetwork {
     pub fn create(&self,subnet: &str) {
             println!("subnet create!");
             SubNetwork::create(|ctx| {
                 let addr = ctx.address();

                 SubNetwork {
                     //name: "sub1".to_string(),
                     name: subnet.to_string(),
                     addr: addr,
                 }
             });
     }
}

pub fn network() {

  println!("network");

}
