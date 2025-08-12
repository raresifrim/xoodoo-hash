mod xoodoo_plane;
pub mod xoodoo_state;

use crate::xoodoo_hash::xoodoo_state::{XoodooState,XoodooStateFull,XoodooStateNC};

const MAX_ROUNDS: usize = 12;

pub struct XoodooHash<XoodooState> {
    state: XoodooState,
    num_rounds: usize
}

impl XoodooHash<XoodooStateFull>  {
    pub fn new(num_rounds: usize, num_lanes_per_plane: usize) -> Self{
        assert!(num_rounds <= MAX_ROUNDS);
        let state = XoodooState::initialize(Some(num_lanes_per_plane));
        Self {
            state,
            num_rounds
        }
    }

    pub fn new_from_bytes(data: &[u8], num_rounds: usize, num_lanes_per_plane: usize) -> Self{
        assert!(num_rounds <= MAX_ROUNDS);
        assert!(data.len() <= 3 * num_lanes_per_plane * 4);
        let state = XoodooState::new_from_bytes(data, Some(num_lanes_per_plane));
        Self {
            state,
            num_rounds
        }
    }

    /// add next input into state
    pub fn next(&mut self, data: &[u8]) {
        self.state.xor_state(data);
    }

    pub fn permute(&mut self) {
        for i in (MAX_ROUNDS-self.num_rounds)..MAX_ROUNDS {
            self.state.round(i);
        }
    }

    /// get current hash digest
    pub fn digest(&self) -> Vec<u8> {
        self.state.get_state()
    }
}


impl XoodooHash<XoodooStateNC>  {
   
   pub fn new(data: &[u8]) -> Self{
        assert!(data.len() <= 12);
        let state = XoodooState::new_from_bytes(data, None);
        Self {
            state,
            num_rounds: 3
        }
    }

    pub fn permute_nc(&mut self) {
        self.state.round(0);
        self.state.round(1);
        self.state.round(2);
    }

    pub fn digest_nc(&self) -> Vec<u8> {
        self.state.get_state()
    }

}

