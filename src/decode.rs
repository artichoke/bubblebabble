use alloc::vec::Vec;

use crate::DecodeError;

const HEADER: u8 = b'x';
const TRAILER: u8 = b'x';

// one `bool` for every byte. The positions that are set to `true` are the byte
// values for characters in the alphabet:
//
// ```
// const ALPHABET: [u8; 24] = *b"aeiouybcdfghklmnprstvzx-";
// ```
//
// This table is generated with the following Ruby script:
//
// ```ruby
// a = Array.new(256, 0)
// bytes = "aeiouybcdfghklmnprstvzx-".split("").map(&:ord)
// bytes.each {|b| a[b] = 1}
// puts a.inspect
// ```
const ALPHABET_TABLE: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub fn inner(encoded: &[u8]) -> Result<Vec<u8>, DecodeError> {
    // `xexax` is the encoded representation of an empty byte string. Test for
    // it directly to short circuit.
    if encoded == b"xexax" {
        return Ok(Vec::new());
    }
    let enc = match encoded {
        [HEADER, enc @ .., TRAILER] => enc,
        [HEADER, ..] => return Err(DecodeError::MalformedTrailer),
        [.., TRAILER] => return Err(DecodeError::MalformedHeader),
        _ => return Err(DecodeError::Corrupted),
    };
    // This validation step ensures that the encoded byte string only contains
    // ASCII bytes in the 24 character encoding alphabet.
    //
    // Code below must still handle None results from `find_byte` because bytes
    // may not be from the right subset of the alphabet, e.g. a vowel present
    // when a consonant is expected.
    if let Some((_, pos)) = enc
        .iter()
        .zip(1_usize..) // start `pos` at 1 because we stripped off a leading 'x'
        .find(|(&byte, _)| ALPHABET_TABLE[usize::from(byte)] == 0)
    {
        return Err(DecodeError::InvalidByte(pos));
    }
    let mut decoded = {
        let len = encoded.len();
        Vec::with_capacity(if len == 5 { 1 } else { 2 * ((len + 1) / 6) })
    };
    let mut checksum = 1_u8;
    let mut chunks = enc.chunks_exact(6);
    while let Some(&[left, mid, right, up, b'-', down]) = chunks.next() {
        let byte1 = decode_3_tuple(
            index_from_vowel(left).ok_or(DecodeError::ExpectedVowel)?,
            index_from_consonant(mid).ok_or(DecodeError::ExpectedConsonant)?,
            index_from_vowel(right).ok_or(DecodeError::ExpectedVowel)?,
            checksum,
        )?;
        let byte2 = decode_2_tuple(
            index_from_consonant(up).ok_or(DecodeError::ExpectedConsonant)?,
            index_from_consonant(down).ok_or(DecodeError::ExpectedConsonant)?,
        );
        checksum =
            ((u16::from(checksum * 5) + (u16::from(byte1) * 7) + u16::from(byte2)) % 36) as u8;
        decoded.push(byte1);
        decoded.push(byte2);
    }
    if let [left, mid, right] = *chunks.remainder() {
        let a = index_from_vowel(left).ok_or(DecodeError::ExpectedVowel)?;
        let c = index_from_vowel(right).ok_or(DecodeError::ExpectedVowel)?;

        match mid {
            b'x' if a != checksum % 6 || c != checksum / 6 => Err(DecodeError::ChecksumMismatch),
            b'x' => Ok(decoded),
            _ => {
                let b = index_from_consonant(mid).ok_or(DecodeError::ExpectedConsonant)?;
                let byte = decode_3_tuple(a, b, c, checksum)?;
                decoded.push(byte);
                Ok(decoded)
            }
        }
    } else {
        Err(DecodeError::Corrupted)
    }
}

#[inline]
fn index_from_consonant(consonant: u8) -> Option<u8> {
    let index = match consonant {
        b'b' => 0,
        b'c' => 1,
        b'd' => 2,
        b'f' => 3,
        b'g' => 4,
        b'h' => 5,
        b'k' => 6,
        b'l' => 7,
        b'm' => 8,
        b'n' => 9,
        b'p' => 10,
        b'r' => 11,
        b's' => 12,
        b't' => 13,
        b'v' => 14,
        b'z' => 15,
        _ => return None,
    };
    Some(index)
}

#[inline]
fn index_from_vowel(vowel: u8) -> Option<u8> {
    let index = match vowel {
        b'a' => 0,
        b'e' => 1,
        b'i' => 2,
        b'o' => 3,
        b'u' => 4,
        b'y' => 5,
        _ => return None,
    };
    Some(index)
}

#[inline]
fn decode_3_tuple(byte1: u8, byte2: u8, byte3: u8, checksum: u8) -> Result<u8, DecodeError> {
    // Will not overflow since:
    // - `byte1` is guaranteed to be ASCII or < 128.
    // Will not underflow since:
    // - 6 - (checksum % 6) > 0
    let high = (byte1 + 6 - (checksum % 6)) % 6;
    let mid = byte2;
    // Will not overflow since:
    // - `byte3` is guaranteed to be ASCII or < 128.
    // Will not underflow since:
    // - 6 - ((checksum / 6) % 6) > 0
    let low = (byte3 + 6 - ((checksum / 6) % 6)) % 6;
    if high >= 4 || low >= 4 {
        Err(DecodeError::Corrupted)
    } else {
        Ok((high << 6) | (mid << 2) | low)
    }
}

#[inline]
fn decode_2_tuple(byte1: u8, byte2: u8) -> u8 {
    (byte1 << 4) | byte2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_from_consonant_valid() {
        // Valid consonants should correctly map to their respective indices.
        let cases = [
            (b'b', 0),
            (b'c', 1),
            (b'd', 2),
            (b'f', 3),
            (b'g', 4),
            (b'h', 5),
            (b'k', 6),
            (b'l', 7),
            (b'm', 8),
            (b'n', 9),
            (b'p', 10),
            (b'r', 11),
            (b's', 12),
            (b't', 13),
            (b'v', 14),
            (b'z', 15),
        ];
        for &(input, expected) in &cases {
            assert_eq!(
                index_from_consonant(input),
                Some(expected),
                "Expected consonant '{}' to map to {}",
                char::from_u32(input.into()).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn test_index_from_consonant_invalid() {
        // Non-consonant characters should return None.
        for &input in b"aeiouyx-" {
            assert_eq!(
                index_from_consonant(input),
                None,
                "Expected '{}' to be invalid as a consonant",
                char::from_u32(input.into()).unwrap(),
            );
        }
    }

    #[test]
    fn test_index_from_vowel_valid() {
        // Valid vowels should correctly map to their respective indices.
        let cases = [
            (b'a', 0),
            (b'e', 1),
            (b'i', 2),
            (b'o', 3),
            (b'u', 4),
            (b'y', 5),
        ];
        for &(input, expected) in &cases {
            assert_eq!(
                index_from_vowel(input),
                Some(expected),
                "Expected vowel '{}' to map to {}",
                char::from_u32(input.into()).unwrap(),
                expected,
            );
        }
    }

    #[test]
    fn test_index_from_vowel_invalid() {
        // Non-vowel characters should return None.
        for &input in b"bcdfghklmnpqrstvxz-" {
            assert_eq!(
                index_from_vowel(input),
                None,
                "Expected '{}' to be invalid as a vowel",
                char::from_u32(input.into()).unwrap(),
            );
        }
    }

    #[test]
    fn test_index_from_consonant_exhaustive() {
        // Iterate over all ASCII characters (0–127) and verify that valid
        // consonants are mapped to the expected index, while any other character
        // returns None.
        for byte in 0u8..=127 {
            let expected = match byte {
                b'b' => Some(0),
                b'c' => Some(1),
                b'd' => Some(2),
                b'f' => Some(3),
                b'g' => Some(4),
                b'h' => Some(5),
                b'k' => Some(6),
                b'l' => Some(7),
                b'm' => Some(8),
                b'n' => Some(9),
                b'p' => Some(10),
                b'r' => Some(11),
                b's' => Some(12),
                b't' => Some(13),
                b'v' => Some(14),
                b'z' => Some(15),
                _ => None,
            };
            assert_eq!(
                index_from_consonant(byte),
                expected,
                "index_from_consonant failed for byte {} ('{}')",
                byte,
                char::from_u32(byte.into()).unwrap(),
            );
        }
    }

    #[test]
    fn test_index_from_vowel_exhaustive() {
        // Iterate over all ASCII characters (0–127) and verify that valid
        // vowels are mapped to the expected index, while any other character returns None.
        for byte in 0u8..=127 {
            let expected = match byte {
                b'a' => Some(0),
                b'e' => Some(1),
                b'i' => Some(2),
                b'o' => Some(3),
                b'u' => Some(4),
                b'y' => Some(5),
                _ => None,
            };
            assert_eq!(
                index_from_vowel(byte),
                expected,
                "index_from_vowel failed for byte {} ('{}')",
                byte,
                char::from_u32(byte.into()).unwrap(),
            );
        }
    }

    #[test]
    fn test_decode_2_tuple() {
        // Verify that two 4‑bit values are correctly combined.
        // For example, (1,2) should produce (1 << 4) | 2 = 18,
        // and the maximum (15,15) yields 255.
        assert_eq!(decode_2_tuple(0, 0), 0);
        assert_eq!(decode_2_tuple(1, 2), (1 << 4) | 2);
        assert_eq!(decode_2_tuple(15, 15), 255);
    }

    #[test]
    fn test_decode_3_tuple_success() {
        // Test a successful three-tuple decoding.
        // With vowel index 1, consonant index 2, vowel index 1, and checksum 1:
        //   high = (1 + 6 - (1 % 6)) % 6 = (7 - 1) % 6 = 6 % 6 = 0,
        //   low  = (1 + 6 - ((1 / 6) % 6)) % 6 = (7 - 0) % 6 = 7 % 6 = 1,
        //   final byte = (0 << 6) | (2 << 2) | 1 = 0 | 8 | 1 = 9.
        assert_eq!(decode_3_tuple(1, 2, 1, 1), Ok(9));
    }

    #[test]
    fn test_decode_3_tuple_valid_with_checksum() {
        // Test a valid decoding where the checksum adjusts the values.
        // For inputs: vowel index 2, consonant index 3, vowel index 2, and checksum 7:
        //   high = (2 + 6 - (7 % 6)) % 6 = (8 - 1) % 6 = 7 % 6 = 1,
        //   low  = (2 + 6 - ((7 / 6) % 6)) % 6 = (8 - 1) % 6 = 7 % 6 = 1,
        //   final byte = (1 << 6) | (3 << 2) | 1 = 64 | 12 | 1 = 77.
        assert_eq!(decode_3_tuple(2, 3, 2, 7), Ok(77));
    }

    #[test]
    fn test_decode_3_tuple_error_high() {
        // Test that an invalid 'high' component causes an error.
        // For instance, using vowel index 4 with checksum 0 gives:
        //   high = (4 + 6 - 0) % 6 = 10 % 6 = 4 (>= 4 is invalid).
        assert_eq!(decode_3_tuple(4, 0, 0, 0), Err(DecodeError::Corrupted));
    }

    #[test]
    fn test_decode_3_tuple_error_low() {
        // Test that an invalid 'low' component causes an error.
        // For example, using vowel index 4 for the third value with checksum 0 yields:
        //   low = (4 + 6 - 0) % 6 = 10 % 6 = 4 (invalid since it must be < 4).
        assert_eq!(decode_3_tuple(0, 0, 4, 0), Err(DecodeError::Corrupted));
    }
}
