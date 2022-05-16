use std::{net::IpAddr, time::Duration};

use fastping_rs::Pinger;

use crate::ping_state::GeneralPingState;

const MAX_RTT: Duration = Duration::from_millis(250);

pub fn ping_list_of_hosts(hosts: &[IpAddr]) -> Vec<(IpAddr, GeneralPingState)> {
    trace!("Initializing pinger");
    let (pinger, results) = match Pinger::new(Some(MAX_RTT.as_millis().try_into().unwrap()), Some(56)) {
        Ok((pinger, results)) => (pinger, results),
        Err(e) => panic!(
            "Error creating pinger: {}\n\n\
            Most likely this is because raw sockets are not allowed. Try: \n\
              sudo setcap cap_net_raw=eip <path-to-executable>\n\
            or run a shell that gives its children the raw socket capability with:\
              sudo -E capsh --keep=1 --user=$USER --inh=cap_net_raw --addamb=cap_net_raw --user=$USER --", e),
    };
    let mut remaining = hosts.len();
    trace!("Adding {} hosts to pinger", remaining);
    for host in hosts {
        pinger.add_ipaddr(host.to_string().as_str());
    }
    let mut ping_results = Vec::new();
    pinger.ping_once();

    while remaining > 0 {
        remaining -= 1;
        trace!("{} hosts remaining", remaining);
        match results.recv() {
            Ok(result) => match result {
                fastping_rs::PingResult::Idle { addr } => {
                    debug!("Ping to {} timed out", addr);
                    ping_results.push((addr, GeneralPingState{ is_up: false, is_tested: true, time_taken: MAX_RTT }));
                }
                fastping_rs::PingResult::Receive { addr, rtt } => {
                    debug!("Ping to {} returned in {} ms", addr, rtt.as_millis());
                    ping_results.push((addr, GeneralPingState {is_up: true, is_tested: true, time_taken: rtt} ));
                }
            },
            Err(_) => panic!("Worker threads disconnected before the solution was found!"),
        }
    }


    ping_results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_localhost() {
        let results = ping_list_of_hosts(&[ [127, 0, 0, 1].into()]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.is_up, true);
    }
}