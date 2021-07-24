pub mod grid3;
pub mod grid4;

use std::collections::hash_map::Entry;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Index;

use lru::LruCache;
use rustc_hash::{FxHashMap, FxHashSet};

pub struct Point {
    x: isize,
    z: isize,
}

impl Point {
    pub fn new(x: isize, z: isize) -> Self {
        Self { x, z }
    }

    pub fn copy(&self) -> Self { Self { x: self.x, z: self.z } }

    fn to_subgrid_index(&self, l_i: isize) -> SubGridIndex {
        SubGridIndex::new(div_neg_isize_3(self.x, l_i), div_neg_isize_3(self.z, l_i))
    }

    fn to_subgrid_point(&self, l_i: isize) -> SubGridPoint {
        SubGridPoint::new(self.x.rem_euclid(l_i) as usize, self.z.rem_euclid(l_i) as usize)
    }

    fn shift(&self, o: &Offset) -> Self {
        Self {
            x: self.x + o.x,
            z: self.z + o.z,
        }
    }

    fn is_in_range(&self, start: &Point, end: &Point) -> bool {
        self.x >= start.x && self.z >= start.z && self.x <= end.x && self.z <= end.z
    }

    pub fn moore_neighbors(&self, dist: u8, inclusive: bool) -> Vec<Point> {
        let mut r = Vec::new();

        if inclusive {
            r.push(Point { x: self.x, z: self.z });
        }

        for d in 1..(dist as isize) + 1 {
            let mut d_x = self.x - d;
            let mut d_y = self.z - d;

            for _ in 0..d * 2 {
                r.push(Point::new(d_x, d_y));
                d_x += 1;
            }

            for _ in 0..d * 2 {
                r.push(Point::new(d_x, d_y));
                d_y += 1;
            }

            for _ in 0..d * 2 {
                r.push(Point::new(d_x, d_y));
                d_x -= 1;
            }

            for _ in 0..d * 2 {
                r.push(Point::new(d_x, d_y));
                d_y -= 1;
            }
        }

        r
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{}]", self.x, self.z)
    }
}

//

pub struct Offset {
    x: isize,
    z: isize,
}

impl Offset {
    pub fn new(x: isize, z: isize) -> Self {
        Self { x, z }
    }
}

pub struct Update<T>
    where T: Default + Clone + Display
{
    p: Point,
    f: fn(Option<&T>) -> Option<T>,
}

impl<T> Update<T>
    where T: Default + Clone + Display,
{
    pub fn new(p: Point, f: fn(Option<&T>) -> Option<T>) -> Self {
        Self {
            p,
            f,
        }
    }
}

//

pub struct Grid<T, const L: usize> where T: Default + Clone + Display {
    values: FxHashMap<SubGridIndex, SubGrid<T, L>>,

    to_scan: Option<Vec<SubGridIndex>>,
    // sub_cache: LruCache<SubGridIndex, &'a SubGrid<T, L>>,
}

impl<'a, T, const L: usize> Grid<T, L> where T: Default + Clone + Display {
    pub const L_I: isize = L as isize;

    pub fn new() -> Grid<T, L> {
        Grid {
            values: FxHashMap::default(),
            to_scan: None,
            // sub_cache: LruCache::new(3),
        }
    }

    //

    pub fn get(&self, p: &Point) -> Option<&T> {
        match self.get_subgrid(p) {
            None => None,
            Some(sub) => {
                Some(sub.get(&p.to_subgrid_point(Grid::<T, L>::L_I)))
            }
        }
    }

    pub fn set(&mut self, p: &Point, v: T) {
        let sub = self.get_subgrid_or_expand(p);

        sub.set(&p.to_subgrid_point(Grid::<T, L>::L_I), v);
    }

    pub fn tick<FVisit: Fn(&Point) -> &Vec<Offset>, FUpdate: Fn(&Point, Option<&T>, Vec<Option<&T>>) -> Vec<Update<T>>>(
        &mut self,
        visitor: FVisit,
        updater: FUpdate,
    ) {
        let mut updates = Vec::new();

        // let mut hit = 0;
        // let mut miss = 0;

        for sub_index in self.subgrids_to_scan() {
            let sub = self.values.get(&sub_index);
            // println!("{}", sub.is_some());

            let start = Point::new(sub_index.x * Grid::<T, L>::L_I, sub_index.z * Grid::<T, L>::L_I);
            let end = Point::new(sub_index.x * Grid::<T, L>::L_I + Grid::<T, L>::L_I, sub_index.z * Grid::<T, L>::L_I + Grid::<T, L>::L_I);

            for x in 0..Grid::<T, L>::L_I {
                for z in 0..Grid::<T, L>::L_I {
                    let point = Point::new(start.x + x, start.z + z);

                    //

                    let neighbor_offsets = visitor(&point);
                    let mut neighbor_values = Vec::new();
                    for neighbor_offset in neighbor_offsets {
                        let neighbor_point = point.shift(neighbor_offset);

                        // println!("{} in {},{} <{}> ? {}", neighbor_point, start, end, sub.is_some(), neighbor_point.is_in_range(&start, &end));

                        let value = match neighbor_point.is_in_range(&start, &end) {
                            false => {
                                // miss += 1;
                                self.get(&neighbor_point)
                            }
                            true => {
                                match sub {
                                    None => {
                                        // miss += 1;
                                        self.get(&neighbor_point)
                                    }
                                    Some(v) => {
                                        // hit += 1;
                                        Some(self.get_in_known_subgrid(v, &neighbor_point))
                                    }
                                }
                            }
                        };

                        neighbor_values.push(value);
                    }

                    //

                    let value = match sub {
                        None => None,
                        Some(v) => {
                            Some(v.get(&point.to_subgrid_point(Grid::<T, L>::L_I)))
                        }
                    };

                    updates.append(&mut updater(&point, value, neighbor_values));
                }
            }
        }

        for update in &updates {
            let old = self.get(&update.p);
            let new = (update.f)(old);

            // if new.is_none() {
            //     println!("UNSET: {}", update.p);
            // } else {
            //     println!("SET V: {} => {}", update.p, new.as_ref().unwrap());
            // }

            self.set(&update.p, new.unwrap_or_default());
        }

        // println!("Hit: {} | Miss: {}", hit, miss);
    }

    /*

    -5 -> 7
    Range: 12

    -5, 12 =

    */

    pub fn print<FShouldDisplay: Fn(&T) -> bool>(&mut self, should_display: FShouldDisplay) -> String {
        let (min_sub, max_sub) = self.find_subgrid_index_bounds();
        let (min, max) = self.find_grid_point_bounds();

        // println!("Min sub: {},{}", min_sub.x, min_sub.z);
        // println!("Max sub: {},{}", max_sub.x, max_sub.z);
        // println!("Min p: {},{}", min.x, min.z);
        // println!("Max p: {},{}", max.x, max.z);

        let len_x = (max.x - min.x) as usize;
        let len_z = (max.z - min.z) as usize;

        let mut rows = Vec::<String>::with_capacity(len_z);
        for _ in 0..len_z {
            rows.push(" ".repeat(len_x));
        }

        for (sub_index, sub) in &self.values {
            let sub_abs_x = sub_index.x * Grid::<T, L>::L_I;
            let sub_abs_z = sub_index.z * Grid::<T, L>::L_I;
            let sub_rel_x = (sub_abs_x - min.x) as usize;
            let sub_rel_z = (sub_abs_z - min.z) as usize;

            // println!("Sub index: {} {} | Sub abs: {} {} | Rel: {} {}", sub_index.x, sub_index.z, sub_abs_x, sub_abs_z, sub_rel_x, sub_rel_z);

            for x in 0..L {
                let this_rel_x = sub_rel_x + x;

                for z in 0..L {
                    let point = SubGridPoint::new(x, z);
                    let value = sub.get(&point);

                    if should_display(value) {
                        let this_rel_z = sub_rel_z + z;

                        rows[this_rel_z].replace_range(this_rel_x..=this_rel_x, "#");
                    }
                }
            }
        }

        let mut p = String::new();

        // p.push_str(&format!("+{}+\n", "-".repeat(len_x)));
        // for row in &rows {
        //     p.push_str(&format!("|{}|\n", row));
        // }
        // p.push_str(&format!("+{}+", "-".repeat(len_x)));
        // p.push_str(&format!("{}, {}", len_x, len_z));

        println!("+{}+", "-".repeat(len_x));
        for row in &rows {
            println!("|{}|", row);
        }
        println!("+{}+", "-".repeat(len_x));
        println!("{}, {}", len_x, len_z);

        p
    }

    //

    fn invalidate_subgrids_to_scan(&mut self) {
        self.to_scan = None;
    }

    fn subgrids_to_scan(&mut self) -> Vec<SubGridIndex> {
        match &self.to_scan {
            None => {
                let mut checked_neighbors = FxHashSet::default();
                let mut to_scan = Vec::new();

                for (sub_index, sub) in &self.values {
                    to_scan.push(sub_index.copy());

                    for neighbor in sub_index.moore_neighbors(1, false) {
                        if self.values.contains_key(&neighbor) || checked_neighbors.contains(&neighbor) {
                            continue;
                        }

                        checked_neighbors.insert(neighbor.copy());
                        to_scan.push(neighbor.copy());
                    }
                }

                self.to_scan = Some(to_scan.to_vec());

                to_scan
            }
            Some(v) => v.to_vec()
        }
    }

    fn get_subgrid(&self, p: &Point) -> Option<&SubGrid<T, L>> {
        let index = p.to_subgrid_index(Grid::<T, L>::L_I);
        self.values.get(&index)
    }

    fn get_subgrid_or_expand(&mut self, p: &Point) -> &mut SubGrid<T, L> {
        let mut changed = false;

        let r = self.values.entry(p.to_subgrid_index(Grid::<T, L>::L_I)).or_insert_with(|| {
            changed = true;
            SubGrid::new()
        });

        if changed {
            self.to_scan = None;
        }

        r
    }

    fn get_in_known_subgrid<'b, 'c>(&self, sub: &'b SubGrid<T, L>, p: &'c Point) -> &'b T {
        sub.get(&p.to_subgrid_point(Grid::<T, L>::L_I))
    }

    //

    fn find_subgrid_index_bounds(&self) -> (SubGridIndex, SubGridIndex) {
        let mut min_sub_x = isize::MAX;
        let mut min_sub_z = isize::MAX;
        let mut max_sub_x = isize::MIN;
        let mut max_sub_z = isize::MIN;

        for (p, _) in &self.values {
            if p.x < min_sub_x { min_sub_x = p.x }
            if p.z < min_sub_z { min_sub_z = p.z }
            if p.x > max_sub_x { max_sub_x = p.x }
            if p.z > max_sub_z { max_sub_z = p.z }
        }

        (SubGridIndex::new(min_sub_x, min_sub_z), SubGridIndex::new(max_sub_x, max_sub_z))
    }

    fn find_grid_point_bounds(&self) -> (Point, Point) {
        let (min_sub, max_sub) = self.find_subgrid_index_bounds();

        let min_x = min_sub.x * Grid::<T, L>::L_I;
        let min_z = min_sub.z * Grid::<T, L>::L_I;
        let max_x = max_sub.x * Grid::<T, L>::L_I + Grid::<T, L>::L_I; // TODO: minus 1?
        let max_z = max_sub.z * Grid::<T, L>::L_I + Grid::<T, L>::L_I; // TODO: minus 1?

        (Point::new(min_x, min_z), Point::new(max_x, max_z))
    }
}

//

#[derive(PartialEq, Eq, Hash, Clone)]
struct SubGridIndex {
    x: isize,
    z: isize,
}

impl SubGridIndex {
    pub fn new(x: isize, z: isize) -> Self {
        Self { x, z }
    }

    pub fn copy(&self) -> Self {
        Self { x: self.x, z: self.z }
    }

    pub fn moore_neighbors(&self, dist: u8, inclusive: bool) -> Vec<SubGridIndex> {
        let mut r = Vec::new();

        if inclusive {
            r.push(SubGridIndex { x: self.x, z: self.z });
        }

        for d in 1..(dist as isize) + 1 {
            let mut d_x = self.x - d;
            let mut d_y = self.z - d;

            for _ in 0..d * 2 {
                r.push(SubGridIndex::new(d_x, d_y));
                d_x += 1;
            }

            for _ in 0..d * 2 {
                r.push(SubGridIndex::new(d_x, d_y));
                d_y += 1;
            }

            for _ in 0..d * 2 {
                r.push(SubGridIndex::new(d_x, d_y));
                d_x -= 1;
            }

            for _ in 0..d * 2 {
                r.push(SubGridIndex::new(d_x, d_y));
                d_y -= 1;
            }
        }

        r
    }
}

struct SubGridPoint {
    x: usize,
    z: usize,
}

impl SubGridPoint {
    pub fn new(x: usize, z: usize) -> Self {
        Self { x, z }
    }
}

struct SubGrid<T, const L: usize> where T: Default + Clone {
    values: [[T; L]; L],
}

impl<T, const L: usize> SubGrid<T, L> where T: Default + Clone {
    pub const L_I: isize = L as isize;

    fn new() -> SubGrid<T, L> {
        SubGrid {
            values: allocate_2d(),
        }
    }

    fn get(&self, p: &SubGridPoint) -> &T {
        &self.values[p.x][p.z]
    }

    fn set(&mut self, p: &SubGridPoint, v: T) {
        self.values[p.x][p.z] = v;
    }
}

//

// https://stackoverflow.com/a/3042066/9911189
pub fn div_neg_isize(a: isize, b: isize) -> isize {
    if a >= 0 {
        a / b
    } else {
        (a - b + 1) / b
    }
}

pub fn div_neg_isize_2(a: isize, b: isize) -> isize {
    (a - (((a % b) + b) % b)) / b
}

pub fn div_neg_isize_3(a: isize, b: isize) -> isize {
    (a / b) + ((a % b) >> 31)
}

fn allocate_2d<T, const L: usize>() -> [[T; L]; L] where T: Default {
    let mut values: [[MaybeUninit<T>; L]; L] = unsafe {
        MaybeUninit::uninit().assume_init()
    };

    unsafe {
        for col in &mut values[..] {
            let mut row: [MaybeUninit<T>; L] = MaybeUninit::uninit().assume_init();

            for col in &mut row[..] {
                *col = MaybeUninit::new(T::default());
            }

            *col = row;
        }
    }

    unsafe { std::mem::transmute_copy(&values) }
}
