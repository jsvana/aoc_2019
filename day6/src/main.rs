use std::collections::{BTreeMap, BTreeSet};
use std::convert::{TryFrom, TryInto};

use anyhow::{format_err, Error, Result};
use log::{debug, info};

type Orbits = BTreeMap<String, OrbitInfo>;

#[derive(Debug, PartialEq, Eq)]
struct Orbit {
    center_of_mass: String,
    outer_planet: String,
}

#[derive(Debug)]
struct OrbitInfo {
    distance_from_center: usize,
    total_distances: usize,
    outer_planets: BTreeSet<String>,
}

impl OrbitInfo {
    fn new() -> Self {
        OrbitInfo {
            distance_from_center: 0,
            total_distances: 0,
            outer_planets: BTreeSet::new(),
        }
    }

    fn add_outer_planet(&mut self, outer_planet: &str) {
        self.outer_planets.insert(outer_planet.to_string());
    }
}

impl TryFrom<&str> for Orbit {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.splitn(2, ")").collect();

        let center_of_mass = parts.get(0).ok_or(format_err!(
            "Somehow didn't get single string from Orbit (passed \"{}\")",
            value
        ))?;

        match parts.get(1) {
            Some(outer_planet) => Ok(Orbit {
                center_of_mass: center_of_mass.to_string(),
                outer_planet: outer_planet.to_string(),
            }),
            None => Err(format_err!(
                "Orbit was not passed an outer planet (passed \"{}\")",
                value
            )),
        }
    }
}

fn string_to_orbits(input: &str) -> Result<Orbits> {
    let mut orbits = BTreeMap::new();
    for line in input.trim().split("\n") {
        let orbit: Orbit = line.try_into()?;

        orbits
            .entry(orbit.center_of_mass.clone())
            .or_insert(OrbitInfo::new())
            .add_outer_planet(&orbit.outer_planet);

        orbits
            .entry(orbit.outer_planet.clone())
            .or_insert(OrbitInfo::new());
    }
    Ok(orbits)
}

fn read_input(filename: &str) -> Result<Orbits> {
    let data = std::fs::read_to_string(filename)?;

    string_to_orbits(&data)
}

fn add_counts(orbits: &mut Orbits, node: &str, distance: usize) {
    debug!("CHECKING {}", node);
    if let Some(info) = orbits.get_mut(node) {
        info.distance_from_center = distance;

        let planets_to_check = info.outer_planets.clone();
        for outer_planet in planets_to_check.iter() {
            add_counts(orbits, &outer_planet, distance + 1);
        }
    }
}

fn sum_counts(orbits: &Orbits, node: &str) -> usize {
    let mut sum = 0;

    if let Some(info) = orbits.get(node) {
        if info.distance_from_center > 0 {
            sum += info.distance_from_center;
        }

        let planets_to_check = info.outer_planets.clone();
        for outer_planet in planets_to_check.iter() {
            sum += sum_counts(orbits, &outer_planet);
        }
    }

    sum
}

fn main() -> Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut orbits = read_input("input.txt")?;

    debug!("ORBITS: {:?}", orbits);

    add_counts(&mut orbits, "COM", 0);

    debug!("ORBITS AFTER TOTAL SIZE: {:?}", orbits);

    info!("Total count: {}", sum_counts(&orbits, "COM"));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_orbit() -> Result<()> {
        let orbit: Orbit = "A)B".try_into()?;
        assert_eq!(
            orbit,
            Orbit {
                center_of_mass: "A".to_string(),
                outer_planet: "B".to_string(),
            }
        );
        Ok(())
    }
}
