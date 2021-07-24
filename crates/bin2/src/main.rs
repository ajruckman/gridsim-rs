use grid::grid3::*;
use std::time::{Instant, Duration};
use grid::grid4::{Grid4, Point, TieredGrid};
use std::thread::Thread;
use std::thread;

use peak_alloc::PeakAlloc;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

fn main() {
    let g3: GridL3<u8, 3, 3, 3> = GridL3::init_none();

    // let start = Instant::now();
    // for i in 0..100_000_000 {
    //     ternary::<24>(i);
    // }
    // println!("{:?}", start.elapsed());

    // ternary_2::<2>(14, &[5, 3]);
    // ternary_2::<3>(29, &[3, 5, 3]);
    //
    // println!("{:?}", index_3l::<3, 5, 3>(14));
    // println!("{:?}", index_3l::<3, 5, 3>(29));

    // ternary::<3>(18);
    // ternary::<3>(26);
    // ternary::<3>(27);

    // for i in 0..(37 * 41 * 43) {
    //     println!("{:?}", index_3l::<37, 41, 43>(i));
    // }

    let mut g4: TieredGrid<usize, 16, 24, 32> = TieredGrid::new();
    let l = 16 * 24 * 32;

    let start = Instant::now();

    for x in 0..l {
        for z in 0..l {
            // if z != 0 && (x / z) % 3 == 2 { continue; }
            // if x != 0 && (z / x) % 3 == 2 { continue; }
            g4.set(&Point::new(x, z), 5);
        }
    }
    for x in 0..l {
        for z in 0..l {
            // if z != 0 && (x / z) % 3 == 2 { continue; }
            // if x != 0 && (z / x) % 3 == 2 { continue; }
            g4.set(&Point::new(x, z), 5);
        }
    }

    println!("{:?}", start.elapsed());

    thread::sleep(Duration::from_secs(1));
    println!("{:?}", PEAK_ALLOC.peak_usage_as_gb());

    // for x in 0..l {
    //     print!("|");
    //     for z in 0..l {
    //         match g4.get(&Point::new(x, z)) {
    //             None => print!(" "),
    //             Some(v) => match v {
    //                 0 => print!(" "),
    //                 _ => print!("*"),
    //             }
    //         }
    //     }
    //     println!("|");
    // }

    // for z in 0..(83 * 89 * 97) {
    //     println!("{}", z);
    // g4.set(&Point::new(z, z), 5);
    // }

    // for z in 0..(83 * 89 * 97) {
    //     g4.set(&Point::new(z, z), 5);
    // }


    // thread::sleep(Duration::from_secs(100));

    return;

    Index::new(0, 0).to_l3::<3, 3, 3>();
    println!();

    Index::new(2, 2).to_l3::<3, 3, 3>();
    println!();

    Index::new(3, 3).to_l3::<3, 3, 3>();
    println!();

    Index::new(5, 5).to_l3::<3, 3, 3>();
    println!();

    Index::new(6, 6).to_l3::<3, 3, 3>();
    println!();

    Index::new(8, 8).to_l3::<3, 3, 3>();
    println!();

    Index::new(9, 9).to_l3::<3, 3, 3>();
    println!();
}

fn index_3l<const L1: usize, const L2: usize, const L3: usize>(x: usize) -> (usize, usize, usize) {
    let mut x = x;

    let i3 = x % L3;
    x /= L3;

    let i2 = x % L2;
    x /= L2;

    let i1 = x % L1;
    x /= L1;

    if x != 0 {
        panic!("attempted to create index for a number not representable in [{},{},{}] leveling", L1, L2, L3);
    }

    (i1, i2, i3)
}

fn ternary<const L: usize>(x: usize) -> [usize; L] {
    let mut x = x.clone();
    let mut r: [usize; L] = [0; L];

    let mut i = L;
    while x != 0 {
        i -= 1;
        r[i] = x % 3;
        x /= 3;
    }

    // println!("{:?}", r);

    r
}

fn ternary_2<const L: usize>(x: usize, lengths: &[usize; L]) -> [usize; L] {
    let mut x = x.clone();
    let mut r: [usize; L] = [0; L];

    for i in (0..L).rev() {
        let m = lengths[i];
        r[i] = x % m;
        x /= m;

        if x == 0 { break; }
    }

    println!("{:?}", r);

    r
}

fn div_rem(n: usize, d: usize) -> (usize, usize) {
    let q = n / d;
    let r = n % d;
    (q, r)
}
