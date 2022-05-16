use crate::{host_state_database::Level2HostStateDatabase, ping_state::BinaryPingState};
use std::net::{IpAddr, Ipv4Addr};
use crate::pinger::ping_list_of_hosts;
use crate::host_state_database::HostStateDatabase;
use crate::ping_state::PingState;



pub fn ping_and_save_range(db: &mut Level2HostStateDatabase, start: u32, end: u32) {
    let mut ipaddr = start;
    let mut ip_range = Vec::new();
    debug!("Collecting {} hosts to ping", end-start);
    while ipaddr <= end {
        let ipaddr_obj = IpAddr::V4(Ipv4Addr::from([(ipaddr >> 24) as u8, (ipaddr >> 16) as u8, (ipaddr >> 8) as u8, ipaddr as u8]));
        ip_range.push(ipaddr_obj);
        ipaddr += 1;
    }
    
    let results = ping_list_of_hosts(ip_range.as_slice());
    for (addr, state) in results {
        let state = BinaryPingState::from(state);

        let addr = match addr {
            IpAddr::V6(_) => unreachable!("Should never have IPv6 addresses"),
            IpAddr::V4(addr) => addr
        };

        let (mut file, offset) = db.ipaddr_to_file_offset(addr);



        state.write_to(&mut file, offset).unwrap();
    }
}