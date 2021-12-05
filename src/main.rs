extern crate libdotpm;

use libdotpm::basic::*;

fn main() {
    args: Vec<String> = env::args().collect();
    if args.is_empty() {
        println!("Usage: dot <command> <package>");
        return;
    }
    match args[1] {
        "install" => {
            install(&args[2]);
        }
        "remove" => {
            uninstall(&args[2]);
        }
    }
}