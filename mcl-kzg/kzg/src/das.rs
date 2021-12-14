use crate::data_types::fr::Fr;
use crate::fk20_fft::*;
use crate::utilities::is_power_of_2;

impl FFTSettings {
    pub fn das_fft_extension(&self, values: &mut Vec<Fr>) -> Result<(), String> {
        if values.is_empty() {
            return Err(String::from("Values cannot be empty"));
        }
        if !is_power_of_2(values.len()) {
            return Err(String::from("Value count must be a number of two"));
        }
        if values.len() << 1 > self.max_width {
            return Err(String::from("ftt settings max width too small!"));
        }

        //larger stride if more roots fttsettings
        let stride = self.max_width / (values.len() * 2);
        self._das_fft_extension(values, stride);
        // just dividing every value by 1/(2**depth) aka length
        // TODO: what's faster, maybe vec[x] * vec[x], ask herumi to implement?
        let inv_length = Fr::from_int(values.len() as i32).get_inv();
        for item in values.iter_mut() {
            *item *= &inv_length;
        }
        Ok(())
    }

    // #[cfg(feature = "parallel")] 
    fn _das_fft_extension(&self, values: &mut [Fr], stride: usize) {
        if values.len() < 2 {
            return;
        }
        if values.len() == 2 {
            let (x, y) = FFTSettings::_calc_add_and_sub(&values[0], &values[1]);

            let temp = y * self.exp_roots_of_unity[stride];
            values[0] = x + temp;
            values[1] = x - temp;
            return;
        }

        let length = values.len();
        let half = length >> 1;
        
        for i in 0..half {
            let (add, sub) = FFTSettings::_calc_add_and_sub(&values[i], &values[half + i]);
            values[half + i] = sub * self.exp_roots_of_unity_rev[(i << 1) * stride];
            values[i] = add;
        }

        #[cfg(feature = "parallel")]
        {
            if values.len() > 32 {
                let (lo, hi) = values.split_at_mut(half);
                rayon::join(
                    || self._das_fft_extension(hi, stride * 2),
                    || self._das_fft_extension(lo, stride * 2),
                );
            } else {
                self._das_fft_extension(&mut values[..half], stride << 1);
                self._das_fft_extension(&mut values[half..], stride << 1);
            }
        }
        #[cfg(not(feature="parallel"))]
        {
            // left
            self._das_fft_extension(&mut values[..half], stride << 1);
            // right
            self._das_fft_extension(&mut values[half..], stride << 1);
        }
        

        for i in 0..half {
            let root = &self.exp_roots_of_unity[((i << 1) + 1) * stride];
            let y_times_root = &values[half + i] * root;

            let (add, sub) = FFTSettings::_calc_add_and_sub(&values[i], &y_times_root);
            values[i] = add;
            values[i + half] = sub;
        }
    }

    fn _calc_add_and_sub(a: &Fr, b: &Fr) -> (Fr, Fr) {
        (a + b, a - b)
    }
}
