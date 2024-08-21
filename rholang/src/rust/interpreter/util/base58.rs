// See rholang/src/main/scala/coop/rchain/rholang/interpreter/util/codec/Base58.scala
const ALPHABET: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

pub fn encode(input: &[u8]) -> String {
    if input.is_empty() {
        return String::new();
    }

    let mut num = 0;

    for &byte in input {
        num = num * 256 + byte as usize;
    }

    let mut result = String::new();

    while num > 0 {
        let remainder = num % 58;

        result.push(ALPHABET.chars().nth(remainder).unwrap());

        num /= 58;
    }

    for &_byte in input.iter().take_while(|&&byte| byte == 0) {
        result.push(ALPHABET.chars().next().unwrap());
    }

    result.chars().rev().collect()
}

pub fn decode(input: &str) -> Result<Vec<u8>, String> {
    let mut num = 0;

    let mut leading_zeros = 0;

    for c in input.chars() {
        let value = ALPHABET
            .chars()
            .position(|x| x == c)
            .ok_or("Invalid Base58 character")?;

        num = num * 58 + value;

        if c == ALPHABET.chars().next().unwrap() {
            leading_zeros += 1;
        }
    }

    let mut result = Vec::new();

    while num > 0 {
        result.push((num % 256) as u8);

        num /= 256;
    }

    for _ in 0..leading_zeros {
        result.push(0);
    }

    result.reverse();

    Ok(result)
}
