fn is_prime(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    for i in 2..=((n as f64).sqrt() as u32) {
        if n % i == 0 {
            return false;
        }
    }
    true
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

fn are_coprime(a: u32, b: u32) -> bool {
    gcd(a, b) == 1
}

fn sing_bottles() {
    for i in (1..=99).rev() {
        let bottle_word = if i == 1 { "bottle" } else { "bottles" };
        let next_bottle = if i == 1 { "no more bottles" } else { &format!("{} bottles", i - 1)[..] }; // Using slicing to get &str

        println!("{} {} of beer on the wall, {} {} of beer.", i, bottle_word, i, bottle_word);
        println!("Take one down, pass it around, {} of beer on the wall.\n", next_bottle);
    }
    println!("No more bottles of beer on the wall.");
    println!("Go to the store and buy some more, 99 bottles of beer on the wall.");
}

fn main() {
    for i in 0..100 {
        if is_prime(i) {
            println!("{}", i);
        }
    }
    for i in 0..100 {
        for j in 0..100 {
            if i != j && are_coprime(i, j) {
                println!("({}, {})", i, j);
            }
        }
    }
    sing_bottles();
}
