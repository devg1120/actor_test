/*
 *  https://qiita.com/atsuk/items/6b620484aad019638795
 *  Actor model by Rust with Actix
 *
 */


extern crate actix;

use actix::prelude::*;


struct Ping(usize);

impl Message for Ping {
    type Result = usize;
}

struct Pkt{body: String,}

impl Message for Pkt {
    type Result = String;
}

// Testアクター構造体
struct Test{
    count: usize,
}

// Actorトレイト実装
impl Actor for Test {
    type Context = Context<Self>;

    // Testアクター開始時に呼ばれる処理
    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("started");

        // System停止処理
        //System::current().stop();
    }
    fn stopping(&mut self, _ctx: &mut Self::Context)-> Running {
        println!("stopping");
        Running::Stop

    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("stopped");
        System::current().stop();
    }
}

impl Handler<Ping> for Test {
    type Result = usize;

    fn handle(&mut self, msg: Ping, _: &mut Context<Self>) -> Self::Result {
        self.count += msg.0;
        self.count
    }
}

impl Handler<Pkt> for Test {
    type Result = String;

    fn handle(&mut self, msg: Pkt, _: &mut Context<Self>) -> Self::Result  {
        println!("recv:{}", msg.body);
        msg.body
    }
}

fn main() {

  let mut sys = System::new("test");


  sys.block_on(async {
    // start new actor
    let addr = Test { count: 10 }.start();

    addr.do_send(Pkt{body: "msg test1".to_string()});

    let res = addr.send(Ping(10)).await;

    println!("RESULT: {}", res.unwrap() == 20);

    addr.do_send(Pkt{body: "msg test2".to_string()});

    let res2 = addr.send(Ping(10)).await;

    println!("RESULT: {}", res2.unwrap() == 30);



    // stop system and exit
    //System::current().stop();

  });

}
