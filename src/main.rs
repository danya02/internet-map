mod ping_state;
mod pinger;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use fastping_rs::PingResult::{Idle, Receive};
use fastping_rs::Pinger;

fn main() {
    pretty_env_logger::init();
    let (pinger, results) = match Pinger::new(None, Some(56)) {
        Ok((pinger, results)) => (pinger, results),
        Err(e) => panic!(
            "Error creating pinger: {}\n\n\
            Most likely this is because raw sockets are not allowed. Try: \n\
              sudo setcap cap_net_raw=eip <path-to-executable>\n\
            or run a shell that gives its children the raw socket capability with:\
              sudo -E capsh --keep=1 --user=$USER --inh=cap_net_raw --addamb=cap_net_raw --user=$USER --", e),
    };

    pinger.add_ipaddr("8.8.8.8");
    pinger.add_ipaddr("1.1.1.1");
    pinger.add_ipaddr("7.7.7.7");
    pinger.add_ipaddr("2001:4860:4860::8888");
    pinger.ping_once();
    let mut remaining = 4;

    while remaining > 0 {
        remaining -= 1;
        match results.recv() {
            Ok(result) => match result {
                Idle { addr } => {
                    error!("Idle Address {}.", addr);
                }
                Receive { addr, rtt } => {
                    error!("Receive from Address {} in {:?}.", addr, rtt);
                }
            },
            Err(_) => panic!("Worker threads disconnected before the solution was found!"),
        }
    }
}