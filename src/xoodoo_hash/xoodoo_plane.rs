const MAX_PLANE_BYTES: usize = 16;
const MAX_PLANE_LANES: usize = 4;
const LANE_SIZE: usize = 4; //bytes

#[derive(Debug, Clone)]
pub struct XoodooPlane {
    /// each lane in a plane is 32-bis
    lanes: Vec<u32>
}

impl XoodooPlane {

    pub fn new_from_plane(other: &Self) -> Self {
        Self {
            lanes: other.lanes.clone()
        }
    }

    pub fn new_with_capacity(num_lanes: usize) -> Self{
        assert!(num_lanes <= MAX_PLANE_LANES);
        let lanes = vec![0; num_lanes];
        Self {
            lanes
        }
    }

    pub fn new_from_bytes (data: &[u8], num_lanes: usize) -> Self {
        assert!(data.len() <= MAX_PLANE_BYTES);
        assert!(num_lanes <= MAX_PLANE_LANES);
        assert!(data.len() <= num_lanes * LANE_SIZE);

        let chunks = if data.len() % LANE_SIZE == 0 { 
            data.len() / LANE_SIZE //a lane must always be 32-bits
        } else {
            (data.len() / LANE_SIZE) + 1
        };
        assert!(chunks <= num_lanes);
        
        let mut lanes: Vec<u32> = vec![0; num_lanes];
        for i in 0..chunks {
            let mut integer: u32 = 0; 
            for j in 0..4 {
                if i*LANE_SIZE + j >= data.len() {
                    break;
                }
                integer = integer | ((data[i*LANE_SIZE + j] as u32) << 8*j);
            }
            lanes[i] = integer;
        }
        
        Self {
            lanes
        }
    }
    
    pub fn get_num_lanes(&self) -> usize {
        self.lanes.len()
    }
    
    pub fn xor_word(&mut self, lane:usize, x:u32) {
        self.lanes[lane] ^= x;
    }

    pub fn xor(&mut self, other: &XoodooPlane) {
        for i in 0..self.lanes.len() {
            self.lanes[i] ^= other.lanes[i];
        }
    }

    pub fn and(&mut self, other: &XoodooPlane) {
        for i in 0..self.lanes.len() {
            self.lanes[i] &= other.lanes[i];
        }
    }

    pub fn complement(&mut self) {
        for i in 0..self.lanes.len() {
            self.lanes[i] = !self.lanes[i];  
        }
    }

    /// x represents the lane, z represents the bit position in a lane
    /// the shift operation must be applied to all lanes in a plane
    /// the shifting is cyclic and can be performed in both direction by using positive/negative offsets
    pub fn shift(&mut self, x: i32, z: i32 ) {
        
        if self.lanes.len() == 1 {//NC-variant
            self.lanes[0] = shift_lane(self.lanes[0], z);
            return;
        }
        
        let mut tmp = Self::new_with_capacity(self.lanes.len());
        for i in 0..self.lanes.len() {
            let num_lanes = self.lanes.len() as i32;
            let index = i as i32;
            let index = (((index - x) % num_lanes) + num_lanes) % num_lanes;
            tmp.lanes[i] = shift_lane(self.lanes[index as usize], z);
        }
        self.lanes = tmp.lanes;
    }

    pub fn get_lanes(&self) -> Vec<u8>{
        let mut data = vec![];
        for i in 0..self.lanes.len() {
            let bytes = self.lanes[i].to_le_bytes();
            data.extend_from_slice(&bytes);
        }
        data
    }


}

#[inline]
pub fn shift_lane(lane: u32, z: i32) -> u32 {
	if z == 0 {
		lane
	} else {
        //reduce z in 32-bit shift range for both positive and negative cases
        let z = ((z % 32) + 32) % 32; 
		(lane << z) | (lane >> (32 - z))
	}
}
