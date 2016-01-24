use std::env;
use std::process;
use std::fs::File;
use std::io;
use std::io::prelude::*;

enum ArchiveType {
    Zip,
    Tar,
    Gzip,
    Bzip2,
    Xz,
    SevenZ,
    MSCabinet,
    Unknown,
}

impl ArchiveType {
    fn typical_extension(&self) -> Option<&str> {
        match self {
            &ArchiveType::Zip => Some(".zip"),
            &ArchiveType::Tar => Some(".tar"),
            &ArchiveType::Gzip => Some(".gz"),
            &ArchiveType::Bzip2 => Some(".bz2"),
            &ArchiveType::Xz => Some(".xz"),
            &ArchiveType::SevenZ => Some(".7z"),
            &ArchiveType::MSCabinet => Some(".cab"),
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

fn check_tar(f: &mut Read) -> bool {
    let mut buf = [0; 265];
    match f.read(&mut buf) {
        Ok(n) => {
            if n < 263 {
                false
            } else {
                let sample = &buf[257..n];
                sample.starts_with(b"ustar\0") || sample.starts_with(b"ustar\x20\x20\0")
            }
        },
        Err(_) => false
    }
}

fn detect_archive(f: &mut File) -> ArchiveType {
    if check_zip(f) {
        ArchiveType::Zip
    } else {
        f.seek(io::SeekFrom::Start(0)).unwrap();
        if check_tar(f) {
            return ArchiveType::Tar
        }
        f.seek(io::SeekFrom::Start(0)).unwrap();
        let mut buffer = [0; 32];
        match f.read(&mut buffer) {
            Ok(size) => {
                let buf_read = &buffer[..size];
                if buf_read.starts_with(b"\x1f\x8b") {
                    ArchiveType::Gzip
                } else if buf_read.starts_with(b"BZh") {
                    ArchiveType::Bzip2
                } else if buf_read.starts_with(b"\xfd7zXZ\0") {
                    ArchiveType::Xz
                } else if buf_read.starts_with(b"7z\xbc\xaf\x27\x1c") {
                    ArchiveType::SevenZ
                } else if buf_read.starts_with(b"MSCF") {
                    ArchiveType::MSCabinet
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
