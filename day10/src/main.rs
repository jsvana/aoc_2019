use std::cmp::{max, min, Ordering};
use std::collections::BTreeSet;

use anyhow::Result;
use log::{debug, info};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.x, &self.y).cmp(&(other.x, &other.y))
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Point {
    fn add(&mut self, other: &Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Point {
    fn reflect_x(&self) -> Point {
        Point {
            x: -self.x,
            y: self.y,
        }
    }

    fn reflect_y(&self) -> Point {
        Point {
            x: self.x,
            y: -self.y,
        }
    }

    fn reflect(&self) -> Point {
        Point {
            x: -self.x,
            y: -self.y,
        }
    }
}

struct Map {
    data: Vec<Vec<char>>,
    bounds: Point,
}

impl Map {
    fn new(data: &Vec<Vec<char>>) -> Self {
        Self {
            data: data.clone(),
            bounds: Point {
                x: data[0].len() as i32,
                y: data.len() as i32,
            },
        }
    }

    fn contains(&self, point: &Point) -> bool {
        point.x >= 0 && point.x < self.bounds.x && point.y >= 0 && point.y < self.bounds.y
    }

    fn get(&self, point: &Point) -> Option<char> {
        if !self.contains(point) {
            return None;
        }

        Some(self.data[point.y as usize][point.x as usize])
    }
}

fn count_ray(map: &Map, origin: &Point, angle: &Point) -> BTreeSet<Point> {
    let mut points = BTreeSet::new();
    let mut iter_point = origin.clone();
    iter_point.add(angle);

    while map.contains(&iter_point) {
        if map.get(&iter_point).unwrap() == '#' {
            debug!("  FOUND ({}, {})", iter_point.x, iter_point.y);
            points.insert(iter_point);
            break;
        }

        iter_point.add(angle);
    }

    points
}

fn count_angle(map: &Map, origin: &Point, angle: &Point) -> BTreeSet<Point> {
    let mut total_count = BTreeSet::new();

    total_count.append(&mut count_ray(map, origin, angle));
    total_count.append(&mut count_ray(map, origin, &angle.reflect_x()));
    total_count.append(&mut count_ray(map, origin, &angle.reflect_y()));
    total_count.append(&mut count_ray(map, origin, &angle.reflect()));

    total_count
}

// Check those that can't reduce further (2, 3), (1, 2)
fn count_visible_asteroids(map: &Map, point: &Point, angles: &BTreeSet<Point>) -> usize {
    let mut total_count = BTreeSet::new();

    debug!(
        "POINT ({}, {}), {} angles to check",
        point.x,
        point.y,
        angles.len()
    );
    for angle in angles.iter() {
        total_count.append(&mut count_angle(map, point, angle));
    }

    total_count.len()
}

// Taken from RosettaCode
fn gcd(a: usize, b: usize) -> usize {
    match ((a, b), (a & 1, b & 1)) {
        ((x, y), _) if x == y => y,
        ((0, x), _) | ((x, 0), _) => x,
        ((x, y), (0, 1)) | ((y, x), (1, 0)) => gcd(x >> 1, y),
        ((x, y), (0, 0)) => gcd(x >> 1, y >> 1) << 1,
        ((x, y), (1, 1)) => {
            let (x, y) = (min(x, y), max(x, y));
            gcd((y - x) >> 1, x)
        }
        _ => unreachable!(),
    }
}

fn build_angles(bounds: &Point) -> BTreeSet<Point> {
    let mut angles = BTreeSet::new();

    angles.insert(Point { x: 0, y: 1 });
    angles.insert(Point { x: 1, y: 0 });

    for y in 1..bounds.y {
        for x in 1..bounds.x {
            let divisor = gcd(x as usize, y as usize);
            if divisor == 0 {
                continue;
            }

            let normalized_x = (x as usize / divisor) as i32;
            let normalized_y = (y as usize / divisor) as i32;

            debug!(
                "({}, {}): {}, ({}, {})",
                x, y, divisor, normalized_x, normalized_y
            );

            if normalized_x != 0 && normalized_y != 0 {
                angles.insert(Point {
                    x: normalized_x,
                    y: normalized_y,
                });
            }
        }
    }

    angles
}

fn count_asteroids(map: &Map) -> usize {
    let angles = build_angles(&map.bounds);

    let mut max_count = 0;
    for y in 0..map.bounds.y {
        for x in 0..map.bounds.x {
            let point = Point { x, y };
            if map.get(&point).unwrap() == '#' {
                debug!("Checking ({}, {})", x, y);
                let count = count_visible_asteroids(map, &point, &angles);
                debug!("({}, {}): {}", x, y, count);
                max_count = max(max_count, count);
            }
        }
    }

    max_count
}

fn print_map(map: &Map) {
    for row in map.data.iter() {
        for ch in row.iter() {
            print!("{}", ch);
        }
        println!("");
    }
}

fn read_input(filename: &str) -> Result<Map> {
    let data = std::fs::read_to_string(filename)?;

    let mut output = Vec::new();
    for line in data.split("\n") {
        if line.len() == 0 {
            continue;
        }
        output.push(line.chars().collect());
    }

    Ok(Map::new(&output))
}

fn main() -> Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let map = read_input("test.txt")?;

    print_map(&map);

    let count = count_asteroids(&map);
    info!("Max count: {}", count);

    Ok(())
}
