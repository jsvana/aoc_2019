use std::cmp::Ordering;

use anyhow::Result;

#[derive(Clone, Debug)]
struct Point3d {
    x: i64,
    y: i64,
    z: i64,
}

impl PartialEq for Point3d {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl Point3d {
    fn new() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
}

#[derive(Clone, Debug)]
struct Moon {
    position: Point3d,
    velocity: Point3d,
}

impl PartialEq for Moon {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.velocity == other.velocity
    }
}

impl Moon {
    fn new(initial_position: Point3d) -> Self {
        Self {
            position: initial_position,
            velocity: Point3d::new(),
        }
    }

    fn apply_gravity_from_moon(&mut self, other: &mut Self) {
        let x_delta = match self.position.x.cmp(&other.position.x) {
            Ordering::Less => 1,
            Ordering::Equal => 0,
            Ordering::Greater => -1,
        };

        self.velocity.x += x_delta;
        other.velocity.x -= x_delta;

        let y_delta = match self.position.y.cmp(&other.position.y) {
            Ordering::Less => 1,
            Ordering::Equal => 0,
            Ordering::Greater => -1,
        };

        self.velocity.y += y_delta;
        other.velocity.y -= y_delta;

        let z_delta = match self.position.z.cmp(&other.position.z) {
            Ordering::Less => 1,
            Ordering::Equal => 0,
            Ordering::Greater => -1,
        };

        self.velocity.z += z_delta;
        other.velocity.z -= z_delta;
    }

    fn apply_velocity(&mut self) {
        self.position.x += self.velocity.x;
        self.position.y += self.velocity.y;
        self.position.z += self.velocity.z;
    }

    fn potential_energy(&self) -> i64 {
        self.position.x.abs() + self.position.y.abs() + self.position.z.abs()
    }

    fn kinetic_energy(&self) -> i64 {
        self.velocity.x.abs() + self.velocity.y.abs() + self.velocity.z.abs()
    }

    fn total_energy(&self) -> i64 {
        self.potential_energy() * self.kinetic_energy()
    }

    fn print(&self) {
        println!(
            "pos=<x={}, y={}, z={}>, vel=<x={}, y={}, z={}>",
            self.position.x,
            self.position.y,
            self.position.z,
            self.velocity.x,
            self.velocity.y,
            self.velocity.z,
        );
    }
}

fn step(io: &mut Moon, ganymede: &mut Moon, callisto: &mut Moon, europa: &mut Moon) {
    io.apply_gravity_from_moon(ganymede);
    io.apply_gravity_from_moon(callisto);
    io.apply_gravity_from_moon(europa);
    ganymede.apply_gravity_from_moon(europa);
    ganymede.apply_gravity_from_moon(callisto);
    europa.apply_gravity_from_moon(callisto);

    io.apply_velocity();
    ganymede.apply_velocity();
    callisto.apply_velocity();
    europa.apply_velocity();
}

fn main() -> Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut io = Moon::new(Point3d { x: -7, y: -1, z: 6 });
    let mut ganymede = Moon::new(Point3d { x: 6, y: -9, z: -9 });
    let mut callisto = Moon::new(Point3d {
        x: -12,
        y: 2,
        z: -7,
    });
    let mut europa = Moon::new(Point3d {
        x: 4,
        y: -17,
        z: -12,
    });

    let initial_io = io.clone();
    let initial_ganymede = ganymede.clone();
    let initial_callisto = callisto.clone();
    let initial_europa = europa.clone();

    let mut counter: u64 = 0;
    loop {
        step(&mut io, &mut ganymede, &mut callisto, &mut europa);

        counter += 1;

        if initial_io == io
            && initial_ganymede == ganymede
            && initial_callisto == callisto
            && initial_europa == europa
        {
            println!("Equal after {} step(s)", counter);
            break;
        }
    }

    println!(
        "Total energy: {}",
        io.total_energy()
            + ganymede.total_energy()
            + callisto.total_energy()
            + europa.total_energy()
    );

    Ok(())
}
