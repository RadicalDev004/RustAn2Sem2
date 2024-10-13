fn main() {
    // let mut s = String::from("");
    // let mut i = 0;
    // while i < 26 {
    //     let c = (i as u8 + 'a' as u8) as char;
    //     add_chars_n_ref(&mut s, c, 26-i);
    //     s = add_chars_n(s, c, 26 - i);

    //     i += 1;
    // }

    // print!("{}", s);

    let mut S : String = String::from("");

    add_space(&mut S, 40);
    add_str(&mut S, "I");
    add_space(&mut S, 1);
    add_str(&mut S, "💚\n");

    add_space(&mut S, 40);
    add_str(&mut S, "RUST\n");

    add_space(&mut S, 4);
    add_str(&mut S, "Most");
    add_space(&mut S, 12);
    add_str(&mut S, "crate");
    add_space(&mut S, 6);
    add_integer(&mut S, 306437968);
    add_space(&mut S, 11);
    add_str(&mut S, "and");
    add_space(&mut S, 5);
    add_str(&mut S, "latest");
    add_space(&mut S, 9);
    add_str(&mut S, "is\n");

    add_space(&mut S, 9);
    add_str(&mut S, "downloaded");
    add_space(&mut S, 8);
    add_str(&mut S, "has");
    add_space(&mut S, 13);
    add_str(&mut S, "downloads");
    add_space(&mut S, 5);
    add_str(&mut S, "the");
    add_space(&mut S, 8);
    add_str(&mut S, "version");
    add_space(&mut S, 4);
    add_float(&mut S, 2.038, 3);
    add_str(&mut S, ".");

    println!("{}", S);
}

fn add_chars_n(s : String, c : char, n : u32) -> String {
    let mut st : String = String::from(s);
    for j in 1..=n {
        st.push(c);
    }
    st
}

fn add_chars_n_ref(s : &mut String, c : char, n : u32) {
    for j in 1..=n {
        s.push(c);
    }
}

fn add_space(s : &mut String, n : u32) {
    for j in 1..=n {
        s.push(' ');
    }
}

fn add_str2(s : &mut String, st : String) {
    s.push_str(&st[0..st.len()]);
}

fn add_str(s : &mut String, st : &str) {
    s.push_str(st);
}

fn add_integer(s : &mut String, i : u32) {
    let mut tmp : String = String::from("");
    let mut cnt = 0;
    let mut j = i;

    while j > 0 {
        tmp.push(('0' as u8 + (j%10) as u8) as char);
        cnt+=1;
        
        j/=10;

        if cnt % 3 == 0 && j > 0 {
            tmp.push('_');
        }
    }

    for k in (0..tmp.len()).rev() {
        s.push(tmp.chars().nth(k).unwrap());
    }
}

fn add_float(s : &mut String, i : f32, mut prc : i32) {
    let mut tmp : String = String::from("");
    let mut j = i as u32;

    //before .
    while j > 0 {
        tmp.push(('0' as u8 + (j%10) as u8) as char);
        j/=10;
    }

    for k in (0..tmp.len()).rev() {
        s.push(tmp.chars().nth(k).unwrap());
    }
    s.push('.');

    //after .
    let mut l = i;

    while l != (l as u32) as f32 {
        l *= 10 as f32;
        s.push(('0' as u8 + (l as u32 %10) as u8) as char);
        prc = prc -1;
        if prc == 0 {
            break;
        }
    }
}