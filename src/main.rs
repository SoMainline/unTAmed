// Licensing: See the LICENSE file

use clap::{AppSettings, Clap};
use std::fs::{write, File};
use std::io::{prelude::*, Result};
use std::path::PathBuf;

#[derive(Clap)]
enum Func {
    /// Dump boot logs (TA stores up to ten of these)
    DumpBootlogs,
    /// Dump the internal SQLite database
    DumpSqlitedb,
    /// Show build number
    ShowBuildid,
    /// Show serial number
    ShowSerial,
}

/// unTAmed is an OSS tool for inspecting the data contained inside the TA (Trim Area) as found on SoMC devices.
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// The TA file to open
    file: PathBuf,
    /// The action to perform
    #[clap(subcommand)]
    func: Func,
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

fn read_ta(ta_file_content: &[char], offset: usize, length: usize) -> String {
    return ta_file_content[offset..offset + length].iter().collect();
}

fn dump_bootlogs(ta_file_content: &[char]) {
    const BOOTLOG_SIZE: usize = 14309;

    // We have 10 bootlogs but want to keep the indices sane
    let mut bootlogs: [String; 11] = Default::default();
    let mut temp_filename: String;
    for i in 1..11 {
        println!(
            "Dumping bootlog {} at {}..",
            i,
            format!("{:X}", BOOTLOG_OFFSET[i])
        );
        bootlogs[i] = read_ta(ta_file_content, BOOTLOG_OFFSET[i], BOOTLOG_SIZE);
        temp_filename = format!("bootlogs/bootlog{}.txt", i);
        println!("writing to {}", temp_filename);
        write(temp_filename, &bootlogs[i]).expect("Could not dump bootlog..");
    }
}

fn show_build(ta_file_content: &[char]) {
    const VERSION_OFFSET: usize = 0x7B4;
    // 32 is an educated guess, it's actually 29 on Tama-Akari
    const VERSION_SIZE: usize = 32;

    let build_id: String = read_ta(ta_file_content, VERSION_OFFSET, VERSION_SIZE);
    println!("Image version: {}", build_id);
}

fn show_serialno(ta_file_content: &[char]) {
    const SERIAL_OFFSET: usize = 0x600B4;
    const SERIAL_SIZE: usize = 10;

    let serial_no: String = read_ta(ta_file_content, SERIAL_OFFSET, SERIAL_SIZE);
    println!("Serial no.: {}", serial_no);
}

fn dump_sqlitedb(ta_file_content: &[char]) -> Result<()> {
    const SQLITEDB_OFFSET: usize = 0x20044;
    const SQLITEDB_HEADER_SIZEVAL_OFF: usize = 16;

    let sqlitedb_len: String = read_ta(
        ta_file_content,
        SQLITEDB_OFFSET + SQLITEDB_HEADER_SIZEVAL_OFF,
        2,
    );
    // Swap byte order to LE
    let mut sqlitedb_len: usize =
        (sqlitedb_len.as_bytes()[0] + (sqlitedb_len.as_bytes()[1] << 2)) as usize;
    println!(
        "SQLite DB size: 2^{:?} ({} B)",
        sqlitedb_len,
        (2i32).pow(sqlitedb_len as u32)
    );

    sqlitedb_len = (2usize).pow(sqlitedb_len as u32);

    let mut sqlitedb: Vec<char> = Default::default();
    sqlitedb.extend(ta_file_content[SQLITEDB_OFFSET..SQLITEDB_OFFSET + sqlitedb_len].iter());
    let sqlitedb: Vec<u8> = sqlitedb.iter().map(|c| *c as u8).collect::<Vec<_>>();

    write("sqlite.db", sqlitedb).expect("Could not dump SQLite DB..");

    let sqlitedb_file = File::open("sqlite.db")?;

    if sqlitedb_file.metadata().unwrap().len() as usize != sqlitedb_len {
        panic!(
            "SQLite DB file size mismatch! Got {}, expected {}",
            sqlitedb_file.metadata().unwrap().len(),
            sqlitedb_len
        )
    }

    println!("Saved results to sqlite.db!");

    Ok(())
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    const TA_EXPECTED_SIZE_BYTES: usize = 2097152; /* TODO: SMxxxx devices seem to use a new format. */

    println!("Opening file: {:?}", opts.file);

    let mut ta_file = File::open(opts.file)?;

    match ta_file.metadata().unwrap().len() as usize {
        TA_EXPECTED_SIZE_BYTES => println! {"TA size in tact, proceeding.."},
        _ => panic!(
            "TA size mismatch, got: {} expected: {}. Is your dump corrupted?",
            ta_file.metadata().unwrap().len(),
            TA_EXPECTED_SIZE_BYTES
        ),
    }

    let mut content: Vec<u8> = Vec::new();
    ta_file.read_to_end(&mut content)?;

    // TA magic, seems to be common for all generations
    if content[0] != 0xC1 && content[1] != 0xE9 {
        println!("TA header mismatch!");
        return Ok(());
    }

    let content: Vec<char> = content.iter().map(|b| *b as char).collect::<Vec<_>>();

    match opts.func {
        Func::DumpBootlogs => dump_bootlogs(&content),
        Func::DumpSqlitedb => dump_sqlitedb(&content)?,
        Func::ShowBuildid => show_build(&content),
        Func::ShowSerial => show_serialno(&content),
    }

    Ok(())
}
