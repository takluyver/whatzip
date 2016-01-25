extern crate whatzip;

use whatzip::ArchiveType::*;
use whatzip::detect_archive;
use std::fs::File;

#[test]
fn detect_zip() {
    match File::open("samples/src.zip") {
        Ok(mut f) => {
            assert_eq!(detect_archive(&mut f), Some(Zip));
        },
        Err(e) => assert!(false, e)
    }
}
