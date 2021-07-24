#![feature(fn_traits)]

use std::fs::File;
use std::time::Instant;

use lazy_static::lazy_static;
use rand::{Rng, SeedableRng};
use rand::prelude::StdRng;

use grid::{Offset, Point, Update};
use std::ops::Range;

lazy_static! {
    static ref MOORE_NEIGHBORS: Vec<Offset> = {
        let mut n = Vec::new();
        n.push(Offset::new(-1, -1));
        n.push(Offset::new(-1, 0));
        n.push(Offset::new(-1, 1));
        n.push(Offset::new(0, -1));
        n.push(Offset::new(0, 1));
        n.push(Offset::new(1, -1));
        n.push(Offset::new(1, 0));
        n.push(Offset::new(1, 1));
        n
    };
}

fn bench_divs() {
    let mut r = StdRng::seed_from_u64(2);

    for i in 0..100 {
        let l = r.gen_range::<isize, Range<isize>>(-100..100);
        let r = r.gen_range::<isize, Range<isize>>(-100..100);

        if l == 0 || r == 0 { continue }

        let d = grid::div_neg_isize_3(l, r);

        println!("{} {} -> {}", l, r, d);
    }

    for i in 0..10000000 {
        let x = i * i;
    }


    let s1 = Instant::now();
    for i in 0..100000000 {
        let l = r.gen_range::<isize, Range<isize>>(-100..100);
        let r = r.gen_range::<isize, Range<isize>>(-100..100);

        if l == 0 || r == 0 { continue }

        let d = grid::div_neg_isize(l, r);
    }
    println!("{:?}", s1.elapsed());


    let s2 = Instant::now();
    for i in 0..100000000 {
        let l = r.gen_range::<isize, Range<isize>>(-100..100);
        let r = r.gen_range::<isize, Range<isize>>(-100..100);

        if l == 0 || r == 0 { continue }

        let d = grid::div_neg_isize_2(l, r);
    }
    println!("{:?}", s2.elapsed());

    let s3 = Instant::now();
    for i in 0..100000000 {
        let l = r.gen_range::<isize, Range<isize>>(-100..100);
        let r = r.gen_range::<isize, Range<isize>>(-100..100);

        if l == 0 || r == 0 { continue }

        let d = grid::div_neg_isize_3(l, r);
    }
    println!("{:?}", s3.elapsed());
}

fn main() {
    // let p = grid::Point::new(0, 0);
    //
    // println!("Von Neumann neighbors");
    // for i in 1..=3 {
    //     println!("{}:", i);
    //     for n in p.von_neumann_neighbors(i, false) {
    //         println!("\t{}", n);
    //     }
    //     println!("\t-----");
    //     for n in p.von_neumann_neighbors(i, true) {
    //         println!("\t{}", n);
    //     }
    //     println!("");
    // }
    //
    // println!("Moore neighbors");
    // for i in 1..=3 {
    //     println!("{}:", i);
    //     for n in p.moore_neighbors(i, false) {
    //         println!("\t{}", n);
    //     }
    //     println!("\t-----");
    //     for n in p.moore_neighbors(i, true) {
    //         println!("\t{}", n);
    //     }
    //     println!("");
    // }

    //

    let mut r = StdRng::seed_from_u64(2);

    //

    let mut grid: grid::Grid<usize, 32> = grid::Grid::new();
    grid.set(&grid::Point::new(0, 0), 0);

    for _ in 0..250 {
        let x: isize = r.gen_range(-7..8);
        let z: isize = r.gen_range(-7..8);

        grid.set(&grid::Point::new(x, z), 1);
    }

    //

    grid.print(|v| v != &0);

    // return;

    let start = Instant::now();

    for i in 0..2000 {
        grid.tick(|point| {
            &MOORE_NEIGHBORS
        }, |point, cur, neighbors| {
            let mut r = Vec::with_capacity(1);

            let mut live = neighbors.into_iter().filter(|v| v == &Some(&1)).count();

            if cur.is_none() || cur == Some(&0) {
                if live == 3 {
                    r.push(Update::new(point.copy(), |_| Some(1)));
                }
            } else {
                if live < 2 || live > 3 {
                    r.push(Update::new(point.copy(), |_| Some(0)));
                }
            }

            r
        });

        // println!("{}", i);

        //
        println!("{}", i);

        // if i % 99 == 0 {
        //     println!("{}", i);
        // grid.print(|v| v != &0);
        // }
    }

    // std::fs::write("out.txt", grid.print(|v| v == &1));

    println!("{:?}", start.elapsed());
}
