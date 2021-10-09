// env RUSTFLAGS="-L <mcl>/lib" cargo run
use mcl_rust::*;

#[allow(non_snake_case)]
fn main() {
    println!("mcl version={:04x}", get_version());
    let b = init(CurveType::BN254);
    if !b {
        println!("init err");
    }
    let mut x = Fr::zero();
    println!("x={}", x.get_str(10));
    x.set_int(123456);
    println!("x={}", x.get_str(10));
    x.set_int(0xfff);
    println!("x={}", x.get_str(16));
    x.clear();
    println!("x={}", x.get_str(10));
    x.set_str("0x123", 0);
    println!("x={}", x.get_str(16));
    let buf = x.serialize();
    println!("serialize={:x?}", buf); // put hex byte
    let mut y = Fr::zero();
    if y.deserialize(&buf) {
        println!("y={}", y.get_str(16));
    } else {
        println!("err deserialize");
    }
    if x != y {
        println!("ng");
    }
    x.set_int(1);
    if x == y {
        println!("ng");
    }
    if !x.is_one() {
        println!("ng");
    }
    x.set_int(123);
    y.set_int(567);
    let mut z = unsafe { Fr::uninit() };
    Fr::add(&mut z, &x, &y);

    let x1 = Fr::from_str("1234", 10).unwrap();
    println!("x1={}", x1.get_str(10));

    println!("z={}", z.get_str(10));
    println!("x={}", x.get_str(10));
    println!("y={}", y.get_str(10));

    let mut P1 = unsafe { G1::uninit() };
    let mut P2 = unsafe { G1::uninit() };
    let mut Q1 = unsafe { G2::uninit() };
    let mut Q2 = unsafe { G2::uninit() };
    let mut e1 = unsafe { GT::uninit() };
    let mut e2 = unsafe { GT::uninit() };
    let mut e3 = unsafe { GT::uninit() };
    P1.set_hash_of("abc".as_bytes());
    Q1.set_hash_of("abc".as_bytes());
    pairing(&mut e1, &P1, &Q1);
    x.set_by_csprng();
    y.set_by_csprng();
    G1::mul(&mut P2, &P1, &x);
    G2::mul(&mut Q2, &Q1, &y);
    pairing(&mut e2, &P2, &Q2);
    GT::pow(&mut e3, &e1, &x);
    GT::pow(&mut e1, &e3, &y);
    if e1 == e2 {
        println!("ok");
    } else {
        println!("ng");
    }
}
