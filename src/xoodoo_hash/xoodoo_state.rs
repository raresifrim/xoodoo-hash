use crate::xoodoo_hash::xoodoo_plane::{shift_lane, XoodooPlane};

const MAX_STATE_BYTES: usize = 48;
const MAX_STATE_WORDS: usize = 12;
const MAX_NUM_PLANES: usize = 3;

const ROUND_CONSTANTS: [u32;12] = [
		0x00000058,
		0x00000038,
		0x000003C0,
		0x000000D0,
		0x00000120,
		0x00000014,
		0x00000060,
		0x0000002C,
		0x00000380,
		0x000000F0,
		0x000001A0,
		0x00000012,
];


pub trait XoodooState {
    fn new_from_bytes(data: &[u8], num_lanes_per_plane: Option<usize>) -> Self;

    fn initialize(num_lanes_per_plane: Option<usize>) -> Self;

    fn xor_state(&mut self, data: &[u8]);

    fn get_plane(&self, index: usize) -> XoodooPlane;

    fn theta_step(&mut self);

    fn rho_west_step(&mut self);

    fn iota_step(&mut self, i:usize);

    fn chi_step(&mut self);

    fn rho_east_step(&mut self);

    fn round(&mut self, i:usize);

    fn get_state(&self) -> Vec<u8>;
}

pub struct XoodooStateFull {
    planes: Vec<XoodooPlane>
}

impl XoodooState for XoodooStateFull{
    fn new_from_bytes(data: &[u8], num_lanes_per_plane: Option<usize>) -> Self {
        let num_lanes = match num_lanes_per_plane {
            Some(x) => x,
            None => 4 //default behaviour
        };
       
        assert!(data.len() <= MAX_STATE_BYTES);
        assert!(num_lanes * 3 <= MAX_STATE_WORDS);
        
        let mut planes= vec![];
        let bytes_per_plane = num_lanes * 4;
        assert!(data.len() <= bytes_per_plane * 3);
        
        for i in 0..MAX_NUM_PLANES {
            let start = i * bytes_per_plane;
            let end = i * bytes_per_plane + bytes_per_plane;
            if data.len() > end {
                planes.push(XoodooPlane::new_from_bytes(&data[start..end], num_lanes));
            } else {
                planes.push(XoodooPlane::new_from_bytes(&data[start..data.len()], num_lanes));
            }
        }
        
        Self { planes }
    }

    fn initialize(num_lanes_per_plane: Option<usize>) -> Self {
        let num_lanes = match num_lanes_per_plane {
            Some(x) => x,
            None => 4 //default behaviour
        };

        assert!(num_lanes * 3 <= MAX_STATE_WORDS);
        
        let mut planes= vec![];
        for _ in 0..MAX_NUM_PLANES {
            planes.push(XoodooPlane::new_with_capacity(num_lanes));
        }
        Self { planes }
    }

    fn xor_state(&mut self, data: &[u8]) {
        let input = Self::new_from_bytes(
            data,
            Some(self.planes[0].get_num_lanes())
        );
        for i in 0..self.planes.len(){
            self.planes[i].xor(&input.planes[i]);
        }
    }

    fn get_plane(&self, index: usize) -> XoodooPlane {
        assert!(index < self.planes.len());
        self.planes[index].clone()
    }

    fn theta_step(&mut self) {
        let mut p1 = XoodooPlane::new_from_plane(&self.planes[0]);
        p1.xor(&self.planes[1]);
        p1.xor(&self.planes[2]);

        let mut p2 = XoodooPlane::new_from_plane(&p1);
        p1.shift(1, 5);
        p2.shift(1, 14);

        let mut e = XoodooPlane::new_from_plane(&p1);
        e.xor(&p2);

        for i in 0..self.planes.len() {
            self.planes[i].xor(&e);
        }
    }

    fn rho_west_step(&mut self){
        self.planes[1].shift(1,0);
        self.planes[2].shift(0,11);
    }

    fn iota_step(&mut self, i:usize){
	    self.planes[0].xor_word(0,ROUND_CONSTANTS[i]);
    }

    fn chi_step(&mut self){
	    
	    let mut b0 = XoodooPlane::new_from_plane(&self.planes[1]);
        b0.complement();
        b0.and(&self.planes[2]);

        let mut b1 = XoodooPlane::new_from_plane(&self.planes[2]);
        b1.complement();
        b1.and(&self.planes[0]);

        let mut b2 = XoodooPlane::new_from_plane(&self.planes[0]);
        b2.complement();
        b2.and(&self.planes[1]);
	    
		self.planes[0].xor(&b0);
        self.planes[1].xor(&b1);
        self.planes[2].xor(&b2);    
    }

    fn rho_east_step(&mut self){
	    self.planes[1].shift(0, 1);
	    self.planes[2].shift(2, 8);
    }

    fn round(&mut self, i:usize)
    {
	    self.theta_step();
	    self.rho_west_step();
	    self.iota_step(i);
	    self.chi_step();
	    self.rho_east_step();
    }

    fn get_state(&self) -> Vec<u8> {
        let mut data = vec![];
        for i in 0..self.planes.len() {
            let mut bytes = self.planes[i].get_lanes();
            data.append(&mut bytes);
        }
        data
    }
}


pub struct XoodooStateNC {
    pub planes: [u32; 3]
}

impl XoodooStateNC {
    pub fn new_from_u32(data: u32) -> Self {
        Self { planes: [data, 0, 0] }
    }

    pub fn new_from_u64(data: u64) -> Self {
        Self { planes: [(data & u32::MAX as u64) as u32 , (data >> 32) as u32, 0] }
    }
}

impl XoodooState for XoodooStateNC{
    
    fn new_from_bytes(data: &[u8], num_lanes_per_plane: Option<usize>) -> Self {
        assert!(data.len() <= 12); //96-bit state in this case
        
        let num_lanes = 1;
        let mut planes= [0; 3];
        let bytes_per_plane = num_lanes * 4;
        
        for i in 0..MAX_NUM_PLANES {
            let start = i * bytes_per_plane;
            let end = if i * bytes_per_plane + 4 <= data.len() {
                i * bytes_per_plane + 4   
            } else {
                data.len()
            };
            for j in 0..(end-start) {
                planes[i] |= (data[start + j] as u32) << 8*j;
            }
        }
        
        Self { planes }
    }

    fn initialize(num_lanes_per_plane: Option<usize>) -> Self {
        let planes= [0; 3];
        Self { planes }
    }

    fn xor_state(&mut self, data: &[u8]) {
        let input = Self::new_from_bytes(
            data,
            Some(1)
        );
        for i in 0..MAX_NUM_PLANES {
            self.planes[i] ^= input.planes[i];
        }
    }

    fn get_plane(&self, index: usize) -> XoodooPlane {
        unimplemented!()
    }

    #[inline]
    fn theta_step(&mut self) {
        let mut p1 = self.planes[0] ^ self.planes[1] ^ self.planes[2];
        let mut p2 = p1;
        
        p1 = shift_lane(p1, 5);
        p2 = shift_lane(p2, 14);
        
        let e = p1 ^ p2;

        self.planes[0] ^= e;
        self.planes[1] ^= e;
        self.planes[2] ^= e;
    }

    #[inline]
    fn rho_west_step(&mut self){
        self.planes[2] = shift_lane(self.planes[2], 11);
    }

    #[inline]
    fn iota_step(&mut self, i:usize){
	    self.planes[0] ^= ROUND_CONSTANTS[i];
    }

    #[inline]
    fn chi_step(&mut self){
	    
	    let b0 = !self.planes[1] ^ self.planes[2];
        let b1 = !self.planes[2] ^ self.planes[0];
        let b2 = !self.planes[0] ^ self.planes[1];
	    
		self.planes[0] ^= b0;
        self.planes[1] ^= b1;
        self.planes[2] ^= b2;    
    }

    #[inline]
    fn rho_east_step(&mut self){
	    self.planes[1] = shift_lane(self.planes[1], 1);
	    self.planes[2] = shift_lane(self.planes[2], 8);
    }

    #[inline]
    fn round(&mut self, i:usize)
    {
	    self.theta_step();
	    self.rho_west_step();
	    self.iota_step(i);
	    self.chi_step();
	    self.rho_east_step();
    }

    fn get_state(&self) -> Vec<u8> {
        let mut data = vec![];
        for i in 0..self.planes.len() {
            let mut bytes = self.planes[i].to_le_bytes();
            data.extend_from_slice(&mut bytes);
        }
        data
    }
}
