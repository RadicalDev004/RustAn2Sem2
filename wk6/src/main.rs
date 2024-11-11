use std::borrow::BorrowMut;
use std::{io, fs};
use std::cell::Cell;
use std::collections::HashMap;

trait commandTrait {
    fn get_name(&self) -> &str;
    fn exec(&mut self, args: &[&str]);
}

struct PingCommand;
struct CountCommand;
struct TimesCommand {
    count: u32,
}

impl commandTrait for PingCommand{
    fn get_name(&self) -> &str {
        "ping"
    }
    fn exec(&mut self, args: &[&str]){
        println!("pong");
    }
}

impl commandTrait for CountCommand{
    fn get_name(&self) -> &str {
        "count"
    }
    fn exec(&mut self, args: &[&str]){
        println!("counted {} args", args.len());
    }
}

impl TimesCommand {
    fn new() -> Self {
        TimesCommand {
            count: 0,
        }
    }
}
impl commandTrait for TimesCommand{
    fn get_name(&self) -> &str {
        "times"
    }
    fn exec(&mut self, args: &[&str]){
        self.count = self.count + 1;
        
        println!("Called times {} time(s)", self.count);
    }
}

struct Terminal {
    commands: HashMap<String, Box<dyn commandTrait>>,
}
impl Terminal {
    fn new() -> Self {
        Terminal { commands: HashMap::new(), }
    }
    fn register(&mut self, command: Box<dyn commandTrait>) {
        self.commands.insert(command.get_name().to_string(), command);
    }
    fn run(&mut self) -> Result<(), io::Error> {

        let mut input = fs::read_to_string("src/input.txt")?;
        for line in input.split("\n") {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            
            if parts.is_empty() {
                continue;
            }

            let command_name = parts[0].to_lowercase();
            let args = &parts[1..];


            if command_name == "stop" {
                println!("Stopping terminal...");
                break;
            }

            match self.commands.get_mut(&command_name) {
                Some(command) => command.exec(args),
                None => println!("Unknown command: '{}'", command_name),
            }
        }
    
            

        Ok(())
    }
}

fn main() {
    let mut terminal = Terminal::new();

    terminal.register(Box::new(PingCommand {}));
    terminal.register(Box::new(CountCommand {}));
    terminal.register(Box::new(TimesCommand { count: 0 }));

    terminal.run();
}

