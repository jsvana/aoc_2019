use std::collections::BTreeMap;
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

#[derive(Debug)]
struct OreCost {
    unused: BTreeMap<String, u32>,
    ore_count: u32,
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

fn lowest_ore_cost_for_node(
    component_name: &str,
    quantity: u32,
    reaction_map: &BTreeMap<String, Reaction>,
    indent: usize,
    //costs: &mut BTreeMap<String, OreCost>,
) -> OreCost {
    let mut min_ore_cost = OreCost {
        unused: BTreeMap::new(),
        ore_count: std::u32::MAX,
    };

    let reaction = reaction_map.get(component_name).unwrap();
    let mut total_cost = 0;
    let mut unused = BTreeMap::new();
    for inner_component in reaction.components.iter() {
        if inner_component.name == "ORE" {
            total_cost += inner_component.quantity;
        } else {
            debug!(
                "{:width$}Need {} {}, recursing",
                " ",
                quantity,
                inner_component.name,
                width = indent,
            );

            let mut extra = *unused.get(&inner_component.name).unwrap_or(&0);

            let mut required = inner_component.quantity;

            if extra > 0 {
                if required <= extra {
                    debug!(
                        "{:width$}Using some extra {} ({} used, {} total)",
                        " ",
                        inner_component.name,
                        required,
                        extra,
                        width = indent,
                    );

                    extra -= required;
                    required = 0;

                    *unused.entry(inner_component.name.to_string()).or_insert(0) = extra;
                } else {
                    debug!(
                        "{:width$}Using all extra {} ({} total)",
                        " ",
                        inner_component.name,
                        extra,
                        width = indent,
                    );

                    required -= extra;

                    *unused.entry(inner_component.name.to_string()).or_insert(0) = 0;
                }
            }

            if required > 0 {
                let cost = lowest_ore_cost_for_node(
                    &inner_component.name,
                    required,
                    reaction_map,
                    indent + 2,
                );

                total_cost += cost.ore_count;
                for (extra_name, extra_quantity) in cost.unused.iter() {
                    debug!(
                        "{:width$}Found {} extra {}",
                        " ",
                        extra_quantity,
                        extra_name,
                        width = indent
                    );
                    *unused.entry(extra_name.to_string()).or_insert(0) += extra_quantity;
                }

                debug!(
                    "{:width$}REQUIRED {} {}",
                    " ",
                    required,
                    component_name,
                    width = indent
                );
            }
        }
    }

    if reaction.result.quantity > quantity {
        *unused.entry(component_name.to_string()).or_insert(0) +=
            reaction.result.quantity - quantity;
    }

    if total_cost < min_ore_cost.ore_count {
        min_ore_cost = OreCost {
            ore_count: total_cost,
            unused,
        };
    }

    debug!(
        "{:width$}Required {} {}. Ore cost: {}",
        " ",
        quantity,
        component_name,
        min_ore_cost.ore_count,
        width = indent,
    );

    min_ore_cost
}

fn main() -> Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let reactions = read_input("test1.txt")?;

    let reaction_map = build_reaction_map(&reactions);
    println!("{:?}", reaction_map);

    println!(
        "Lowest cost: {:?}",
        lowest_ore_cost_for_node("FUEL", 1, &reaction_map, 0)
    );

    Ok(())
}
