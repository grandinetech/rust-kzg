use crate::msm::pippenger_utils::num_bits;

pub fn breakdown(window: usize, ncpus: usize) -> (usize, usize, usize) {
    const NBITS: usize = 255;

    option_env!("WINDOW_NX")
        .map(|v| {
            v.parse()
                .expect("WINDOW_NX environment variable must be valid number")
        })
        .map(|nx| {
            let ny = NBITS / window + 1;
            (nx, ny, NBITS / ny + 1)
        })
        .unwrap_or({
            let mut nx: usize;
            let mut wnd: usize;

            if NBITS > window * ncpus {
                nx = 1;
                wnd = num_bits(ncpus / 4);
                if (window + wnd) > 18 {
                    wnd = window - wnd;
                } else {
                    wnd = (NBITS / window).div_ceil(ncpus);
                    if (NBITS / (window + 1)).div_ceil(ncpus) < wnd {
                        wnd = window + 1;
                    } else {
                        wnd = window;
                    }
                }
            } else {
                nx = 2;
                wnd = window - 2;
                while (NBITS / wnd + 1) * nx < ncpus {
                    nx += 1;
                    wnd = window - num_bits(3 * nx / 2);
                }
                nx -= 1;
                wnd = window - num_bits(3 * nx / 2);
            }
            let ny = NBITS / wnd + 1;
            wnd = NBITS / ny + 1;

            (nx, ny, wnd)
        })
}
