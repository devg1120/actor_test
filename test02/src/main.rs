/*
 *  https://qiita.com/atsuk/items/6b620484aad019638795
 *  Actor model by Rust with Actix
 *
 */


extern crate actix;

use actix::prelude::*;

struct Sum(usize, usize);

impl Message for Sum {
    type Result = usize;
}

// Actor definition
struct Calculator;

impl Actor for Calculator {
    type Context = Context<Self>;
}

// now we need to implement `Handler` on `Calculator` for the `Sum` message.
impl Handler<Sum> for Calculator {
    type Result = usize; // <- Message response type

    fn handle(&mut self, msg: Sum, ctx: &mut Context<Self>) -> Self::Result {
        msg.0 + msg.1
    }
}


fn main() {

  let mut sys = System::new("test");


  sys.block_on(async {
    // start new actor
    let addr = Calculator.start();
    let res = addr.send(Sum(10, 5)).await; // <- send message and get future for result

    match res {
        Ok(result) => println!("SUM: {}", result),
        _ => println!("Communication to the actor has failed"),
    }
  });

}
