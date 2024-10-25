use core::unicode::conversions::to_upper;

fn main() {

    let mut i = 62345;
    let mut condition = true;
    while condition {
        match next_prime(i){
            Some(x) => {
                println!("Gt next prime : {}", x); 
                i = x;
            },
            None => {
                println!("Reached limit at {}.", i); 
                condition = false;
            },
        }
        
    }
}

enum CharErr{
    NotAscii,
    NotDigit,
    NotBase16Digit,
    NotLetter,
    NotPrintable,
}

fn to_uppercase(ch : char) -> Result<char, CharErr> {
    if ch < 'A' || ch > 'z' {
        return Err(CharErr::NotLetter);
    }
    let c = ch.to_ascii_uppercase();
    return Ok(c);
}
fn to_lowercase(ch : char) -> Result<char, CharErr> {
    if ch < 'A' || ch > 'z' {
        return Err(CharErr::NotLetter);
    }
    let c = ch.to_ascii_lowercase();
    return Ok(c);
}

fn print_char(ch : char) -> Result<char, CharErr> {
    
    if !ch.is_control() {
        return Err(CharErr::NotPrintable);
    }
    println!("{}", ch);
    Ok(ch)
}

fn char_to_number(ch : char) -> Result<u32, CharErr> {
    if !ch.is_ascii() {
        return Err(CharErr::NotAscii);
    }
    if !ch.is_digit(10) {
        return Err(CharErr::NotDigit);
    }
    
    match ch.to_digit(10) {
        Some(result) => Ok(result),
        None => return Err(CharErr::NotDigit),
    }
}

fn char_to_number_hex(ch : char) -> Result<u32, CharErr> {
    if !ch.is_ascii() {
        return Err(CharErr::NotAscii);
    }
    if !ch.is_digit(16) {
        return Err(CharErr::NotBase16Digit);
    }
    
    match ch.to_digit(16) {
        Some(result) => Ok(result),
        None => return Err(CharErr::NotDigit),
    }
}

fn print_error(err : CharErr) {
        match err {
            CharErr::NotAscii => println!("Character is not part of Ascii"),
            CharErr::NotDigit => println!("Character is not a digit"),
            CharErr::NotBase16Digit => println!("Character is not a base 16 digit"),
            CharErr::NotLetter => println!("Character is not a letter"),
            CharErr::NotPrintable => println!("Character is not printable"),
        }
}

enum OperationError{
    Overflow,
}

fn check_overflow(value : u16, increment : u16) -> bool {
    match value.checked_add(increment) {
        Some(result) => return true,
        None => return false,
    }
}

fn check_addition_u32_regular(value : u32, increment : u32) -> bool {
    match value.checked_add(increment) {
        Some(result) => return true,
        None => panic!("Result of {} incremented by {} excedes u32", value, increment),
    }
}
fn check_multiplication_u32_regular(value : u32, increment : u32) -> bool {
    match value.checked_mul(increment) {
        Some(result) => return true,
        None => panic!("Result of {} multiplied by {} excedes u32", value, increment),
    }
}


fn check_addition_u32_Result(value : u32, increment : u32) -> Result<u32, OperationError> {
    match value.checked_add(increment) {
        Some(result) => return Ok(result),
        None => return Err(OperationError::Overflow),
    }
}
fn check_multiplication_u32_Result(value : u32, increment : u32) -> Result<u32, OperationError> {
    match value.checked_mul(increment) {
        Some(result) => return Ok(result),
        None => return Err(OperationError::Overflow),
    }
}
fn showcase_overflow_with_propagation(value : u32, increment : u32) -> Result<u32, OperationError>{
    let added_result = check_addition_u32_Result(value, increment)?;
    println!("Addition result: {}", added_result);

    let multiplied_result = check_multiplication_u32_Result(value, increment)?;
    println!("Multiplication result: {}", multiplied_result);

    Ok(multiplied_result)
}

fn next_prime(x: u16) -> Option<u16> {
    if !check_overflow(x, 1)
    {
        return None;
    }
    let mut i = x + 1;

    fn isPrim(n : u16) -> bool
    {
        if n < 2
        {
            return false;
        }
        
        (2..=(n as f64).sqrt() as u16).all(|j| n % j != 0)
    }

    while !isPrim(i) {
        if(!check_overflow(i, 1))
        {
            return None;
        }
        i = i+1;

    }
    Some(i)
}
