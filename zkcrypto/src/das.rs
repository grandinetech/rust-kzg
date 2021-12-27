use crate::fftsettings::ZkFFTSettings;
use crate::zkfr::blsScalar;
use kzg::{Fr, DAS};

impl DAS<blsScalar> for ZkFFTSettings {
    fn das_fft_extension(&self, val: &[blsScalar]) 
	-> Result<Vec<blsScalar>, String> {
		
		if val.is_empty() { // !(val.len() > 0)
			return Err(String::from("The list cannot be empty"));
		}
		else if !(val.len().is_power_of_two()) {
			return Err(String::from("The list must be power of two"));
		}
		else if val.len() * 2 > self.max_width {
			return Err(String::from("The list is too long"));
		}
		
	    // assert!(val.len() > 0);
        // assert!(val.len().is_power_of_two());
        // assert!(val.len() * 2 <= self.max_width);

        let mut vals = val.to_vec();
        let stride = self.max_width / (vals.len() * 2);

        self.das_fft_extension_stride(&mut vals, stride);

        let invlen = blsScalar::from_u64(val.len() as u64);
        let invlen = invlen.inverse();
		let mut odds = Vec::new(); //val.to_vec();
		self.das_fft_extension_stride(&mut odds, stride);

		//let odds = odds.iter().map(|f| f.mul(&invlen)).collect();
		
        for vale in &mut vals { // i in 0..vals.len()
            odds.push(vale.mul(&invlen));// = vals[i].mul(&invlen);
        }
		// for i in 0..vals.len(){
            // vals[i] = vals[i].mul(&invlen);
			// // println!("element={}", vals[i].mul(&invlen));
			// // println!("element2={}", vals[i].mul(&invlen));
        // }
		// for i in 0..vals.len() {
		// println!("element vals={}", vals[i]);
		// println!("element odds={}", odds[i]);
		// }
		
        Ok(odds)
    }
}

