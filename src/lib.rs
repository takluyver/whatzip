extern crate flate2;
extern crate bzip2;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use flate2::read::GzDecoder;
use bzip2::reader::BzDecompressor;

#[derive(Debug, PartialEq, Eq)]
pub enum ArchiveType {
    Zip,
    Tar,
    Gzip {tar: bool},
    Bzip2 {tar: bool},
    Xz,
    SevenZ,
    MSCabinet,
}

impl ArchiveType {
    pub fn typical_extension(&self) -> &str {
        match self {
            &ArchiveType::Zip => ".zip",
            &ArchiveType::Tar => ".tar",
            &ArchiveType::Gzip{tar: false} => ".gz",
            &ArchiveType::Gzip{tar: true} => ".tar.gz",
            &ArchiveType::Bzip2{tar: false} => ".bz2",
            &ArchiveType::Bzip2{tar: true} => ".tar.bz2",
            &ArchiveType::Xz => ".xz",
            &ArchiveType::SevenZ => ".7z",
            &ArchiveType::MSCabinet => ".cab",
        }
    }

    pub fn decompress_cmd(&self, filename: &str) -> String {
        let f = escape_filename(filename);
        match self {
            &ArchiveType::Zip => format!("unzip {}", f),
            &ArchiveType::Tar => format!("tar -xf {}", f),
            &ArchiveType::Gzip{tar: false} => format!("gunzip {}", f),
            &ArchiveType::Gzip{tar: true} => format!("tar -xzf {}", f),
            &ArchiveType::Bzip2{tar: false} => format!("bunzip2 {}", f),
            &ArchiveType::Bzip2{tar: true} => format!("tar -xjf {}", f),
            &ArchiveType::Xz => format!("xz -d {}", f),
            &ArchiveType::SevenZ => format!("7z x {}", f),
            &ArchiveType::MSCabinet => format!("cabextract {}", f),
        }
    }
}

fn escape_filename(s: &str) -> String {
    for c in s.chars() {
        if c == ' ' || c == '\t' {
            return format!("\"{}\"", s.replace("\"", "\\\""));
        }
    }
    s.to_string()
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

pub fn detect_archive(f: &mut File) -> Option<ArchiveType> {
    if check_zip(f) {
        Some(ArchiveType::Zip)
    } else {
        f.seek(io::SeekFrom::Start(0)).unwrap();
        if check_tar(f) {
            return Some(ArchiveType::Tar)
        }
        f.seek(io::SeekFrom::Start(0)).unwrap();
        let mut buffer = [0; 32];
        match f.read(&mut buffer) {
            Ok(size) => {
                let buf_read = &buffer[..size];
                if buf_read.starts_with(b"\x1f\x8b") {
                    f.seek(io::SeekFrom::Start(0)).unwrap();
                    match GzDecoder::new(f) {
                        Ok(mut d) => Some(ArchiveType::Gzip{tar: check_tar(&mut d)}),
                        Err(_) => None,
                    }
                } else if buf_read.starts_with(b"BZh") {
                    f.seek(io::SeekFrom::Start(0)).unwrap();
                    let mut d = BzDecompressor::new(f);
                    return Some(ArchiveType::Bzip2{tar: check_tar(&mut d)});
                } else if buf_read.starts_with(b"\xfd7zXZ\0") {
                    Some(ArchiveType::Xz)
                } else if buf_read.starts_with(b"7z\xbc\xaf\x27\x1c") {
                    Some(ArchiveType::SevenZ)
                } else if buf_read.starts_with(b"MSCF") {
                    Some(ArchiveType::MSCabinet)
                } else {
                    None
                }
            }
            Err(_) => {
                None
            }
        }
    }
}
