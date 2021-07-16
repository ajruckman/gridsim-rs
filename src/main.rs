#![feature(fn_traits)]

use rand::{Rng, SeedableRng};
use rand::prelude::StdRng;
use crate::grid::{Update, Point};
use std::time::Instant;

mod grid;

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

    let mut grid: grid::Grid<usize, 8> = grid::Grid::new();
    grid.set(&grid::Point::new(0, 0), 0);

    for _ in 0..512 {
        let x: isize = r.gen_range(-32..32);
        let z: isize = r.gen_range(-8..8);

        grid.set(&grid::Point::new(x, z), 1);
    }

    //

    grid.print(|v| v != &0);

    let start = Instant::now();

    for i in 0..5000 {
        grid.tick(5, |point| {
            point.moore_neighbors(1, false)
        }, |point, neighbors, old| {
            let mut r = Vec::new();

            let mut live = neighbors.into_iter().filter(|v| v == &Some(&1)).count();

            match old {
                None | Some(0) => {
                    if live == 3 {
                        r.push(Update::new(Point::copy(point), |_| Some(1)));
                    }
                }
                Some(v) => {
                    if live < 2 {
                        r.push(Update::new(Point::copy(point), |_| Some(0)));
                    } else if live > 3 {
                        r.push(Update::new(Point::copy(point), |_| Some(0)));
                    }
                }
            }

            // if value == &0 {
            //
            // } else if value == &1 {
            //     if live < 2 {
            //         new.set(point, 0);
            //     } else if live > 3 {
            //         new.set(point, 0);
            //     }
            // }

            r
        });

        // println!("{}", i);

        //

        // if i % 99 == 0 {
        //     grid.print(|v| v != &0);
        // }
    }

    println!("{:?}", start.elapsed());

}
