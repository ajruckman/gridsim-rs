use std::fmt::Display;
use std::mem::MaybeUninit;

use slab::Slab;

pub struct IndexedGrid<T, const L: usize> {
    values: Vec<T>,
    values_counter: usize,
    map: [[Option<usize>; L]; L],
}

const DEBUG: bool = false;

impl<T, const L: usize> IndexedGrid<T, L> {
    pub fn init() -> Self {
        Self {
            values: Vec::with_capacity(L * L),
            values_counter: 0,
            map: allocate_2dg(),
        }
    }

    pub fn get(&self, p: &Point) -> Option<&T> {
        if DEBUG {
            if p.x > L || p.z > L {
                panic!("attempted to get at index outside of grid");
            }
        }

        match self.map[p.x][p.z] {
            None => None,
            Some(i) => Some(unsafe { self.values.get_unchecked(i) }),
        }
    }

    pub fn get_mut(&mut self, p: &Point) -> Option<&mut T> {
        if DEBUG {
            if p.x > L || p.z > L {
                panic!("attempted to get at index outside of grid");
            }
        }

        match self.map[p.x][p.z] {
            None => None,
            Some(i) => Some(unsafe { self.values.get_unchecked_mut(i) }),
        }
    }

    pub fn get_or_init<FInit: Fn() -> T>(&mut self, p: &Point, init: FInit) -> &mut T {
        if DEBUG {
            if p.x > L || p.z > L {
                panic!("attempted to get at index outside of grid");
            }
        }

        let i = match self.map[p.x][p.z] {
            None => {
                if DEBUG {
                    println!("MISS: {}/{}", p.x, p.z);
                }

                let g = init();

                self.values.insert(self.values_counter, g);
                self.map[p.x][p.z] = Some(self.values_counter);

                let i = self.values_counter;
                self.values_counter += 1;

                i
            }
            Some(i) => i,
        };

        unsafe { self.values.get_unchecked_mut(i) }
    }

    pub fn set(&mut self, p: &Point, v: T) {
        if DEBUG {
            if p.x > L || p.z > L {
                panic!("attempted to get at index outside of grid");
            }
        }

        match self.map[p.x][p.z] {
            None => {
                self.values.insert(self.values_counter, v);
                self.map[p.x][p.z] = Some(self.values_counter);
                self.values_counter += 1;
            }
            Some(i) => {
                self.values[i] = v;
            }
        };
    }

    pub fn set_or_init<FInit: Fn() -> T>(&mut self, p: &Point, init: FInit, v: T) {
        *self.get_or_init(p, init) = v;
    }
}

//

fn allocate_grid3<T, const L1: usize, const L2: usize, const L3: usize>() -> Grid3<T, L1, L2, L3> {
    let mut values: [[MaybeUninit<Option<Grid2<T, L1, L2>>>; L3]; L3] = unsafe {
        MaybeUninit::uninit().assume_init()
    };
    unsafe {
        for col in &mut values[..] {
            let mut row: [MaybeUninit<Option<Grid2<T, L1, L2>>>; L3] = MaybeUninit::uninit().assume_init();
            for col in &mut row[..] {
                *col = MaybeUninit::new(None);
            }
            *col = row;
        }
    }
    unsafe { std::mem::transmute_copy(&values) }
}

fn allocate_grid2<T, const L1: usize, const L2: usize>() -> Grid2<T, L1, L2> {
    let mut values: [[MaybeUninit<Option<Grid1<T, L1>>>; L2]; L2] = unsafe {
        MaybeUninit::uninit().assume_init()
    };
    unsafe {
        for col in &mut values[..] {
            let mut row: [MaybeUninit<Option<Grid1<T, L1>>>; L2] = MaybeUninit::uninit().assume_init();
            for col in &mut row[..] {
                *col = MaybeUninit::new(None);
            }
            *col = row;
        }
    }
    unsafe { std::mem::transmute_copy(&values) }
}

fn allocate_grid1<T, const L1: usize>() -> Grid1<T, L1> {
    let mut values: [[MaybeUninit<Option<T>>; L1]; L1] = unsafe {
        MaybeUninit::uninit().assume_init()
    };
    unsafe {
        for col in &mut values[..] {
            let mut row: [MaybeUninit<Option<T>>; L1] = MaybeUninit::uninit().assume_init();
            for col in &mut row[..] {
                *col = MaybeUninit::new(None);
            }
            *col = row;
        }
    }
    unsafe { std::mem::transmute_copy(&values) }
}

pub type Grid3<T, const L1: usize, const L2: usize, const L3: usize> = [[Option<Grid2<T, L1, L2>>; L3]; L3];
pub type Grid2<T, const L1: usize, const L2: usize> = [[Option<Grid1<T, L1>>; L2]; L2];
pub type Grid1<T, const L1: usize> = [[Option<T>; L1]; L1];

pub struct TieredGrid<T, const L1: usize, const L2: usize, const L3: usize> {
    top: IndexedGrid<IndexedGrid<IndexedGrid<T, L3>, L2>, L1>,

    // values_3: [[Option<usize>; L3]; L3],
    // grids_3: Vec<Grid3<T, L1, L2, L3>>,
    // grids_3_i: usize,
    //
    // values_2: [[Option<usize>; L2]; L2],
    // grids_2: Vec<Grid2<T, L1, L2>>,
    // grids_2_i: usize,
    //
    // values_1: [[Option<usize>; L1]; L1],
    // grids_1: Vec<Grid1<T, L1>>,
    // grids_1_i: usize,
}

impl<T, const L1: usize, const L2: usize, const L3: usize> TieredGrid<T, L1, L2, L3> {
    pub fn new() -> Self {
        Self {
            top: IndexedGrid::init(),
        }
    }

    pub fn get(&mut self, p: &Point) -> Option<&T> {
        let (xi3, xi2, xi1) = index_3l::<L1, L2, L3>(p.x);
        let (zi3, zi2, zi1) = index_3l::<L1, L2, L3>(p.z);

        match self.top.get(&Point::new(xi3, zi3)) {
            None => None,
            Some(v) => match v.get(&Point::new(xi2, zi2)) {
                None => None,
                Some(v) => v.get(&Point::new(xi1, zi1)),
            }
        }
    }

    pub fn set(&mut self, p: &Point, v: T) {
        let (xi3, xi2, xi1) = index_3l::<L1, L2, L3>(p.x);
        let (zi3, zi2, zi1) = index_3l::<L1, L2, L3>(p.z);

        // println!("{}/{} {}/{} {}/{}", xi3, zi3, xi2, zi2, xi1, zi1);

        let g3 = self.top.get_or_init(&Point::new(xi3, zi3), || IndexedGrid::init());
        let g2 = g3.get_or_init(&Point::new(xi2, zi2), || IndexedGrid::init());

        g2.set(&Point::new(xi1, zi1), v);
    }

    // WORKS:
    // pub fn set(&mut self, p: Point, v: T) {
    //     let (xi3, xi2, xi1) = index_3l::<L1, L2, L3>(p.x);
    //     let (zi3, zi2, zi1) = index_3l::<L1, L2, L3>(p.z);
    //
    //     let i3 = match &self.values_3[xi3][zi3] {
    //         None => {
    //             let g = allocate_grid3();
    //
    //             self.grids_3.insert(self.grids_3_i, g);
    //             self.values_3[xi3][zi3] = Some(self.grids_3_i);
    //
    //             let i = self.grids_3_i;
    //             self.grids_3_i += 1;
    //
    //             i
    //         }
    //         Some(v) => *v,
    //     };
    //     let g3 = unsafe { self.grids_3.get_unchecked_mut(i3) };
    //
    //     // let i2 = match &self.values_2[xi2][zi2] {
    //     //     None => {
    //     //         let mut g = allocate_grid2();
    //     //
    //     //         self.grids_2.insert(self.grids_2_i, g);
    //     //         g3[xi2][zi2] = Some(self.grids_2_i);
    //     //
    //     //         let i = self.grids_2_i;
    //     //         self.grids_2_i += 1;
    //     //
    //     //         i
    //     //     }
    //     //     Some(v) => *v,
    //     // };
    //     //
    //     // let i2 = &g3[xi2][zi2];
    //     //
    //     // match i2 {};
    // }
}

//

pub struct Point {
    x: usize,
    z: usize,
}

impl Point {
    pub fn new(x: usize, z: usize) -> Self {
        Self { x, z }
    }
}

pub struct Grid4<T, const L1: usize, const L2: usize, const L3: usize> where T: Display {
    // values: [[Option<[[Option<[[Option<T>; L3]; L3]>; L2]; L2]>; L1]; L1],

    grids: Vec<[[Option<T>; L2]; L2]>,
    grids_pos: usize,

    values: [[Option<usize>; L1]; L1],
}

impl<T, const L1: usize, const L2: usize, const L3: usize> Grid4<T, L1, L2, L3> where T: Display {
    pub fn init() -> Self {
        Self {
            grids: Vec::with_capacity(L1 * L1),
            grids_pos: 0,

            values: allocate_2d_index::<usize, L1, L2>(),
        }
    }

    pub fn set(&mut self, p: Point, v: T) {
        let (xi1, xi2, xi3) = index_3l::<L1, L2, L3>(p.x);
        let (zi1, zi2, zi3) = index_3l::<L1, L2, L3>(p.z);

        let i1 = &self.values[xi1][zi1];
        println!("{}", i1.is_some());

        let i = match i1 {
            None => {
                let g = allocate_2d::<T, L1, L2>();

                self.grids.insert(self.grids_pos, g);
                self.values[xi1][zi1] = Some(self.grids_pos);

                let i = self.grids_pos;
                self.grids_pos += 1;

                i

                // self.values[xi1][zi1] = Some(&g);

                // let x = self.grids.insert(g);

                // println!("{:?}", self.grids.get(x));
            }
            Some(i) => *i,
        };

        println!("[{},{}] {} -> {}", p.x, p.z, i, v);

        (unsafe { self.grids.get_unchecked_mut(i) })[xi2][zi2] = Some(v);


        // let r = &unsafe { self.grids.get_unchecked(v) }[xi2][zi2];
    }
}

fn allocate_2dg<'a, T, const L: usize>() -> [[Option<T>; L]; L] where T: Default {
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

fn allocate_2d<'a, T, const L1: usize, const L2: usize>() -> [[Option<T>; L2]; L2] {
    let mut values: [[MaybeUninit<Option<T>>; L2]; L2] = unsafe {
        MaybeUninit::uninit().assume_init()
    };
    unsafe {
        for col in &mut values[..] {
            let mut row: [MaybeUninit<Option<T>>; L2] = MaybeUninit::uninit().assume_init();
            for col in &mut row[..] {
                *col = MaybeUninit::new(None);
            }
            *col = row;
        }
    }
    unsafe { std::mem::transmute_copy(&values) }
}

fn allocate_2d_index<T, const L1: usize, const L2: usize>() -> [[Option<T>; L1]; L1] {
    let mut values: [[MaybeUninit<Option<T>>; L1]; L1] = unsafe {
        MaybeUninit::uninit().assume_init()
    };
    unsafe {
        for col in &mut values[..] {
            let mut row: [MaybeUninit<Option<T>>; L1] = MaybeUninit::uninit().assume_init();
            for col in &mut row[..] {
                *col = MaybeUninit::new(None);
            }
            *col = row;
        }
    }
    unsafe { std::mem::transmute_copy(&values) }
}

// fn allocate_2d3<T, const L1: usize, const L2: usize, const L3: usize>() -> [[Option<[[Option<[[Option<T>; L3]; L3]>; L2]; L2]>; L1]; L1] {
//     let mut values: [[MaybeUninit<Option<[[Option<[[Option<T>; L3]; L3]>; L2]; L2]>>; L1]; L1] = unsafe {
//         MaybeUninit::uninit().assume_init()
//     };
//
//     unsafe {
//         for col in &mut values[..] {
//             let mut row: [MaybeUninit<Option<[[Option<[[Option<T>; L3]; L3]>; L2]; L2]>>; L1] = MaybeUninit::uninit().assume_init();
//
//             for col in &mut row[..] {
//                 *col = MaybeUninit::new(None);
//             }
//
//             *col = row;
//         }
//     }
//
//     unsafe { std::mem::transmute_copy(&values) }
// }

// fn allocate_2d2<'a, T, const L1: usize, const L2: usize>() -> [[Option<&'a [[Option<T>; L2]; L2]>; L1]; L1] {
//     let mut values: [[MaybeUninit<Option<&[[Option<T>; L2]; L2]>>; L1]; L1] = unsafe {
//         MaybeUninit::uninit().assume_init()
//     };
//
//     unsafe {
//         for col in &mut values[..] {
//             let mut row: [MaybeUninit<Option<&[[Option<T>; L2]; L2]>>; L1] = MaybeUninit::uninit().assume_init();
//
//             for col in &mut row[..] {
//                 *col = MaybeUninit::new(None);
//             }
//
//             *col = row;
//         }
//     }
//
//     unsafe { std::mem::transmute_copy(&values) }
// }

fn index_3l<const L3: usize, const L2: usize, const L1: usize>(i: usize) -> (usize, usize, usize) {
    let mut i = i;

    let i3 = i % L3;
    i /= L3;

    let i2 = i % L2;
    i /= L2;

    let i1 = i % L1;

    if DEBUG {
        i /= L1;

        if i != 0 {
            panic!("attempted to create index for a number not representable in [{},{},{}] leveling", L3, L2, L1);
        }
    }

    (i3, i2, i1)
}

