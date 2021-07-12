use rand::{Rng, SeedableRng};
use rand::prelude::StdRng;

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

    let mut grid: grid::Grid<usize, 4> = grid::Grid::new();
    grid.set(&grid::Point::new(0, 0), 0);

    for _ in 0..800 {
        let x: isize = r.gen_range(-64..64);
        let z: isize = r.gen_range(-4..5);

        grid.set(&grid::Point::new(x, z), 1);
    }

    grid.print(|v| v != &0);
}
