use anyhow::Result;

fn valid_password(password: i64) -> bool {
    let mut digits: Vec<i64> = Vec::new();
    for password_char in format!("{}", password).chars() {
        digits.push(password_char.to_digit(10).unwrap() as i64);
    }

    let mut last_digit = digits.get(0).unwrap();
    let mut found_pair = false;
    for digit in &digits[1..] {
        if digit < last_digit {
            return false;
        }

        if digit == last_digit {
            found_pair = true;
        }

        last_digit = digit;
    }

    found_pair
}

fn main() -> Result<()> {
    let mut valid_count = 0;
    for i in 245182..790573 {
        if valid_password(i) {
            valid_count += 1;
        }
    }

    println!("Valid: {}", valid_count);

    Ok(())
}
