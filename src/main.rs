// Licensing: See the LICENSE file

use std::fs::{write,File};
use std::io::prelude::*;

fn print_help() {
    println!("Usage: ./untamed func filename");
    println!("Where func is one of:");
    println!("\thelp - prints this message");
    println!("\tdump_bootlogs - dumps boot logs (TA stores up to ten of these)");
    println!("\tdump_sqlitedb - dumps the internal SQLite database");
    println!("\tshow_buildid - shows build number");
    println!("\tshow_serial - shows serial number");
}

static BOOTLOG_OFFSET: [usize; 11] = [
    0,       // 0-element, ignore
    0x2A22E, // 1
	0x2DA22, // 2
	0x31CEE, // 3
	0x3542A, // 4
	0x38C46, // 5
	0x3C7A2, // 6
	0x65412, // 7
	0x68C2E, // 8
	0x6C78A, // 9
	0x70A2E, // 10
];

fn dump_bootlogs(ta_file_content: &[char]) {
    const BOOTLOG_SIZE: usize = 14309;

    // We have 10 bootlogs but want to keep the indices sane
    let mut bootlogs: [String; 11] = Default::default();
    let mut temp_filename: String;
    for i in 1..11 {
        println!("Dumping bootlog {} at {}..", i, format!("{:X}", BOOTLOG_OFFSET[i]));
        bootlogs[i] = ta_file_content[BOOTLOG_OFFSET[i]..BOOTLOG_OFFSET[i]+BOOTLOG_SIZE].iter().collect();
        temp_filename = format!("bootlogs/bootlog{}.txt", i);
        println!("writing to {}", temp_filename);
        write(temp_filename, &bootlogs[i]).expect("Could not dump bootlog..");
    }
}

fn show_build(ta_file_content: &[char]) {
    const VERSION_OFFSET: usize = 0x7B4;
    // 32 is an educated guess, it's actually 29 on Tama-Akari
    const VERSION_SIZE: usize = 32;

    let build_id: String = ta_file_content[VERSION_OFFSET..VERSION_OFFSET+VERSION_SIZE].iter().collect();
    println!("Image version: {}", build_id);
}

fn show_serialno(ta_file_content: &[char]) {
    const SERIAL_OFFSET: usize = 0x600B4;
    const SERIAL_SIZE: usize = 10;

    let serial_no: String = ta_file_content[SERIAL_OFFSET..SERIAL_OFFSET+SERIAL_SIZE].iter().collect();
    println!("Serial no.: {}", serial_no);
}

fn dump_sqlitedb(ta_file_content: &[char]) {
    const SQLITEDB_OFFSET: usize = 0x20044;
    const SQLITEDB_HEADER_SIZEVAL_OFF: usize = 16;

    let sqlitedb_len: String = ta_file_content[SQLITEDB_OFFSET+SQLITEDB_HEADER_SIZEVAL_OFF..SQLITEDB_OFFSET+SQLITEDB_HEADER_SIZEVAL_OFF+2].iter().collect();
    // Swap byte order to LE
    let mut sqlitedb_len: usize = (sqlitedb_len.as_bytes()[0] + (sqlitedb_len.as_bytes()[1]<<2)) as usize;
    println!("SQLite DB size: 2^{:?} ({} B)", sqlitedb_len, (2 as i32).pow(sqlitedb_len as u32));

    sqlitedb_len = (2 as usize).pow(sqlitedb_len as u32);

    let mut sqlitedb: Vec<char> = Default::default();
    sqlitedb.extend(ta_file_content[SQLITEDB_OFFSET..SQLITEDB_OFFSET+sqlitedb_len].iter());
    let sqlitedb: Vec<u8> = sqlitedb.iter().map(|c| *c as u8).collect::<Vec<_>>();

    write("sqlite.db", sqlitedb).expect("Could not dump SQLite DB..");

    let sqlitedb_file = match File::open("sqlite.db") {
        Ok(sqlitedb_file) => sqlitedb_file,
        Err(e) => panic!("Could not access sqlite.db! err: {}", e),
    };

    if sqlitedb_file.metadata().unwrap().len() as usize != sqlitedb_len {
        panic!("SQLite DB file size mismatch! Got {}, expected {}", sqlitedb_file.metadata().unwrap().len(), sqlitedb_len)
    }
    
    println!("Saved results to sqlite.db!");
}

fn main() {
    const TA_EXPECTED_SIZE_BYTES: usize = 2097152;
    let action: String = std::env::args().nth(1).expect("No action given.");
    let filename: String = std::env::args().nth(2).expect("No filename given.");
    let num_args: usize = std::env::args().len();

    // We are helpful around here. Want help? Get help.
    if action == "help" { print_help(); std::process::exit(0); };

    // Not enough arguments to mess with TA -> print help
    if num_args < 3 { print_help(); std::process::exit(0); };

    println!("Opening file: {}", filename);

    let mut ta_file = match File::open(filename) {
        Ok(ta_file) => ta_file,
        Err(e) => panic!("Could not open file: {:?}", e),
    };

    match ta_file.metadata().unwrap().len() as usize {
        TA_EXPECTED_SIZE_BYTES => println!{"TA size in tact, proceeding.."},
        _ => panic!("TA size mismatch, got: {} expected: {}. Is your dump corrupted?",
            ta_file.metadata().unwrap().len(), TA_EXPECTED_SIZE_BYTES),
    }

    let mut content: Vec<u8> = Vec::new();
    match ta_file.read_to_end(&mut content) {
        Err(e) => panic!("Unable to read contents of the file: {:?}", e),
        _ => (),
    }

    if content[0] != 0xC1 && content[1] != 0xE9 {
        println!("TA header mismatch!");
        std::process::exit(0);
    }

    let content: Vec<char> = content.iter().map(|b| *b as char).collect::<Vec<_>>();

    match action.as_str() {
        "dump_bootlogs" => dump_bootlogs(&content),
        "dump_sqlitedb" => dump_sqlitedb(&content),
        "show_build" => show_build(&content),
        "show_serialno" => show_serialno(&content),
        _ => println!("Unknown operation '{}'", action), // How did we get here?
    }
}
