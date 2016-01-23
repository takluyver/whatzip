use std::env;
use std::process;
use std::fs::File;
use std::io;
use std::io::prelude::*;

enum ArchiveType {
    Zip,
    Gzip,
    Unknown,
}

impl ArchiveType {
    fn typical_extension(&self) -> Option<&str> {
        match self {
            &ArchiveType::Zip => Some(".zip"),
            &ArchiveType::Gzip => Some(".gz"),
            &ArchiveType::Unknown => None
        }
    }
}

fn check_zip(f: &mut File) -> bool {
    if f.seek(io::SeekFrom::End(-22)).is_err() {
        return false;
    }
    let mut end_buf = Vec::new();
    if f.read_to_end(&mut end_buf).is_err() {
        return false;
    }

    // TODO: If these bits fail, it may still be a zip file with an
    // archive comment.
    if &end_buf[.. 4] != b"PK\x05\x06" {
        return false;
    }
    if &end_buf[(22 - 2)..] != b"\x00\x00" {
        return false
    }
    true
}

fn detect_archive(f: &mut File) -> ArchiveType {
    if check_zip(f) {
        ArchiveType::Zip
    } else {
        f.seek(io::SeekFrom::Start(0)).unwrap();
        let mut buffer = vec![0; 32];
        match f.read(&mut buffer) {
            Ok(size) => {
                let buf_read = &buffer[..size];
                if buf_read.starts_with(b"\x1f\x8b") {
                    ArchiveType::Gzip
                } else {
                    ArchiveType::Unknown
                }
            }
            Err(_) => {
                ArchiveType::Unknown
            }
        }
    }
}

fn main() {
    match env::args().nth(1) {
        Some(s) => {
            match File::open(&s) {
                Ok(mut f) => {
                    println!("OK");
                    let at = detect_archive(&mut f);
                    match at.typical_extension() {
                        Some(ext) => println!("Detected {} file", ext),
                        None => println!("Unknown file type")
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
