use alloc::string::String;

const VOWELS: [u8; 6] = *b"aeiouy";
const CONSONANTS: [u8; 16] = *b"bcdfghklmnprstvz";
const HEADER: &str = "x";
const TRAILER: &str = "x";
const SEPARATOR: &str = "-";
const MID: &str = "x";

#[must_use]
pub fn inner(data: &[u8]) -> String {
    if data.is_empty() {
        return String::from("xexax");
    }

    let mut encoded = String::with_capacity(6 * (data.len() / 2) + 3 + 2);
    encoded.push_str(HEADER);
    let mut checksum = 1_u8;
    let mut chunks = data.chunks_exact(2);
    while let Some(&[left, right]) = chunks.next() {
        odd_partial(left, checksum, &mut encoded);
        let d = (right >> 4) & 15;
        let e = right & 15;
        // Panic safety:
        //
        // - `d` is constructed with a mask of `0b1111`.
        // - `CONSONANTS` is a fixed size array with 16 elements.
        // - Maximum value of `d` is 15.
        encoded.push(CONSONANTS[d as usize].into());
        encoded.push_str(SEPARATOR);
        // Panic safety:
        //
        // - `e` is constructed with a mask of `0b1111`.
        // - `CONSONANTS` is a fixed size array with 16 elements.
        // - Maximum value of `e` is 15.
        encoded.push(CONSONANTS[e as usize].into());
        checksum = ((u16::from(checksum * 5) + u16::from(left) * 7 + u16::from(right)) % 36) as u8;
    }
    if let [byte] = chunks.remainder() {
        odd_partial(*byte, checksum, &mut encoded);
    } else {
        even_partial(checksum, &mut encoded);
    }
    encoded.push_str(TRAILER);
    encoded
}

#[inline]
fn odd_partial(raw_byte: u8, checksum: u8, buf: &mut String) {
    let a = (((raw_byte >> 6) & 3) + checksum) % 6;
    let b = (raw_byte >> 2) & 15;
    let c = ((raw_byte & 3) + checksum / 6) % 6;
    // Panic safety:
    //
    // - `a` is constructed with mod 6.
    // - `VOWELS` is a fixed size array with 6 elements.
    // - Maximum value of `a` is 5.
    buf.push(VOWELS[a as usize].into());
    // Panic safety:
    //
    // - `b` is constructed with a mask of `0b1111`.
    // - `CONSONANTS` is a fixed size array with 16 elements.
    // - Maximum value of `e` is 15.
    buf.push(CONSONANTS[b as usize].into());
    // Panic safety:
    //
    // - `c` is constructed with mod 6.
    // - `VOWELS` is a fixed size array with 6 elements.
    // - Maximum value of `c` is 5.
    buf.push(VOWELS[c as usize].into());
}

#[inline]
fn even_partial(checksum: u8, buf: &mut String) {
    let a = checksum % 6;
    // let b = 16;
    let c = checksum / 6;
    // Panic safety:
    //
    // - `a` is constructed with mod 6.
    // - `VOWELS` is a fixed size array with 6 elements.
    // - Maximum value of `a` is 5.
    buf.push(VOWELS[a as usize].into());
    buf.push_str(MID);
    // Panic safety:
    //
    // - `c` is constructed with divide by 6.
    // - Maximum value of `checksum` is 36 -- see `encode` loop.
    // - `VOWELS` is a fixed size array with 6 elements.
    // - Maximum value of `c` is 5.
    buf.push(VOWELS[c as usize].into());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_empty() {
        // For empty input, the encoder returns the special value "xexax".
        let data: [u8; 0] = [];
        assert_eq!(inner(&data), "xexax");
    }

    #[test]
    fn test_encoder_even_length() {
        // Test encoding for a 2-byte input.
        //
        // For data = [0, 0]:
        //   - Start with header "x".
        //   - odd_partial(0, 1):
        //       a = (((0 >> 6) & 3) + 1) % 6 = 1   -> VOWELS[1] = 'e'
        //       b = (0 >> 2) & 15 = 0              -> CONSONANTS[0] = 'b'
        //       c = ((0 & 3) + (1/6)) % 6 = 0      -> VOWELS[0] = 'a'
        //       Result: "eba"
        //   - For right byte 0:
        //       d = (0 >> 4) & 15 = 0              -> CONSONANTS[0] = 'b'
        //       e = 0 & 15 = 0                     -> CONSONANTS[0] = 'b'
        //       Encoded pair: "b-b"
        //   - Update checksum: ((1*5) + (0*7) + 0) % 36 = 5.
        //   - Since there is no remainder, even_partial(5) is called:
        //       a = 5 % 6 = 5                      -> VOWELS[5] = 'y'
        //       MID = "x"
        //       c = 5 / 6 = 0                      -> VOWELS[0] = 'a'
        //       Result: "yxa"
        //   - Trailer "x" is appended.
        //
        // Final expected encoding: "x" + "eba" + "b-b" + "yxa" + "x" = "xebab-byxax"
        let data = [0, 0];
        assert_eq!(inner(&data), "xebab-byxax");
    }

    #[test]
    fn test_encoder_odd_length() {
        // Test encoding for a 1-byte input.
        //
        // For data = [0]:
        //   - Header "x" is added.
        //   - The only byte is processed by odd_partial(0, 1), which produces
        //     "eba" (see previous test).
        //   - No even_partial call is made.
        //   - Trailer "x" is appended.
        //
        // Final expected encoding: "x" + "eba" + "x" = "xebax"
        let data = [0];
        assert_eq!(inner(&data), "xebax");
    }

    #[test]
    fn test_encoder_custom_even() {
        // Test encoding for a custom 2-byte input.
        //
        // For data = [255, 0]:
        //   - odd_partial(255, 1):
        //       raw_byte 255 = 0b11111111.
        //       a = (((255 >> 6) & 3) + 1) % 6 = ((3 + 1) % 6) = 4     -> VOWELS[4] = 'u'
        //       b = (255 >> 2) & 15 = 63 & 15 = 15                     -> CONSONANTS[15] = 'z'
        //       c = ((255 & 3) + (1/6)) % 6 = (3 + 0) % 6 = 3          -> VOWELS[3] = 'o'
        //       Result: "uzo"
        //   - For right byte 0:
        //       d = (0 >> 4) & 15 = 0                                  -> CONSONANTS[0] = 'b'
        //       e = 0 & 15 = 0                                         -> CONSONANTS[0] = 'b'
        //       Encoded pair: "b-b"
        //   - Update checksum: ((1*5) + (255*7) + 0) % 36 = (5 + 1785) % 36 = 26.
        //   - No remainder; call even_partial(26):
        //       a = 26 % 6 = 2                                         -> VOWELS[2] = 'i'
        //       MID = "x"
        //       c = 26 / 6 = 4                                         -> VOWELS[4] = 'u'
        //       Result: "ixu"
        //   - Append trailer "x".
        //
        // Final expected encoding: "x" + "uzo" + "b-b" + "ixu" + "x" = "xuzob-bixux"
        let data = [255, 0];
        assert_eq!(inner(&data), "xuzob-bixux");
    }

    #[test]
    fn test_encoder_custom_odd() {
        // Test encoding for a custom 3-byte input.
        //
        // For data = [255, 0, 1]:
        //   - Process the first pair [255, 0] as in the previous test:
        //       Encoded so far: "xuzob-b" with updated checksum 26.
        //   - Remainder: odd_partial(1, 26):
        //       For raw_byte 1:
        //         a = (((1 >> 6) & 3) + 26) % 6 = (0 + 26) % 6 = 2     -> VOWELS[2] = 'i'
        //         b = (1 >> 2) & 15 = 0                                -> CONSONANTS[0] = 'b'
        //         c = ((1 & 3) + (26/6)) % 6 = (1 + 4) % 6 = 5         -> VOWELS[5] = 'y'
        //       Result: "iby"
        //   - Append trailer "x".
        //
        // Final expected encoding: "xuzob-b" + "iby" + "x" = "xuzob-bibyx"
        let data = [255, 0, 1];
        assert_eq!(inner(&data), "xuzob-bibyx");
    }

    #[test]
    fn test_encoder_multiple_pairs() {
        // Test encoding for a 4-byte input (two pairs).
        //
        // For data = [0, 0, 0, 0]:
        //   First pair [0, 0]:
        //     - odd_partial(0, 1) yields "eba".
        //     - Pair encoding gives "b-b".
        //     - Checksum becomes ((1*5) + (0*7) + 0) % 36 = 5.
        //     - Encoded so far: "xebab-b".
        //
        //   Second pair [0, 0]:
        //     - odd_partial(0, 5):
        //         a = (0 + 5) % 6 = 5      -> VOWELS[5] = 'y'
        //         b = 0                    -> CONSONANTS[0] = 'b'
        //         c = (0 + 0) % 6 = 0      -> VOWELS[0] = 'a'
        //         Result: "yba"
        //     - Pair encoding gives "b-b".
        //     - Update checksum: ((5*5) + 0 + 0) % 36 = 25.
        //     - Appended part: "ybab-b".
        //
        //   Since there's no remainder, even_partial(25):
        //     - a = 25 % 6 = 1             -> VOWELS[1] = 'e'
        //     - MID = "x"
        //     - c = 25 / 6 = 4             -> VOWELS[4] = 'u'
        //     - Result: "exu"
        //
        //   Append trailer "x".
        //
        // Final expected encoding: "xebab-b" + "ybab-b" + "exu" + "x" = "xebab-bybab-bexux"
        let data = [0, 0, 0, 0];
        assert_eq!(inner(&data), "xebab-bybab-bexux");
    }

    #[test]
    fn test_odd_partial() {
        // odd_partial(raw_byte, checksum, buf) appends three characters:
        //   a = (((raw_byte >> 6) & 3) + checksum) % 6     -> from VOWELS
        //   b = (raw_byte >> 2) & 15                       -> from CONSONANTS
        //   c = ((raw_byte & 3) + (checksum / 6)) % 6      -> from VOWELS
        let mut buf = String::new();

        // Test with raw_byte = 0 and checksum = 1.
        // a = (0 + 1) % 6 = 1  -> VOWELS[1] = 'e'
        // b = 0                -> CONSONANTS[0] = 'b'
        // c = (0 + 0) % 6 = 0  -> VOWELS[0] = 'a'
        odd_partial(0, 1, &mut buf);
        assert_eq!(buf, "eba");

        buf.clear();
        // Test with raw_byte = 255 and checksum = 1.
        // raw_byte 255 (binary 11111111):
        //   a = (((255 >> 6) & 3) + 1) % 6 = ((3 + 1) % 6) = 4     -> VOWELS[4] = 'u'
        //   b = (255 >> 2) & 15 = (63 & 15) = 15                   -> CONSONANTS[15] = 'z'
        //   c = ((255 & 3) + (1/6)) % 6 = (3 + 0) % 6 = 3          -> VOWELS[3] = 'o'
        odd_partial(255, 1, &mut buf);
        assert_eq!(buf, "uzo");
    }

    #[test]
    fn test_even_partial() {
        // even_partial(checksum, buf) appends three characters:
        //   a = checksum % 6       -> from VOWELS
        //   MID (a literal "x")
        //   c = checksum / 6       -> from VOWELS
        let mut buf = String::new();

        // For checksum = 5:
        //   a = 5 % 6 = 5      -> VOWELS[5] = 'y'
        //   c = 5 / 6 = 0      -> VOWELS[0] = 'a'
        // So even_partial(5) should append "yxa".
        even_partial(5, &mut buf);
        assert_eq!(buf, "yxa");
    }
}
