macro_rules! field_test {
    ($t:ty) => {{
        let mut x = <$t>::zero();
        assert!(x.is_valid());
        assert!(x.is_zero());
        assert!(!x.is_one());
        x.set_int(1);
        assert!(!x.is_zero());
        assert!(x.is_one());
        let mut y = <$t>::from_int(1);
        assert!(y.is_valid());
        assert_eq!(x, y);
        y.set_int(2);
        assert!(x != y);
        x.set_str("65535", 10);
        y.set_str("ffff", 16);
        assert!(x.is_valid());
        assert_eq!(x, y);
        x.set_int(123);
        assert!(x.is_odd());
        x.set_int(124);
        assert!(!x.is_odd());
        assert!(!x.is_negative());
        x.set_int(-124);
        assert!(x.is_negative());

        let mut z = unsafe { <$t>::uninit() };
        let mut w = unsafe { <$t>::uninit() };

        let a = 256;
        let b = 8;
        x.set_int(a);
        y.set_int(b);
        <$t>::add(&mut z, &x, &y);
        w.set_int(a + b);
        assert_eq!(z, w);
        assert_eq!(w, (&x + &y));
        z = x.clone();
        z += &y;
        assert_eq!(z, w);

        <$t>::sub(&mut z, &x, &y);
        w.set_int(a - b);
        assert_eq!(z, w);
        assert_eq!(w, (&x - &y));
        z = x.clone();
        z -= &y;
        assert_eq!(z, w);

        <$t>::mul(&mut z, &x, &y);
        w.set_int(a * b);
        assert_eq!(z, w);
        assert_eq!(w, (&x * &y));
        z = x.clone();
        z *= &y;
        assert_eq!(z, w);

        <$t>::div(&mut z, &x, &y);
        w.set_int(a / b);
        assert_eq!(z, w);
        assert_eq!(z, (&x / &y));
        z = x.clone();
        z /= &y;
        assert_eq!(z, w);

        assert!(x.set_little_endian_mod(&[1, 2, 3, 4, 5]));
        assert_eq!(x.get_str(16), "504030201");
        <$t>::sqr(&mut y, &x);
        <$t>::mul(&mut z, &x, &x);
        assert_eq!(y, z);

        assert!(<$t>::square_root(&mut w, &y));
        if w != x {
            <$t>::neg(&mut z, &w);
            assert_eq!(x, z);
        }
    }};
}

macro_rules! ec_test {
    ($t:ty, $f:ty, $p:expr) => {
        assert!($p.is_valid());
        assert!(!$p.is_zero());
        let mut p1 = <$t>::zero();
        assert!(p1.is_zero());
        assert_ne!(p1, $p);
        <$t>::neg(&mut p1, &$p);
        let mut x: $f = unsafe { <$f>::uninit() };
        <$f>::neg(&mut x, &p1.y);
        assert_eq!(&x, &$p.y);

        <$t>::dbl(&mut p1, &$p);
        let mut p2: $t = unsafe { <$t>::uninit() };
        let mut p3: $t = unsafe { <$t>::uninit() };
        <$t>::add(&mut p2, &$p, &$p);
        assert_eq!(p2, p1);
        <$t>::add(&mut p3, &p2, &$p);
        assert_eq!(p3, (&p2 + &$p));
        assert_eq!(p2, (&p3 - &$p));
        let mut y: Fr = Fr::from_int(1);
        <$t>::mul(&mut p2, &$p, &y);
        assert_eq!(p2, $p);
        y.set_int(2);
        <$t>::mul(&mut p2, &$p, &y);
        assert_eq!(p2, p1);
        y.set_int(3);
        <$t>::mul(&mut p2, &$p, &y);
        assert_eq!(p2, p3);
        p2 = p1.clone();
        p2 += &$p;
        assert_eq!(p2, p3);

        p2 -= &$p;
        assert_eq!(p2, p1);
    };
}

macro_rules! serialize_test {
    ($t:ty, $x:expr) => {
        let buf = $x.serialize();
        let mut y: $t = unsafe { <$t>::uninit() };
        assert!(y.deserialize(&buf));
        assert_eq!($x, y);
    };
}