extern crate whatzip;

use whatzip::ArchiveType::*;
use whatzip::detect_archive;
use std::fs::File;

macro_rules! assert_detects {
    ( $filename:expr , $res:expr) => {{
        match File::open($filename) {
            Ok(mut f) => {
                assert_eq!(detect_archive(&mut f), $res);
            },
            Err(e) => assert!(false, format!("Error opening file: {}", e))
        }
    }};
}

#[test]
fn detect_zip() {
    assert_detects!("samples/src.zip" , Some(Zip));
}

#[test]
fn detect_gzip() {
    assert_detects!("samples/testg" , Some(Gzip{tar: false}));
}

#[test]
fn detect_tar_gz() {
    assert_detects!("samples/src.tar.gz", Some(Gzip{tar: true}));
}

#[test]
fn detect_tar() {
    assert_detects!("samples/src.tar", Some(Tar));
}

#[test]
fn non_archive() {
    assert_detects!("Cargo.toml", None);
}
