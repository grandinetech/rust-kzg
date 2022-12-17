use std::iter;
use crate::data_types::{fr::*, g1::*, fp::*};
use crate::utilities::{is_power_of_2, next_pow_of_2};

// MODULUS = 52435875175126190479447740508185965837690552500527637822603658699938581184513
// PRIMITIVE_ROOT = 5
// [pow(PRIMITIVE_ROOT, (MODULUS - 1) // (2**i), MODULUS) for i in range(32)]
// TODO: gen dynamically?
pub const SCALE_2_ROOT_OF_UNITY_PR5_STRINGS: [&str; 32] = [
    "1",
    "52435875175126190479447740508185965837690552500527637822603658699938581184512",
    "3465144826073652318776269530687742778270252468765361963008",
    "23674694431658770659612952115660802947967373701506253797663184111817857449850",
    "14788168760825820622209131888203028446852016562542525606630160374691593895118",
    "36581797046584068049060372878520385032448812009597153775348195406694427778894",
    "31519469946562159605140591558550197856588417350474800936898404023113662197331",
    "47309214877430199588914062438791732591241783999377560080318349803002842391998",
    "36007022166693598376559747923784822035233416720563672082740011604939309541707",
    "4214636447306890335450803789410475782380792963881561516561680164772024173390",
    "22781213702924172180523978385542388841346373992886390990881355510284839737428",
    "49307615728544765012166121802278658070711169839041683575071795236746050763237",
    "39033254847818212395286706435128746857159659164139250548781411570340225835782",
    "32731401973776920074999878620293785439674386180695720638377027142500196583783",
    "39072540533732477250409069030641316533649120504872707460480262653418090977761",
    "22872204467218851938836547481240843888453165451755431061227190987689039608686",
    "15076889834420168339092859836519192632846122361203618639585008852351569017005",
    "15495926509001846844474268026226183818445427694968626800913907911890390421264",
    "20439484849038267462774237595151440867617792718791690563928621375157525968123",
    "37115000097562964541269718788523040559386243094666416358585267518228781043101",
    "1755840822790712607783180844474754741366353396308200820563736496551326485835",
    "32468834368094611004052562760214251466632493208153926274007662173556188291130",
    "4859563557044021881916617240989566298388494151979623102977292742331120628579",
    "52167942466760591552294394977846462646742207006759917080697723404762651336366",
    "18596002123094854211120822350746157678791770803088570110573239418060655130524",
    "734830308204920577628633053915970695663549910788964686411700880930222744862",
    "4541622677469846713471916119560591929733417256448031920623614406126544048514",
    "15932505959375582308231798849995567447410469395474322018100309999481287547373",
    "37480612446576615530266821837655054090426372233228960378061628060638903214217",
    "5660829372603820951332104046316074966592589311213397907344198301300676239643",
    "20094891866007995289136270587723853997043774683345353712639419774914899074390",
    "34070893824967080313820779135880760772780807222436853681508667398599787661631",
];

pub const SCALE_2_ROOT_OF_UNITY_PR7_STRINGS: [&str; 32] = [ "1",
/* k=1          r=2          */ "52435875175126190479447740508185965837690552500527637822603658699938581184512",
/* k=2          r=4          */ "3465144826073652318776269530687742778270252468765361963008",
/* k=3          r=8          */ "23674694431658770659612952115660802947967373701506253797663184111817857449850",
/* k=4          r=16         */ "14788168760825820622209131888203028446852016562542525606630160374691593895118",
/* k=5          r=32         */ "36581797046584068049060372878520385032448812009597153775348195406694427778894",
/* k=6          r=64         */ "31519469946562159605140591558550197856588417350474800936898404023113662197331", // iki cia pakeista
/* k=7          r=128        */ "3535074550574477753284711575859241084625659976293648650204577841347885064712",
/* k=8          r=256        */ "21071158244812412064791010377580296085971058123779034548857891862303448703672",
/* k=9          r=512        */ "12531186154666751577774347439625638674013361494693625348921624593362229945844",
/* k=10         r=1024       */ "21328829733576761151404230261968752855781179864716879432436835449516750606329",
/* k=11         r=2048       */ "30450688096165933124094588052280452792793350252342406284806180166247113753719",
/* k=12         r=4096       */ "7712148129911606624315688729500842900222944762233088101895611600385646063109",
/* k=13         r=8192       */ "4862464726302065505506688039068558711848980475932963135959468859464391638674",
/* k=14         r=16384      */ "36362449573598723777784795308133589731870287401357111047147227126550012376068",
/* k=15         r=32768      */ "30195699792882346185164345110260439085017223719129789169349923251189180189908",
/* k=16         r=65536      */ "46605497109352149548364111935960392432509601054990529243781317021485154656122",
/* k=17         r=131072     */ "2655041105015028463885489289298747241391034429256407017976816639065944350782",
/* k=18         r=262144     */ "42951892408294048319804799042074961265671975460177021439280319919049700054024",
/* k=19         r=524288     */ "26418991338149459552592774439099778547711964145195139895155358980955972635668",
/* k=20         r=1048576    */ "23615957371642610195417524132420957372617874794160903688435201581369949179370",
/* k=21         r=2097152    */ "50175287592170768174834711592572954584642344504509533259061679462536255873767",
/* k=22         r=4194304    */ "1664636601308506509114953536181560970565082534259883289958489163769791010513",
/* k=23         r=8388608    */ "36760611456605667464829527713580332378026420759024973496498144810075444759800",
/* k=24         r=16777216   */ "13205172441828670567663721566567600707419662718089030114959677511969243860524",
/* k=25         r=33554432   */ "10335750295308996628517187959952958185340736185617535179904464397821611796715",
/* k=26         r=67108864   */ "51191008403851428225654722580004101559877486754971092640244441973868858562750",
/* k=27         r=134217728  */ "24000695595003793337811426892222725080715952703482855734008731462871475089715",
/* k=28         r=268435456  */ "18727201054581607001749469507512963489976863652151448843860599973148080906836",
/* k=29         r=536870912  */ "50819341139666003587274541409207395600071402220052213520254526953892511091577",
/* k=30         r=1073741824 */ "3811138593988695298394477416060533432572377403639180677141944665584601642504",
/* k=31         r=2147483648 */ "43599901455287962219281063402626541872197057165786841304067502694013639882090",];

pub const G1_GENERATOR: G1 = G1 {
    x: Fp { d: [0x5cb38790fd530c16, 0x7817fc679976fff5, 0x154f95c7143ba1c1, 0xf0ae6acdf3d0e747, 0xedce6ecc21dbf440, 0x120177419e0bfb75] },
    y: Fp { d: [0xbaac93d50ce72271, 0x8c22631a7918fd8e, 0xdd595f13570725ce, 0x51ac582950405194, 0x0e1c8c3fad0059c0, 0x0bbc3efc5008a26a] },
    z: Fp { d: [0x760900000002fffd, 0xebf4000bc40c0002, 0x5f48985753c758ba, 0x77ce585370525745, 0x5c071a97a256ec6d, 0x15f65ec3fa80e493] },
};

pub const G1_NEGATIVE_GENERATOR: G1 = G1 {
    x: Fp { d: [0x5cb38790fd530c16, 0x7817fc679976fff5, 0x154f95c7143ba1c1, 0xf0ae6acdf3d0e747, 0xedce6ecc21dbf440, 0x120177419e0bfb75] },
    y: Fp { d: [0xff526c2af318883a, 0x92899ce4383b0270, 0x89d7738d9fa9d055, 0x12caf35ba344c12a, 0x3cff1b76964b5317, 0x0e44d2ede9774430] },
    z: Fp { d: [0x760900000002fffd, 0xebf4000bc40c0002, 0x5f48985753c758ba, 0x77ce585370525745, 0x5c071a97a256ec6d, 0x15f65ec3fa80e493] },
};

pub static mut SCALE_2_ROOT_OF_UNITY: Vec<Fr> = vec![];
pub static mut GLOBALS_INITIALIZED: bool = false;
pub static mut DEFAULT_GLOBALS_INITIALIZED: bool = false;
pub const PRIMITIVE_ROOT: i32 = 5;


pub fn make_data(n: usize) -> Vec<G1> {
  // Multiples of g1_gen
    if n == 0 {
        return vec![];
    }
    let mut data: Vec<G1> = vec![G1_GENERATOR];
    for _ in 1..n {
        let g1 = data.last().unwrap() + &G1_GENERATOR.clone();
        data.push(g1);
    }
    data
}

/// # Safety
///
/// use of mutable static is unsafe and requires unsafe function or block
pub unsafe fn init_globals() {
    if GLOBALS_INITIALIZED && DEFAULT_GLOBALS_INITIALIZED {
        return;
    }
    SCALE_2_ROOT_OF_UNITY = SCALE_2_ROOT_OF_UNITY_PR5_STRINGS.iter()
    .map(|x| Fr::from_str(x, 10).unwrap())
    .collect();

    GLOBALS_INITIALIZED = true;
    DEFAULT_GLOBALS_INITIALIZED = true;
}

/// # Safety
///
/// use of mutable static is unsafe and requires unsafe function or block
pub unsafe fn init_globals_custom(root_strings: [&str; 32]) {
    SCALE_2_ROOT_OF_UNITY = root_strings.iter()
    .map(|x| Fr::from_str(x, 10).unwrap())
    .collect();

    GLOBALS_INITIALIZED = true;
    DEFAULT_GLOBALS_INITIALIZED = false;
}

pub fn expand_root_of_unity(root: &Fr) -> Vec<Fr> {
    let mut root_z = vec![Fr::one(), *root];
    let mut i = 1;
    while !root_z[i].is_one() {
        let next = &root_z[i] * root;
        root_z.push(next);
        i += 1;
    }
    root_z
}

#[derive(Debug, Clone)]
pub struct FFTSettings {
    pub max_width: usize,
    pub root_of_unity: Fr,
    pub exp_roots_of_unity: Vec<Fr>,
    pub exp_roots_of_unity_rev: Vec<Fr>
}

impl FFTSettings {
    pub fn default() -> Self {
        Self::new(0)
    }
    //fix this mess
    /// # Safety
    ///
    /// use of mutable static is unsafe and requires unsafe function or block
    pub fn new(max_scale: u8) -> FFTSettings {
        let root: Fr;
        unsafe {
            init_globals();
            root = SCALE_2_ROOT_OF_UNITY[max_scale as usize]
        }
        let root_z = expand_root_of_unity(&root);
        let mut root_z_rev = root_z.clone();
        root_z_rev.reverse();

        FFTSettings {
            max_width: 1 << max_scale,
            root_of_unity: root,
            exp_roots_of_unity: root_z,
            exp_roots_of_unity_rev: root_z_rev
        }
    }

    /// # Safety
    ///
    /// use of mutable static is unsafe and requires unsafe function or block
    pub fn new_custom_primitive_roots(max_scale: u8, root_strings: [&str; 32]) -> Result<FFTSettings, String> {
        let root: Fr;
        unsafe {
            init_globals_custom(root_strings);
            if max_scale as usize >= SCALE_2_ROOT_OF_UNITY.len() {
                return Err(String::from("Scale is expected to be within root of unity matrix row size"));
            }
            root = SCALE_2_ROOT_OF_UNITY[max_scale as usize]
        }
        
        let root_z = expand_root_of_unity(&root);
        let mut root_z_rev = root_z.clone();
        root_z_rev.reverse();

        Ok(FFTSettings {
            max_width: 1 << max_scale,
            root_of_unity: root,
            exp_roots_of_unity: root_z,
            exp_roots_of_unity_rev: root_z_rev
        })
    }

    // #[cfg(feature = "parallel")] 
    fn _fft(values: &[Fr], offset: usize, stride: usize, roots_of_unity: &[Fr], root_stride: usize, out: &mut [Fr]) {
        // check if correct value is checked in case of a bug!
        if out.len() <= 4 { // if the value count is small, run the unoptimized version instead. // TODO tune threshold.
            return FFTSettings::_simple_ftt(values, offset, stride, roots_of_unity, root_stride, out);
        }

        let half = out.len() >> 1;

        #[cfg(feature = "parallel")] 
        {
            if half > 256 {
                let (lo, hi) = out.split_at_mut(half);
                rayon::join(
                    || FFTSettings::_fft(values, offset, stride << 1, roots_of_unity, root_stride << 1, lo),
                    || FFTSettings::_fft(values, offset + stride, stride << 1, roots_of_unity, root_stride << 1, hi),
                );
            } else {
                FFTSettings::_fft(values, offset, stride << 1, roots_of_unity, root_stride << 1, &mut out[..half]);
                FFTSettings::_fft(values, offset + stride, stride << 1, roots_of_unity, root_stride << 1, &mut out[half..]);
            }
        }
        #[cfg(not(feature="parallel"))]
        {
            // left
            FFTSettings::_fft(values, offset, stride << 1, roots_of_unity, root_stride << 1, &mut out[..half]);
            // right
            FFTSettings::_fft(values, offset + stride, stride << 1, roots_of_unity, root_stride << 1, &mut out[half..]); 
        }
        

        for i in 0..half {
            let root = &roots_of_unity[i * root_stride];
            let y_times_root = &out[i + half] * root;
            out[i + half] = out[i] - y_times_root;
            out[i] = out[i] + y_times_root;
        }
    }

    fn _simple_ftt(values: &[Fr], offset: usize, stride: usize, roots_of_unity: &[Fr], root_stride: usize, out: &mut [Fr]) {
        let out_len = out.len();
        let init_last = values[offset] * roots_of_unity[0];

        for i in 0..out_len {
            let mut last = init_last;
            for j in 1..out_len {
                let jv = &values[offset + j * stride];
                let r = &roots_of_unity[((i * j) % out_len) * root_stride];
                // last += (jv * r)
                last = last + (jv * r);
            }
            out[i] = last;
        }
    }

    pub fn inplace_fft(&self, values: &[Fr], inv: bool) -> Vec<Fr> {
        if inv {
            let root_z: Vec<Fr> = self.exp_roots_of_unity_rev.iter().copied().take(self.max_width).collect();
            let stride = self.max_width / values.len();

            let mut out = vec![Fr::default(); values.len()];
            FFTSettings::_fft(values, 0, 1, &root_z, stride, &mut out);

            let inv_len = Fr::from_int(values.len() as i32).get_inv();
            for item in out.iter_mut() {
                *item = *item * inv_len;
            }
            out
        } else {
            let root_z: Vec<Fr> = self.exp_roots_of_unity.iter().copied().take(self.max_width).collect();
            let stride = self.max_width / values.len();

            let mut out = vec![Fr::default(); values.len()];
            FFTSettings::_fft(values, 0, 1, &root_z, stride, &mut out);

            out
        }
    }

    pub fn fft(&self, values: &[Fr], inv: bool) -> Result<Vec<Fr>, String> {
        if values.len() > self.max_width {
            return Err(String::from("Supplied values is longer than the available max width"));
        }
        let n = next_pow_of_2(values.len());
        
        let diff = n - values.len();
        let tail= iter::repeat(Fr::zero()).take(diff);
        let values_copy: Vec<Fr> = values.iter().copied()
            .chain(tail)
            .collect();

        Ok(self.inplace_fft(&values_copy, inv))
    }

    pub fn fft_fr_slow(result: &mut [Fr], values: &[Fr], stride: usize, passed_roots_of_unity: &[Fr], root_stride: usize) {
        FFTSettings::_simple_ftt(values, 0, stride, passed_roots_of_unity, root_stride, result);
    }

    pub fn fft_fr_fast(result: &mut [Fr], values: &[Fr], stride: usize, passed_roots_of_unity: &[Fr], root_stride: usize) {
        FFTSettings::_fft(values, 0, stride, passed_roots_of_unity, root_stride, result);
    }

    pub fn fft_g1(&self, values: &[G1]) -> Result<Vec<G1>, String> {
        if values.len() > self.max_width {
            return Err(String::from("length of values is longer than the available max width"));
        } 
        if !is_power_of_2(values.len()) {
            return Err(String::from("length of values must be a power of two"));
        }
        // TODO: check if copy can be removed, opt?
        // let vals_copy = values.clone();
        
        let root_z: Vec<Fr> = self.exp_roots_of_unity.iter()
            .take(self.max_width).copied()
            .collect();

        let stride = self.max_width /  values.len();
        let mut out = vec![G1::zero(); values.len()];

        FFTSettings::_fft_g1(values, 0, 1, &root_z, stride, &mut out);

        Ok(out)
    }

    //just copied of for fk20_matrix
    pub fn fft_g1_inv(&self, values: &[G1]) -> Result<Vec<G1>, String> {
        if values.len() > self.max_width {
            return Err(String::from("length of values is longer than the available max width"));
        } 
        if !is_power_of_2(values.len()) {
            return Err(String::from("length of values must be a power of two"));
        }
        // TODO: check if copy can be removed, opt?
        // let vals_copy = values.clone();
        
        let root_z: Vec<Fr> = self.exp_roots_of_unity_rev.iter()
            .take(self.max_width).copied()
            .collect();

        let stride = self.max_width /  values.len();
        let mut out = vec![G1::zero(); values.len()];

        FFTSettings::_fft_g1(values, 0, 1, &root_z, stride, &mut out);
        
        let inv_len = Fr::from_int(values.len() as i32).get_inv();
        for item in out.iter_mut() {
        // for i in 0..out.len() {
            *item = &*item * &inv_len;
        }

        Ok(out)
    }

    // #[cfg(feature = "parallel")] 
    fn _fft_g1(values: &[G1], value_offset: usize, value_stride: usize, roots_of_unity: &[Fr], roots_stride: usize, out: &mut [G1]) {
        //TODO: fine tune for opt, maybe resolve number dinamically based on experiments
        if out.len() <= 4 {
            return FFTSettings::_fft_g1_simple(values, value_offset, value_stride, roots_of_unity, roots_stride, out);
        }

        let half = out.len() >> 1;

        #[cfg(feature = "parallel")] 
        {
            let (lo, hi) = out.split_at_mut(half);
            rayon::join(
                || FFTSettings::_fft_g1(values, value_offset, value_stride << 1, roots_of_unity, roots_stride << 1, lo),
                || FFTSettings::_fft_g1(values, value_offset + value_stride, value_stride << 1, roots_of_unity, roots_stride << 1, hi),
            );
    
        }
        #[cfg(not(feature="parallel"))]
        {
            // left
            FFTSettings::_fft_g1(values, value_offset, value_stride << 1, roots_of_unity, roots_stride << 1, &mut out[..half]);
            // right
            FFTSettings::_fft_g1(values, value_offset + value_stride, value_stride << 1, roots_of_unity, roots_stride << 1, &mut out[half..]); 
        }
        
        for i in 0..half {
            let x = out[i];
            let y = out[i + half];
            let root = &roots_of_unity[i * roots_stride];

            let y_times_root = y * root;
            G1::add(&mut out[i], &x, &y_times_root);
            out[i + half] = x - y_times_root;
        }
    }

    fn _fft_g1_simple(values: &[G1], value_offset: usize, value_stride: usize, roots_of_unity: &[Fr], roots_stride: usize, out: &mut [G1]) {
        let l = out.len();
        for i in 0..l {
            // TODO: check this logic with a working brain, there could be a simpler way to write this;
            let mut v = &values[value_offset] * &roots_of_unity[0];
            let mut last = v;
            for j in 1..l {
                v = &values[value_offset + j * value_stride] * &roots_of_unity[((i * j) % l) * roots_stride];
                let temp = last;
                last = &temp + &v;
            }
            out[i] = last;
        }
    }

    pub fn fft_g1_slow(out: &mut [G1], values: &[G1], stride: usize, passed_roots_of_unity: &[Fr], root_stride: usize, _n: usize) {
        FFTSettings::_fft_g1_simple(values, 0, stride, passed_roots_of_unity, root_stride, out);
    }

    pub fn fft_g1_fast(out: &mut [G1], values: &[G1], stride: usize, passed_roots_of_unity: &[Fr], root_stride: usize, _n: usize) {
        FFTSettings::_fft_g1(values, 0, stride, passed_roots_of_unity, root_stride, out);
    }
}
