pub struct Bitmap {
    size: usize,
    data: Vec<u32>,
}

impl Bitmap {
    pub fn new(size: usize) -> Bitmap {
        let data = vec![0; size];
        Bitmap { size, data }
    }

    pub fn test_and_set(&mut self, bucket: u32) -> bool {
        let word = bucket >> 5;
        let bit = 1 << (bucket & 0x1F);

        if (self.data[word as usize] & bit) != 0 {
            return true;
        }
        self.data[word as usize] |= bit;
        false
    }

    pub fn clear(&mut self) {
        for i in 0..self.size {
            self.data[i] = 0;
        }
    }
}
