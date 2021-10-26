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
    ($t:ty, $f:ty, $P:expr) => {
        #[allow(non_snake_case)]
        assert!($P.is_valid());
        assert!(!$P.is_zero());
        let mut P1 = <$t>::zero();
        assert!(P1.is_zero());
        assert_ne!(P1, $P);
        <$t>::neg(&mut P1, &$P);
        let mut x: $f = unsafe { <$f>::uninit() };
        <$f>::neg(&mut x, &P1.y);
        assert_eq!(&x, &$P.y);

        <$t>::dbl(&mut P1, &$P);
        let mut P2: $t = unsafe { <$t>::uninit() };
        let mut P3: $t = unsafe { <$t>::uninit() };
        <$t>::add(&mut P2, &$P, &$P);
        assert_eq!(P2, P1);
        <$t>::add(&mut P3, &P2, &$P);
        assert_eq!(P3, (&P2 + &$P));
        assert_eq!(P2, (&P3 - &$P));
        let mut y: Fr = Fr::from_int(1);
        <$t>::mul(&mut P2, &$P, &y);
        assert_eq!(P2, $P);
        y.set_int(2);
        <$t>::mul(&mut P2, &$P, &y);
        assert_eq!(P2, P1);
        y.set_int(3);
        <$t>::mul(&mut P2, &$P, &y);
        assert_eq!(P2, P3);
        P2 = P1.clone();
        P2 += &$P;
        assert_eq!(P2, P3);

        P2 -= &$P;
        assert_eq!(P2, P1);
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