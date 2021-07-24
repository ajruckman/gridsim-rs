use std::mem::MaybeUninit;

pub struct GridL1<T, const L1: usize> where T: Default {
    values: [[Option<T>; L1]; L1],
}

impl<T, const L1: usize> GridL1<T, L1> where T: Default {
    pub fn init_none() -> Self {
        Self {
            values: allocate_2d(),
        }
    }
}

pub struct GridL2<T, const L1: usize, const L2: usize> where T: Default {
    values: [[Option<GridL1<T, L1>>; L2]; L2],
}

impl<T, const L1: usize, const L2: usize> GridL2<T, L1, L2> where T: Default {
    pub fn init_none() -> Self {
        Self {
            values: allocate_2d(),
        }
    }
}

pub struct GridL3<T, const L1: usize, const L2: usize, const L3: usize> where T: Default {
    values: [[Option<GridL2<T, L1, L2>>; L3]; L3],
}

impl<T, const L1: usize, const L2: usize, const L3: usize> GridL3<T, L1, L2, L3> where T: Default {
    pub fn init_none() -> Self {
        Self {
            values: allocate_2d(),
        }
    }
}

//

pub struct Index {
    x: usize,
    z: usize,
}

impl Index {
    pub fn new(x: usize, z: usize) -> Self {
        Self { x, z }
    }

    pub fn to_l3<const L1: usize, const L2: usize, const L3: usize>(&self) {
        let (qx, rx) = div_rem(self.x, L3 * L2 * L1);
        let (qz, rz) = div_rem(self.z, L3 * L2 * L1);

        // let r = L3Index::<L1, L2, L3>::new(qx, qz, rx, rx);
        // let r = ((qx, qz), L3Index::<L1, L2>::new(rx, rz));

        println!("({},{})", qx, rx);
        println!("({},{})", qz, rz);
    }
}

pub struct L1Index<const L1: usize> {
    x: usize,
    z: usize,
}

pub struct L2Index<const L1: usize, const L2: usize> {
    x: usize,
    z: usize,
}

pub struct L3Index<const L1: usize, const L2: usize, const L3: usize> {
    x: usize,
    z: usize,
}

impl<const L1: usize> L1Index<L1> {
    pub fn new(x: usize, z: usize) -> Self {
        Self { x, z }
    }
}

impl<const L1: usize, const L2: usize> L2Index<L1, L2> {
    pub fn new(x: usize, z: usize) -> Self {
        Self { x, z }
    }

    pub fn to_l1(&self) /*-> ((usize, usize), L2Index<L1, L2>)*/ {
        let (qx, rx) = div_rem(self.x, L1);
        let (qz, rz) = div_rem(self.z, L1);

        let r = ((qx, qz), L1Index::<L1>::new(rx, rz));
    }
}

impl<const L1: usize, const L2: usize, const L3: usize> L3Index<L1, L2, L3> {
    pub fn new(x: usize, z: usize) -> Self {
        Self { x, z }
    }

    pub fn to_l2(&self) {
        let (qx, rx) = div_rem(self.x, L3);
        let (qz, rz) = div_rem(self.z, L3);

        let r = ((qx, qz), L2Index::<L1, L2>::new(rx, rz));

        println!("({},{})", qx, rx);
        println!("({},{})", qz, rz);
    }
}

//

fn allocate_2d<T, const L: usize>() -> [[Option<T>; L]; L] {
    let mut values: [[MaybeUninit<Option<T>>; L]; L] = unsafe {
        MaybeUninit::uninit().assume_init()
    };

    unsafe {
        for col in &mut values[..] {
            let mut row: [MaybeUninit<Option<T>>; L] = MaybeUninit::uninit().assume_init();

            for col in &mut row[..] {
                *col = MaybeUninit::new(None);
            }

            *col = row;
        }
    }

    unsafe { std::mem::transmute_copy(&values) }
}

fn div_rem(n: usize, d: usize) -> (usize, usize) {
    let q = n / d;
    let r = n % d;
    (q, r)
}
