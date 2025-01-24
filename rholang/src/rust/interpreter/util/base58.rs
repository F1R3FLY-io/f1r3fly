// See rholang/src/main/scala/coop/rchain/rholang/interpreter/util/codec/Base58.scala

const ALPHABET: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

pub fn encode(input: &[u8]) -> String {
    if input.is_empty() {
        return String::new();
    }

    // Count leading zeros
    let leading_zeros = input.iter().take_while(|&&byte| byte == 0).count();

    // Convert byte array to a BigInteger-like representation
    let mut num = Vec::from(input);

    // Perform base58 encoding
    let mut encoded = String::new();

    while !num.is_empty() {
        let mut remainder = 0;
        let mut new_num = Vec::new();

        for &byte in &num {
            let temp = (remainder << 8) + byte as usize;
            let div = temp / 58;
            remainder = temp % 58;
            if !new_num.is_empty() || div != 0 {
                new_num.push(div as u8);
            }
        }

        encoded.push(ALPHABET.chars().nth(remainder).unwrap());
        num = new_num;
    }

    // Add '1' for each leading zero byte
    for _ in 0..leading_zeros {
        encoded.push(ALPHABET.chars().next().unwrap());
    }

    // Reverse the encoded string to get the correct order
    encoded.chars().rev().collect()
}

pub fn decode(input: &str) -> Result<Vec<u8>, String> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    // Initialize a vector to hold the decoded bytes
    let mut bytes: Vec<u8> = Vec::new();

    // Iterate over each character in the input string
    for c in input.chars() {
        // Get the value of the character in the Base58 alphabet
        let value = ALPHABET
            .find(c)
            .ok_or_else(|| format!("Invalid Base58 character: {}", c))?;

        // Perform the multiplication and addition
        let mut carry = value;
        for byte in bytes.iter_mut().rev() {
            let temp = (*byte as usize) * 58 + carry;
            *byte = (temp & 0xFF) as u8;
            carry = temp >> 8;
        }

        while carry > 0 {
            bytes.insert(0, (carry & 0xFF) as u8);
            carry >>= 8;
        }
    }

    // Handle leading '1's as leading zeros
    let leading_ones = input.chars().take_while(|&c| c == '1').count();
    let mut decoded = vec![0u8; leading_ones];
    decoded.extend(bytes);

    Ok(decoded)
}
