use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::str::FromStr;

use anyhow::Result;
use log::debug;
use thiserror::Error;

#[derive(Error, Debug)]
enum ParseComponentError {
    #[error("The passed source has the wrong number of parts")]
    WrongNumberOfParts,

    #[error("Can't parse passed quantity value")]
    CannotParseQuantity,
}

#[derive(Clone, Debug)]
struct Component {
    name: String,
    quantity: u32,
}

impl FromStr for Component {
    type Err = ParseComponentError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = source.split(" ").collect();

        if parts.len() != 2 {
            return Err(ParseComponentError::WrongNumberOfParts);
        }

        let quantity = match parts.get(0).unwrap().parse() {
            Ok(quantity) => quantity,
            Err(_) => return Err(ParseComponentError::CannotParseQuantity),
        };

        Ok(Component {
            name: parts.get(1).unwrap().to_string(),
            quantity,
        })
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.quantity, self.name)
    }
}

#[derive(Clone, Debug)]
struct Reaction {
    components: Vec<Component>,
    result: Component,
}

#[derive(Error, Debug)]
enum ParseReactionError {
    #[error("The passed source has the wrong number of parts")]
    WrongNumberOfParts,

    #[error("Can't parse passed component")]
    CannotParseComponent,
}

impl fmt::Display for Reaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} => {}",
            self.components
                .iter()
                .map(|c| format!("{}", c))
                .collect::<Vec<String>>()
                .join(", "),
            self.result
        )
    }
}

fn parse_component_list(source: &str) -> Result<Vec<Component>, ParseComponentError> {
    let parts: Vec<&str> = source.split(", ").collect();

    let mut components = Vec::new();
    for part in parts.into_iter() {
        components.push(part.parse()?);
    }

    Ok(components)
}

impl FromStr for Reaction {
    type Err = ParseReactionError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let reaction_parts: Vec<&str> = source.split(" => ").collect();

        if reaction_parts.len() != 2 {
            return Err(ParseReactionError::WrongNumberOfParts);
        }

        let components = parse_component_list(reaction_parts.get(0).unwrap())
            .map_err(|_| ParseReactionError::CannotParseComponent)?;

        let result = reaction_parts
            .get(1)
            .unwrap()
            .parse()
            .map_err(|_| ParseReactionError::CannotParseComponent)?;

        Ok(Reaction { components, result })
    }
}

fn read_input(filename: &str) -> Result<Vec<Reaction>> {
    let file_str = std::fs::read_to_string(filename)?;

    let mut reactions = Vec::new();
    for line in file_str.split("\n").filter(|l| l.len() > 0) {
        reactions.push(line.parse()?);
    }
    Ok(reactions)
}

fn build_reaction_map(reactions: &Vec<Reaction>) -> BTreeMap<String, Reaction> {
    let mut reaction_map = BTreeMap::new();
    for reaction in reactions.iter() {
        reaction_map.insert(reaction.result.name.clone(), reaction.clone());
    }
    reaction_map
}

fn lowest_ore_cost_for_fuel(reaction_map: &BTreeMap<String, Reaction>) -> u32 {
    let mut needed = BTreeMap::new();
    needed.insert("FUEL", 1);

    let mut to_visit = VecDeque::new();
    // Should this be depth- or breadth-first?
    to_visit.push_front("FUEL");

    let mut extra = BTreeMap::new();

    while !to_visit.is_empty() {
        let next = to_visit.pop_front().unwrap();

        if next == "ORE" {
            continue;
        }

        debug!("Checking {}", next);

        let quantity_needed = match needed.get(next) {
            Some(quantity) => *quantity,
            None => break,
        };
        needed.remove(next);

        let reaction = reaction_map.get(next).unwrap();

        let output = reaction.result.quantity;
        let multiplier = (quantity_needed as f32 / output as f32).ceil() as u32;

        /*
        if next == "A" {
            println!("YO GENERATED {} A, NEEDED {}",
        }
        */

        /*
        if quantity_needed < output {
            let extra_generated = output - quantity_needed;
            debug!("Generated {} extra {}", extra_generated, next);
            *extra.entry(next.to_string()).or_insert(0) += extra_generated;
        }
        */

        debug!(
            "Generated {} {}, multiplier {}",
            output * multiplier,
            next,
            multiplier
        );

        let quantity_generated = output * multiplier;
        if quantity_generated < quantity_needed {
            debug!(
                "WAT {}, needed {} generated {}",
                next, quantity_needed, quantity_generated
            );
        }
        *extra.entry(next.to_string()).or_insert(0) += quantity_generated - quantity_needed;

        for component in reaction.components.iter() {
            to_visit.push_front(&component.name);

            let mut component_needed = component.quantity * multiplier;
            let mut component_extra = *extra.get(&component.name).unwrap_or(&0);
            debug!("Need {}", component.name);
            if component_extra > 0 {
                debug!("Have extra {}", component.name);
                if component_needed >= component_extra {
                    debug!("Using all extra {}", component.name);
                    component_needed -= component_extra;
                    component_extra = 0;
                } else {
                    debug!("Using {} extra {}", component_needed, component.name);
                    component_extra -= component_needed;
                    component_needed = 0;
                }
            }

            debug!("Still need {} {}", component_needed, component.name);

            extra.insert(component.name.clone(), component_extra);
            *needed.entry(&component.name).or_insert(0) += component_needed;
        }
    }

    println!("EXTRA: {:?}", extra);

    *needed.get("ORE").unwrap()
}

fn main() -> Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let reactions = read_input("test5.txt")?;

    let reaction_map = build_reaction_map(&reactions);
    //println!("{:?}", reaction_map);

    println!("Lowest cost: {:?}", lowest_ore_cost_for_fuel(&reaction_map));

    Ok(())
}
