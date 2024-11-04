use std::{io, fs};
use serde_derive::{Deserialize, Serialize};

fn main() {
    
}

#[derive(Debug, Deserialize, Clone)]
struct Student <'a> {
    name: &'a str,
    phone: &'a str,
    age: u32
}
fn P1() -> Result<(), io::Error> {
    let s = fs::read_to_string("src/input.txt")?;
    let lines = s.split("\n");
    let mut studenti: Vec<Student> = Vec::new();

    for line in lines {
        let segment: Vec<&str> = line.split(',').collect();
        let name = segment[0];
        let phone= segment[1];
        println!("{}", segment[2]);
        let age: u32 = match segment[2].trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Failed to parse age for line: {}", line);
                0
            }
        };
        println!("{}", age);

        studenti.push(Student {
            name,
            phone,
            age
        });
    }

    let mut oldest : Student = Student {name : "", phone : "", age :0};
    let mut youngest : Student = Student {name : "", phone : "", age :1000};

    for student in studenti {
        if(student.age > oldest.age) {
            oldest = student.clone();

        }
        if(student.age < youngest.age) {
            youngest = student.clone();
        }
    }

    println!("Youngest {} {} {}", youngest.name, youngest.phone, youngest.age);
    println!("Oldest {} {} {}", oldest.name, oldest.phone, oldest.age);

    Ok(())
}

#[derive(Copy, Clone, Debug)]
struct Canvas {
    canvas: [[char; 100]; 55],
}

fn new_canvas() -> Canvas {
    let mut canv = [[' '; 100]; 55];
    Canvas { 
        canvas : canv,
    }
}

fn set_pixels(mut canvas: Canvas, pixels: &[(usize, usize, u8)]) {
    for &(x, y, value) in pixels {
        if x < 55 && y < 100 {
            canvas.canvas[x][y] = value as char;
        }
    }
}

fn print(canvas: Canvas) {
    for row in &canvas.canvas {
        let line: String = row.iter().collect();
        println!("{}", line);
    }
}

fn P3() -> Result<(), io::Error> {
    let s = fs::read_to_string("src/input2.txt")?;
    let lines = s.split("\n");
    let mut studenti: Vec<Student> = Vec::new();

    for line in lines {
        
        let s: Student = serde_json::from_str(&line).unwrap();
        
        studenti.push(s);
    }

    let mut oldest : Student = Student {name : "", phone : "", age :0};
    let mut youngest : Student = Student {name : "", phone : "", age :1000};

    for student in studenti {
        if(student.age > oldest.age) {
            oldest = student.clone();

        }
        if(student.age < youngest.age) {
            youngest = student.clone();
        }
    }

    println!("Youngest {} {} {}", youngest.name, youngest.phone, youngest.age);
    println!("Oldest {} {} {}", oldest.name, oldest.phone, oldest.age);

    Ok(())
}