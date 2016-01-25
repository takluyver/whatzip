extern crate flate2;
extern crate bzip2;

use std::env;
use std::process;
use std::fs::File;

mod lib;

fn main() {
    match env::args().nth(1) {
        Some(s) => {
            match File::open(&s) {
                Ok(mut f) => {
                    println!("OK");
                    match lib::detect_archive(&mut f) {
                        None => println!("Unknown file type"),
                        Some(at) => {
                            println!("Detected {} file", at.typical_extension());
                            println!("To unpack, run:");
                            println!("  {}", at.decompress_cmd(&s));
                        }
                    }
                },
                Err(_) => {
                    println!("Failed to open {}", s);
                    process::exit(1);
                }
            }
        },
        None => {
            println!("No argument!");
            process::exit(2);
        },
    }
}
