use std::time::Duration;

/// Trait for all the representations of the ping state.
pub trait PingState: Sized {
    /// Does this state indicate that the machine at this address is up?
    fn is_up(&self) -> bool;

    /// Does this state indicate that the machine at this address is down?
    /// 
    /// If the machine is already tested, then this is the opposite of [`is_up`].
    /// If the machine has not yet been tested, then it is possible for both [`is_up`] and [`is_down`] to be false.
    fn is_down(&self) -> bool;

    /// Does this state indicate that this has already been tested?
    /// 
    /// If this returns false, then the values of [`is_up`] and [`is_down`] are not guaranteed to be correct.
    fn is_tested(&self) -> bool;


    /// Write the current state into the given [`std::io::Write`]+[`std::io::Read`]+[`std::io::Seek`] object.
    /// 
    /// Takes the `offset` of this ping state into the file. For example, an `offset` of 0 means to write this state
    /// as the first ping state in the file.
    /// 
    /// The state might occupy less than one byte. If that's the case, then the byte is first read, then
    /// updated with the new state, and then written back.
    fn write_to<W: std::io::Read + std::io::Write + std::io::Seek>(&self, writer: &mut W, offset: usize) -> std::io::Result<()>;

    /// Read the state from the given [`std::io::Read`]+[`std::io::Seek`] object.
    /// 
    /// Takes the `offset` of this ping state into the file. For example, an `offset` of 0 means to read this state
    /// from the first ping state in the file.
    fn read_from<R: std::io::Read + std::io::Seek>(reader: &mut R, offset: usize) -> std::io::Result<Self>;

}

/// The most general ping state, containing all possible data values for the state.
/// 
/// This exists to allow transformations between different representations of the ping state.
pub struct GeneralPingState {
    /// Is this host up?
    pub is_up: bool,
    /// Was this host tested?
    pub is_tested: bool,
    /// How long did the ping take?
    pub time_taken: Duration,
}

impl From<BinaryPingState> for GeneralPingState {
    fn from(state: BinaryPingState) -> Self {
        GeneralPingState {
            is_up: state.0,
            is_tested: state.0,
            time_taken: Duration::MAX,
        }
    }
}


/// A ping state that stores a single bit to indicate whether the machine is (up) or (down or not tested).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinaryPingState(bool);
impl BinaryPingState {
    /// Return if the machine at this address is up.
    /// The single bit contained in this state directly indicates the answer.
    fn is_up(&self) -> bool {
        self.0
    }

    /// Return if the machine at this address is down.
    /// If this is true, then the machine could also have been not tested.
    fn is_down(&self) -> bool {
        !self.0
    }

    /// Return if this state has already been tested.
    /// This is true iff `is_up` is true.
    fn is_tested(&self) -> bool {
        self.0
    }

    /// Read the Nth bit from the file.
    /// 
    /// The 0th bit is the most significant bit of the first byte, and the 8th bit is the MSB of the second byte.
    /// The 7th bit is the LSB of the first byte.
    fn read_from<R: std::io::Read + std::io::Seek>(reader: &mut R, offset: usize) -> std::io::Result<Self> {
        let byte_offset = offset / 8;
        reader.seek(std::io::SeekFrom::Start(byte_offset as u64))?;
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        // Get the bit offset
        // Turn it around so that the 0 turns into 7 and 7 turns into 0, because the MSB should be bit 0.
        let bit_offset = 7 - (offset % 8);
        let bit = buf[0] >> bit_offset & 1;

        Ok(BinaryPingState(bit == 1))
    }

    /// Write the Nth bit to the file.
    fn write_to<W: std::io::Read + std::io::Write + std::io::Seek>(&self, writer: &mut W, offset: usize) -> std::io::Result<()> {
        // read the old byte
        let byte_offset = offset / 8;
        writer.seek(std::io::SeekFrom::Start(byte_offset as u64))?;
        let mut buf = [0u8; 1];
        writer.read_exact(&mut buf)?;

        // update the bit
        // first determine the bit's offset inside the byte
        // Turn it around so that the 0 turns into 7 and 7 turns into 0, because the MSB should be bit 0.
        let bit_offset = 7 - (offset % 8);
        // then determine the new value of the bit
        let bit = if self.0 { 1 } else { 0 };
        // first, zero out the bit
        let mask = 255u8 ^ (1 << bit_offset);
        buf[0] &= mask;
        // then, set the bit to the new value
        buf[0] |= bit << bit_offset;
        // write the new byte
        writer.seek(std::io::SeekFrom::Start(byte_offset as u64))?;
        writer.write_all(&buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_ping_state() {
        let state = BinaryPingState(true);
        assert!(state.is_up());
        assert!(!state.is_down());
        assert!(state.is_tested());

        let state = BinaryPingState(false);
        assert!(!state.is_up());
        assert!(state.is_down());
        assert!(!state.is_tested());
    }

    #[test]
    fn test_binary_ping_write(){
        let mut buf = [0u8; 4];
        let up = BinaryPingState(true);
        let down = BinaryPingState(false);

        let mut buf_cursor = std::io::Cursor::new(&mut buf[..]);

        for i in 0..4 {
            up.write_to(&mut buf_cursor, 2*i).unwrap();
            down.write_to(&mut buf_cursor, 2*i+1).unwrap();
        }
        for i in 4..8 {
            down.write_to(&mut buf_cursor, 2*i).unwrap();
            up.write_to(&mut buf_cursor, 2*i+1).unwrap();
        }
        for i in 16..24 {
            up.write_to(&mut buf_cursor, i).unwrap();
        }
        for i in 24..32 {
            down.write_to(&mut buf_cursor, i).unwrap();
        }

        assert_eq!(buf, [0b10101010, 0b01010101, 0b11111111, 0b00000000]);
    }

    #[test]
    fn test_binary_ping_read(){
        let data = [0b10101010, 0b01010101, 0b11111111, 0b00000000];
        let mut expected_values = Vec::new();
        let up = BinaryPingState(true);
        let down = BinaryPingState(false);
        for _ in 0..4 {
            expected_values.push(up);
            expected_values.push(down);
        }
        for _ in 0..4 {
            expected_values.push(down);
            expected_values.push(up);
        }
        for _ in 0..8 {
            expected_values.push(up);
        }
        for _ in 0..8 {
            expected_values.push(down);
        }

        let mut actual_values = Vec::new();
        let mut buf_cursor = std::io::Cursor::new(&data[..]);
        for i in 0..32 {
            actual_values.push(BinaryPingState::read_from(&mut buf_cursor, i).unwrap());
        }

        assert_eq!(actual_values, expected_values);

    }
}

