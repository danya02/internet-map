use std::net::Ipv4Addr;

pub trait HostStateDatabase {
    /// Takes an IP address and returns the file that contains that IP address's state, as well as the offset inside that file.
    fn ipaddr_to_file_offset(&self, ipaddr: Ipv4Addr) -> (std::fs::File, usize);
}

/// Stores IP address data in files of form `192/168.bin`, which contains IP addresses in the range `192.168.0.0` to `192.168.255.255`.
/// Uses [`ping_state::BinaryPingState`] to store the state.
pub struct Level2HostStateDatabase {
    pub db_path: std::path::PathBuf,
    pub record_bit_size: u64,
}

impl HostStateDatabase for Level2HostStateDatabase {

    /// **Note:** This will not create directories if they do not exist.
    /// Please create them ahead of time with:
    ///   `mkdir -p /path/to/db/{0..255}`
    fn ipaddr_to_file_offset(&self, ipaddr: Ipv4Addr) -> (std::fs::File, usize) {
        let mut file_path = self.db_path.clone();
        file_path.push(format!("{}", ipaddr.octets()[0]));
        file_path.push(format!("{}", ipaddr.octets()[1]));
        file_path.set_extension("bin");
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path)
            .unwrap();
        
        // File must have length of (256*256) bits; `set_len` will not delete existing data, so we will call it every time.
        file.set_len(self.record_bit_size * 256*256/8).unwrap();
        let offset = ipaddr.octets()[2] as usize * 256 + ipaddr.octets()[3] as usize;
        (file, offset)
    }
}