mod ping_state;
mod pinger;
mod host_state_database;
mod ping_job;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use std::net::Ipv4Addr;
use std::path::PathBuf;

use fastping_rs::PingResult::{Idle, Receive};
use fastping_rs::Pinger;
use ping_state::{BinaryPingState, PingState};
use crate::host_state_database::Level2HostStateDatabase;

fn main() {
    pretty_env_logger::init();


    let mut db = Level2HostStateDatabase { db_path: PathBuf::from("./data/"), record_bit_size: BinaryPingState::bit_size() as u64 };

    for part_2 in 0..=255 {
        for part_3 in 0..=127 {
            let start = Ipv4Addr::new(127, part_2, 2*part_3, 1).into();
            let end = Ipv4Addr::new(127, part_2, 2*part_3+1, 255).into();
            ping_job::ping_and_save_range(&mut db, start, end);
        }
    }


}