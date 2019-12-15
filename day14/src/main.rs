use std::collections::{BTreeMap, VecDeque};
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
    to_visit.push_front("FUEL");

    let mut extra = BTreeMap::new();

    loop {
        let choices: Vec<&str> = needed.keys().cloned().filter(|k| *k != "ORE").collect();

        if choices.len() == 0 {
            break;
        }

        let next = choices.into_iter().next().unwrap();

        if next == "ORE" {
            continue;
        }

        debug!("Checking {}", next);

        let quantity_needed = match needed.get(&next) {
            Some(quantity) => *quantity,
            None => break,
        };

        let reaction = reaction_map.get(next).unwrap();

        let output = reaction.result.quantity;
        let multiplier = (quantity_needed as f32 / output as f32).ceil() as u32;

        debug!(
            "Generated {} {}, multiplier {}",
            output * multiplier,
            next,
            multiplier
        );

        let quantity_generated = output * multiplier;
        // Mark any extra we've generated
        *extra.entry(next.to_string()).or_insert(0) += quantity_generated - quantity_needed;

        // Remove the stuff we've generated
        needed.remove(&next);

        for component in reaction.components.iter() {
            // Pull from extra before requiring generation
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
    println!("NEEDED: {:?}", needed);

    *needed.get("ORE").unwrap()
}

fn main() -> Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let reactions = read_input("input.txt")?;

    let reaction_map = build_reaction_map(&reactions);

    println!("Lowest cost: {}", lowest_ore_cost_for_fuel(&reaction_map));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_test(filename: &str) -> Result<u32> {
        let reactions = read_input(filename)?;

        let reaction_map = build_reaction_map(&reactions);

        Ok(lowest_ore_cost_for_fuel(&reaction_map))
    }

    #[test]
    fn test_1() -> Result<()> {
        assert_eq!(run_test("test1.txt")?, 31);

        Ok(())
    }

    #[test]
    fn test_2() -> Result<()> {
        assert_eq!(run_test("test2.txt")?, 165);

        Ok(())
    }

    #[test]
    fn test_3() -> Result<()> {
        assert_eq!(run_test("test3.txt")?, 13312);

        Ok(())
    }

    #[test]
    fn test_4() -> Result<()> {
        assert_eq!(run_test("test4.txt")?, 180697);

        Ok(())
    }

    #[test]
    fn test_5() -> Result<()> {
        assert_eq!(run_test("test5.txt")?, 2210736);

        Ok(())
    }
}
