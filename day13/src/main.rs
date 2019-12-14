mod intcode;

use std::collections::{BTreeMap, VecDeque};
use std::str::FromStr;

use anyhow::Result;

use crate::intcode::Program;

fn count_blocks(map: &BTreeMap<i64, BTreeMap<i64, i64>>) -> u64 {
    let mut total = 0;
    for row in map.values() {
        for ch in row.values() {
            if *ch == 2 {
                total += 1;
            }
        }
    }

    total
}

fn set_value(map: &mut BTreeMap<i64, BTreeMap<i64, i64>>, x: i64, y: i64, value: i64) {
    *map.entry(y)
        .or_insert(BTreeMap::new())
        .entry(x)
        .or_insert(0) = value;
}

fn main() -> Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut input = VecDeque::new();

    let program_str = std::fs::read_to_string("input.txt")?;

    let mut program = Program::from_str(&program_str)?;

    let mut outputs = program.run(&mut input)?;
    let mut map = BTreeMap::new();

    while outputs.len() > 0 {
        let x = outputs.pop_front().unwrap();
        let y = outputs.pop_front().unwrap();
        let tile_id = outputs.pop_front().unwrap();

        set_value(&mut map, x, y, tile_id);
    }

    println!("Blocks: {}", count_blocks(&map));

    Ok(())
}
