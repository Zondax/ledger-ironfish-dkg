use lexical_core::FormattedSize;

use crate::parser::ParserError;

macro_rules! num_to_str {
    // we can use a procedural macro to "attach " the type name to the function name
    // but lets do it later.
    ($int_type:ty, $_name: ident) => {
        pub fn $_name(number: $int_type, output: &mut [u8]) -> Result<&mut [u8], ParserError> {
            if output.len() < <$int_type>::FORMATTED_SIZE_DECIMAL {
                return Err(ParserError::UnexpectedBufferEnd);
            }

            if number == 0 {
                output[0] = b'0';
                return Ok(&mut output[..1]);
            }

            let mut offset = 0;
            let mut number = number;
            while number != 0 {
                let rem = number % 10;
                output[offset] = b'0' + rem as u8;
                offset += 1;
                number /= 10;
            }

            // swap values
            let len = offset;
            let mut idx = 0;
            while idx < offset {
                offset -= 1;
                output.swap(idx, offset);
                idx += 1;
            }

            Ok(&mut output[..len])
        }
    };
}

num_to_str!(u64, u64_to_str);
num_to_str!(u32, u32_to_str);
num_to_str!(u8, u8_to_str);

/// Return the len of the string until null termination
fn strlen(bytes: &[u8]) -> usize {
    bytes.split(|&n| n == 0).next().unwrap_or(bytes).len()
}

#[inline(never)]
/// Converts an integer number string
/// to a fixed point number string, in place
///
/// Returns Ok(subslice) which is the subslice with actual content,
/// trimming excess bytes
pub fn intstr_to_fpstr_inplace(s: &mut [u8], decimals: usize) -> Result<&mut [u8], ParserError> {
    //find the length of the string
    // if no 0s are found then the entire string is full with digits
    // so we return error
    let mut num_chars = strlen(s);

    if num_chars == s.len() {
        #[cfg(test)]
        std::println!("num_chars == s.len");
        return Err(ParserError::BufferFull);
    }

    if s.is_empty() {
        #[cfg(test)]
        std::println!("s.is_empty");
        return Err(ParserError::UnexpectedBufferEnd);
    }

    //empty input string
    // let's just write a 0
    if num_chars == 0 {
        s[0] = b'0';
        num_chars = 1;
    }

    let mut first_digit_idx = None;
    //check that all are ascii numbers
    // and first the first digit
    let number_ascii_range = b'0'..=b'9';
    for (i, c) in s[..num_chars].iter_mut().enumerate() {
        if !number_ascii_range.contains(c) {
            #[cfg(test)]
            std::println!("non_ascii number");
            return Err(ParserError::UnexpectedValue);
        }

        //just find the first digit
        if *c != b'0' {
            first_digit_idx = Some(i);
            break;
        }
    }

    //if we have a first digit
    if let Some(idx) = first_digit_idx {
        //move first_digit.. to the front
        s.copy_within(idx.., 0);

        //zero out the remaining
        s[num_chars - idx..].fill(0);

        //same as strlen(s)
        //we know where the string ends
        num_chars -= idx;
    } else {
        //if the first digit wasn't found
        // then it's just all 0s
        //we trim all the 0s after the first one
        s[1..].fill(0);
    }

    //we can return early if we have no decimals
    if decimals == 0 {
        #[cfg(test)]
        std::println!("decimals == 0");
        num_chars = strlen(s);
        return Ok(&mut s[..num_chars]);
    }

    // Now insert decimal point

    //        0123456789012     <-decimal places
    //        abcd              < numChars = 4
    //                 abcd     < shift
    //        000000000abcd     < fill
    //        0.00000000abcd    < add decimal point

    if num_chars < decimals + 1 {
        #[cfg(test)]
        std::println!("num_chars < decimals + 1");
        // Move to end
        let padding = decimals - num_chars + 1;
        s.copy_within(..num_chars, padding);

        //fill the front with zeros
        s[..padding].fill(b'0');
        num_chars = strlen(s);
    }

    // add decimal point
    let point_position = num_chars - decimals;
    //shift content
    // by 1 space after point
    s.copy_within(
        point_position..point_position + decimals, //from: point to all the decimals
        point_position + 1,                        //to: just after point
    );
    s[point_position] = b'.';

    num_chars = strlen(s);

    // skip the trailing zeroes
    // for example 0.00500, so the last two
    // zeroes are completely irrelevant,
    // the same for 2000.00, the fixed point and the zero
    // are not important
    let mut len = num_chars;
    // skip characters before the decimal point
    for x in s[point_position..num_chars].iter().rev() {
        if *x == b'0' {
            len -= 1;
        } else if *x == b'.' {
            // this means everything after
            // the decimal point is zero
            // so remove the decimal point as well
            len -= 1;
            break;
        } else {
            break;
        }
    }

    // recalculate the new len after the filtering above
    let len = num_chars - (num_chars - len);

    Ok(&mut s[..len])
}

pub fn token_to_fp_str(
    value: u64,
    out_str: &mut [u8],
    decimals: usize,
) -> Result<&mut [u8], ParserError> {
    // the number plus '0.'
    if out_str.len() < u64::FORMATTED_SIZE_DECIMAL + 2 {
        return Err(ParserError::UnexpectedBufferEnd);
    }

    u64_to_str(value, &mut out_str[..])?;

    intstr_to_fpstr_inplace(out_str, decimals).map_err(|_| ParserError::UnexpectedError)
}
