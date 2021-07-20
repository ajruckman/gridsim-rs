#![feature(fn_traits)]

use std::fs::File;
use std::time::Instant;

use lazy_static::lazy_static;
use rand::{Rng, SeedableRng};
use rand::prelude::StdRng;

use crate::grid2::{Offset, Point, Update};

mod grid;
mod grid2;

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

fn main() {
    let p = grid::Point::new(0, 0);

    println!("Von Neumann neighbors");
    for i in 1..=3 {
        println!("{}:", i);
        for n in p.von_neumann_neighbors(i, false) {
            println!("\t{}", n);
        }
        println!("\t-----");
        for n in p.von_neumann_neighbors(i, true) {
            println!("\t{}", n);
        }
        println!("");
    }

    println!("Moore neighbors");
    for i in 1..=3 {
        println!("{}:", i);
        for n in p.moore_neighbors(i, false) {
            println!("\t{}", n);
        }
        println!("\t-----");
        for n in p.moore_neighbors(i, true) {
            println!("\t{}", n);
        }
        println!("");
    }

    //

    let mut r = StdRng::seed_from_u64(2);

    let mut grid: grid2::Grid<usize, 128> = grid2::Grid::new();
    grid.set(&grid2::Point::new(0, 0), 0);

    for _ in 0..250 {
        let x: isize = r.gen_range(-7..8);
        let z: isize = r.gen_range(-7..8);

        grid.set(&grid2::Point::new(x, z), 1);
    }

    //

    grid.print(|v| v != &0);

    // return;

    let start = Instant::now();

    for i in 0..10000 {
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

    std::fs::write("out.txt", grid.print(|v| v == &1));

    println!("{:?}", start.elapsed());
}
