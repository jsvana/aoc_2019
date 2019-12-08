use std::collections::BTreeMap;

use anyhow::Result;

fn print_layer(layer: &str, width: usize, height: usize) {
    for y in 0..height {
        println!("{}", &layer[y * width..(y + 1) * width]);
    }
}

fn get_layer(image: &str, width: usize, height: usize, layer: usize) -> &str {
    let layer_size = width * height;

    let layer_index = layer_size * layer;

    &image[layer_index..layer_size * (layer + 1)]
}

fn count_digits(layer: &str) -> BTreeMap<char, usize> {
    let mut counts = BTreeMap::new();

    for c in layer.chars() {
        *counts.entry(c).or_insert(0) += 1;
    }

    counts
}

fn read_input(filename: &str) -> Result<String> {
    Ok(std::fs::read_to_string(filename)?.clone())
}

fn main() -> Result<()> {
    let image = read_input("input.txt")?;

    let width = 25;
    let height = 6;

    let mut min_count: usize = std::usize::MAX;
    let mut min_layer = None;
    let layer_count = image.len() / (width * height);

    let zero = '0';

    for layer_index in 0..layer_count {
        let layer = get_layer(&image, width, height, layer_index);
        let digit_count = count_digits(layer);

        let zero_count = *digit_count.get(&zero).unwrap_or(&0);
        if zero_count < min_count {
            min_count = zero_count;
            min_layer = Some(layer_index);
        }
    }

    println!("Min layer: {}", min_layer.unwrap());
    print_layer(
        get_layer(&image, width, height, min_layer.unwrap()),
        width,
        height,
    );

    match min_layer {
        Some(layer_index) => {
            let digit_count = count_digits(get_layer(&image, width, height, layer_index));

            let one_count = *digit_count.get(&'1').unwrap_or(&0);
            let two_count = *digit_count.get(&'2').unwrap_or(&0);
            println!("Total multiplied: {}", one_count * two_count);
        }
        None => println!("WAT"),
    }

    Ok(())
}
