use actix::prelude::*;

struct Fibonacci(pub u32);

impl Message for Fibonacci {
    type Result = Result<u64, ()>;
}

struct SyncActor;

impl Actor for SyncActor {
    type Context = SyncContext<Self>;
}

impl Handler<Fibonacci> for SyncActor {
    type Result = Result<u64, ()>;

    fn handle(&mut self, msg: Fibonacci, _: &mut Self::Context) -> Self::Result {
        if msg.0 == 0 {
            Err(())
        } else if msg.0 == 1 {
            Ok(1)
        } else {
            let mut i = 0;
            let mut sum = 0;
            let mut last = 0;
            let mut curr = 1;
            while i < msg.0 - 1 {
                sum = last + curr;
                last = curr;
                curr = sum;
                i += 1;
            }
            Ok(sum)
        }
    }
}


fn main() {

  let mut sys = System::new("test");


  sys.block_on(async {

    // start sync arbiter with 3 threads
    let addr = SyncArbiter::start(3, || SyncActor);

    // send 5 messages
    for n in 5..50 {
        println!("{} : {:?}", n, addr.send(Fibonacci(n)).await.unwrap());
    }

    // stop system and exit
    System::current().stop();

  });
}


