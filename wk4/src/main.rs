use std::{io, fs};

fn main() {
    //P1();
    //P2();
    //P3();
    P4();
}


fn P1() -> Result<(), io::Error> {
    let s = fs::read_to_string("src/input.txt")?;
    let lines = s.split("\n");

    let mut maxChars : u32  = 0;
    let mut maxBytes : u32 = 0;
    let mut ChLine : &str = "";
    let mut ByLine : &str = "";

    for line in lines {
        if(line.len() as u32 > maxBytes) {
            maxBytes = line.len() as u32;
            ByLine = line;
        }
        if(line.chars().count() as u32 > maxChars) {
            maxChars = line.len() as u32;
            ChLine = line;
        }
    }

    println!("Longest line by characters: {}", ChLine);
    println!("Longest line by characters: {}", ByLine);
    Ok(())
}

fn P2() -> Result<(), io::Error> {
    let s = fs::read_to_string("src/input2.txt")?;
    let mut newCh : char = ' ';

    for ch in s.chars() {
        newCh = ' ';
        if(ch >= 'a' && ch <='z') {
            if(ch <= ('z' as u8 - 13) as char) {
                newCh = (ch as u8 + 13) as char;
            }
            else {
                let rest = 'z' as u8 - ch as u8;
                newCh = ('a' as u8 + 12 - rest) as char;
            }
        }
        else if(ch >= 'A' && ch <='Z') {
            if(ch <= ('Z' as u8 - 13) as char) {
                newCh = (ch as u8 + 13) as char;
            }
            else {
                let rest = 'Z' as u8 - ch as u8;
                newCh = ('A' as u8 + 12 - rest) as char;
            }
        }
        else if (!ch.is_ascii()) {
            println!("Char is not ascii");
            break;
        }
        print!("{}", if newCh != ' ' {newCh} else {ch});
    }

    Ok(())
}

fn P3() -> Result<(), io::Error> {
    
    let s = fs::read_to_string("src/input3.txt")?;
    let full = ["pentru", "pentru", "domnul", "doamna"];
    let abv = ["pt", "ptr", "dl", "dna"];

    let mut newInfo = String::from("");

    for word in s.split(" ") {
        let mut i = 0;
        let mut found = false;
        for repl in abv {
            if(word == repl) {
                newInfo += full[i];
                found = true;
                break;
            }
            i=i+1;
        }
        if (!found) {
            newInfo += word;
        }
        newInfo += " ";
    }
    fs::write("src/input3.txt", "");
    fs::write("src/input3.txt", newInfo);

   

    Ok(())
}

fn P4()-> Result<(), io::Error> {
    let s = fs::read_to_string("C:\\Windows\\System32\\drivers\\etc\\hosts")?;
    //println!("{}", s);
    for line in s.split("\n") {
        if(line.chars().nth(0).unwrap() == '#') {
            continue;
        }
        let ln : Vec<&str> = line.split_whitespace().collect();
        println!("{} => {}", ln[1], ln[0]);
    }
    Ok(())
}