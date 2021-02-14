/*
 *  https://qiita.com/atsuk/items/6b620484aad019638795
 *  Actor model by Rust with Actix
 *
 */


extern crate actix;

use actix::prelude::*;

mod network;
mod router;

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

  network::network();
  router::router();

  let mut sys = System::new("test");

  let mut ms = network::Manager_SubNetwork{ name: "TEST".to_string()};
  ms.create("10.2.1.0/24");
  

}

/*
    rm = RouterManager()

    asys = ActorSystem()

    manager_subnet = Manager_SubNetwork(asys)
    
    subnet1 = manager_subnet.create("10.2.1.0/24")
    subnet2 = manager_subnet.create("10.2.2.0/24")
    subnet3 = manager_subnet.create("10.2.3.0/24")
    
    print(asys.ask(subnet1, "name"))
    print(asys.ask(subnet2, "name"))
    print(asys.ask(subnet3, "name"))

    RA = router.Router("RouterA", asys, manager_subnet)
    RA.iface_create("eth0", 1000)
    RA.iface_config("eth0", "10.2.1.1", "255.255.255.0")

    RA.iface_create("eth1", 1000)
    RA.iface_config("eth1", "10.2.2.1", "255.255.255.0")

    RB = router.Router("RouterB", asys, manager_subnet)
    RB.iface_create("eth0", 1000)
    RB.iface_config("eth0", "10.2.3.1", "255.255.255.0")

    RB.iface_create("eth1", 1000)
    RB.iface_config("eth1", "10.2.2.254", "255.255.255.0")

    #router_timer = PeriodicTimer(5, router.poll)
    #router_timer = PeriodicTimer(5, poll)

    router_timer = PeriodicTimer(5, rm.poll)
    router_timer.start()

    rm.regist(RA)
    rm.regist(RB)

    RA.start()
    RB.start()


    sys.exit()


*/

/*
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
*/
