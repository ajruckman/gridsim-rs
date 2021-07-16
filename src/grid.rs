use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Index;

pub struct Point {
    x: isize,
    z: isize,
}

impl Point {
    pub fn new(x: isize, z: isize) -> Self {
        Self { x, z }
    }

    pub fn copy(p: &Point) -> Self {
        Self {
            x: p.x,
            z: p.z,
        }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{}]", self.x, self.z)
    }
}

pub struct Offset {
    x: isize,
    z: isize,
}

pub struct GridLength {
    x: usize,
    z: usize,
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

impl GridLength {
    pub fn new(x: usize, z: usize) -> Self {
        Self { x, z }
    }
}

impl Point {
    fn to_subgrid_index(&self, l_i: isize) -> SubGridIndex {
        SubGridIndex::new(div_neg_isize(self.x, l_i), div_neg_isize(self.z, l_i))
    }

    fn to_subgrid_point(&self, l_i: isize) -> SubGridPoint {
        SubGridPoint::new(self.x.rem_euclid(l_i) as usize, self.z.rem_euclid(l_i) as usize)
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

        // for p in &r {
        //     println!("{} -> {}", self, p);
        // }

        r
    }

    pub fn von_neumann_neighbors(&self, dist: u8, inclusive: bool) -> Vec<Point> {
        let mut r = Vec::new();

        if inclusive {
            r.push(Point::new(self.x, self.z));
        }

        for d in 1..(dist as isize) + 1 {
            let mut d_x = self.x - d;
            let mut d_y = self.z;

            for _ in 0..d {
                r.push(Point::new(d_x, d_y));
                d_x += 1;
                d_y += 1;
            }

            for _ in 0..d {
                r.push(Point::new(d_x, d_y));
                d_x += 1;
                d_y -= 1;
            }

            for _ in 0..d {
                r.push(Point::new(d_x, d_y));
                d_x -= 1;
                d_y -= 1;
            }

            for _ in 0..d {
                r.push(Point::new(d_x, d_y));
                d_x -= 1;
                d_y += 1;
            }
        }

        r
    }
}

pub struct Grid<T, const L: usize> where T: Default + Clone + Display {
    values: HashMap<SubGridIndex, SubGrid<T, L>>,
}

impl<T, const L: usize> Grid<T, L> where T: Default + Clone + Display {
    pub const L_I: isize = L as isize;

    pub fn new() -> Grid<T, L> {
        Grid {
            values: HashMap::new(),
        }
    }

    fn clone(old: &HashMap<SubGridIndex, SubGrid<T, L>>) -> Grid<T, L> {
        let mut values = HashMap::new();

        for (k, v) in old {
            let clone = (*v).clone();
            values.insert(SubGridIndex::new(k.x, k.z), clone);
        }

        Grid { values }
    }

    fn find_subgrid_bounds(&self) -> (SubGridIndex, SubGridIndex) {
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

    fn find_grid_length(&self) -> GridLength {
        let (min_sub, max_sub) = self.find_subgrid_bounds();

        let range_x = ((max_sub.x - min_sub.x) as usize) * L + L;
        let range_z = ((max_sub.z - min_sub.z) as usize) * L + L;

        GridLength::new(range_x, range_z)
    }

    fn find_grid_bounds(&self) -> (Point, Point) {
        let (min_sub, max_sub) = self.find_subgrid_bounds();

        let min_x = min_sub.x * Grid::<T, L>::L_I;
        let min_z = min_sub.z * Grid::<T, L>::L_I;
        let max_x = max_sub.x * Grid::<T, L>::L_I;
        let max_z = max_sub.z * Grid::<T, L>::L_I;

        (Point::new(min_x, min_z), Point::new(max_x, max_z))
    }

    pub fn tick<FVisit, FUpdate>(&mut self, pad: isize, visitor: FVisit, updater: FUpdate)
        where FVisit: Fn(&Point) -> Vec<Point>,
              FUpdate: Fn(&Point, Vec<Option<&T>>, Option<&T>) -> Vec<Update<T>>
    {
        let mut updates = Vec::new();

        // let (min_sub, max_sub) = self.find_subgrid_bounds();
        // let grid_length = self.find_grid_length();

        // Fast

        let mut fast = HashSet::new();

        for (sub_index, sub) in &self.values {
            let start_x = sub_index.x * Grid::<T, L>::L_I;
            let start_z = sub_index.z * Grid::<T, L>::L_I;

            for x in 0..Grid::<T, L>::L_I {
                for z in 0..Grid::<T, L>::L_I {
                    let point = Point::new(start_x + x, start_z + z);
                    let value = Some(self.get_in_subgrid(sub, &point));

                    let neighbors = visitor(&point);

                    let mut neighbor_values = Vec::new();

                    for neighbor in neighbors {
                        neighbor_values.push(self.get(&neighbor));
                    }

                    updates.append(&mut updater(&point, neighbor_values, value));
                }
            }

            fast.insert(sub_index);
        }

        // Accurate

        let (min, max) = self.find_grid_bounds();

        for x in (min.x - pad)..(max.x + pad) {
            for z in (min.z - pad)..(max.z + pad) {
                let point = Point::new(x, z);

                let sub_index = point.to_subgrid_index(Grid::<T, L>::L_I);
                if fast.contains(&sub_index) {
                    continue;
                }

                let neighbors = visitor(&point);
                let mut neighbor_values = Vec::new();

                for neighbor in neighbors {
                    neighbor_values.push(self.get(&neighbor));
                }

                //

                updates.append(&mut updater(&point, neighbor_values, Some(T::default()).as_ref()));
            }
        }

        //

        for update in updates {
            let old = self.get(&update.p);
            let new = (update.f)(old);

            self.set(&update.p, new.unwrap_or(T::default()));
        }
    }

    pub fn print<F>(&self, should_display: F) where F: Fn(&T) -> bool {
        let (min_sub, max_sub) = self.find_subgrid_bounds();
        let grid_length = self.find_grid_length();

        //

        let mut rows = Vec::<String>::with_capacity(grid_length.z);

        for _ in 0..grid_length.z {
            rows.push(" ".repeat(grid_length.x));
        }

        for sub_x in min_sub.x..max_sub.x + Grid::<T, L>::L_I {
            for sub_z in min_sub.z..max_sub.z + Grid::<T, L>::L_I {
                let sub_i = SubGridIndex::new(sub_x, sub_z);

                let start_x = (sub_x - min_sub.x) as usize;
                let start_z = (sub_z - min_sub.z) as usize;

                match self.values.get(&sub_i) {
                    None => {}
                    Some(sub) => {
                        for x in 0..L {
                            for z in 0..L {
                                if should_display(sub.get(&SubGridPoint::new(x, z))) {
                                    let d_x = start_x * L + x;
                                    let d_z = start_z * L + z;

                                    rows[d_z].replace_range(d_x..=d_x, "#");
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("+{}+", "-".repeat(grid_length.x));
        for row in rows {
            println!("|{}|", row);
        }
        println!("+{}+", "-".repeat(grid_length.x));
    }

    pub fn get(&self, p: &Point) -> Option<&T> {
        match self.get_subgrid(p) {
            None => None,
            Some(sub) => {
                Some(sub.get(&p.to_subgrid_point(Grid::<T, L>::L_I)))
            }
        }
    }

    pub fn get_or_expand(&mut self, p: &Point) -> &T {
        let sub = self.get_subgrid_or_expand(p);

        sub.get(&p.to_subgrid_point(Grid::<T, L>::L_I))
    }

    pub fn get_in_subgrid<'a>(&self, sub: &'a SubGrid<T, L>, p: &Point) -> &'a T {
        sub.get(&p.to_subgrid_point(Grid::<T, L>::L_I))
    }

    // pub fn get_mut(&mut self, p: &Point) -> &mut T {
    //     let sub = self.get_or_create_subgrid(p);
    //
    //     sub.get_mut(&p.to_subgrid_point(Grid::<T, L>::L_I))
    // }

    pub fn set(&mut self, p: &Point, v: T) {
        let sub = self.get_subgrid_or_expand(p);

        sub.set(&p.to_subgrid_point(Grid::<T, L>::L_I), v);
    }

    fn get_subgrid(&self, p: &Point) -> Option<&SubGrid<T, L>> {
        self.values.get(&p.to_subgrid_index(Grid::<T, L>::L_I))
    }

    fn get_subgrid_or_expand(&mut self, p: &Point) -> &mut SubGrid<T, L> {
        self.values.entry(p.to_subgrid_index(Grid::<T, L>::L_I)).or_insert(SubGrid::new())
    }
}

#[derive(PartialEq, Eq, Hash)]
struct SubGridIndex {
    x: isize,
    z: isize,
}

impl SubGridIndex {
    pub fn new(x: isize, z: isize) -> Self {
        Self { x, z }
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

#[derive(Clone)]
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

    fn get_ro(&self, p: &SubGridPoint) -> T {
        self.values[p.x][p.z].clone()
    }

    fn get_mut(&mut self, p: &SubGridPoint) -> &mut T {
        &mut self.values[p.x][p.z]
    }

    fn set(&mut self, p: &SubGridPoint, v: T) {
        self.values[p.x][p.z] = v;
    }
}

pub fn div_neg_isize(a: isize, b: isize) -> isize {
    if a >= 0 {
        a / b
    } else {
        (a - b + 1) / b
    }
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
