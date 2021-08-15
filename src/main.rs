// Licensing: See the LICENSE file

use clap::{AppSettings, ArgEnum, Clap};
use std::fs::{read, write};
use std::io::Result;
use std::path::PathBuf;

#[derive(ArgEnum, Copy, Clone, Debug)]
enum Platform {
    Loire,
    Tone,
    Yoshino,
    Nile,
    Tama,
    Ganges,
    Kumano,
    Seine,
    Edo,
    Lena,
    Sagami,
}

impl Platform {
    fn bootlog_offset(self) -> [usize; 10] {
        match self {
            Self::Tama => [
                0x2A22E, 0x2DA22, 0x31CEE, 0x3542A, 0x38C46, 0x3C7A2, 0x65412, 0x68C2E, 0x6C78A,
                0x70A2E,
            ],
            p => todo!("No bootlog offsets for `{:?}` yet!", p),
        }
    }
}

#[derive(Clap, Debug)]
enum Func {
    /// Dump boot logs (TA stores up to ten of these)
    DumpBootlogs {
        #[clap(arg_enum)]
        platform: Platform,
    },
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

fn read_ta(ta_file_content: &[u8], offset: usize, length: usize) -> &[u8] {
    &ta_file_content[offset..offset + length]
}

fn dump_bootlogs(platform: Platform, ta_file_content: &[u8]) {
    const BOOTLOG_SIZE: usize = 14309;

    for (i, &offset) in platform.bootlog_offset().iter().enumerate() {
        println!("Dumping bootlog {} at {:x}..", i, offset);
        let bootlog = read_ta(ta_file_content, offset, BOOTLOG_SIZE);
        let filename = format!("bootlogs/bootlog{}.txt", i);
        println!("writing to {}", filename);
        write(filename, bootlog).expect("Could not dump bootlog..");
    }
}

fn show_build(ta_file_content: &[u8]) {
    const VERSION_OFFSET: usize = 0x7B4;
    // 32 is an educated guess, it's actually 29 on Tama-Akari
    const VERSION_SIZE: usize = 32;

    let build_id = read_ta(ta_file_content, VERSION_OFFSET, VERSION_SIZE);
    println!("Image version: {}", std::str::from_utf8(build_id).unwrap());
}

fn show_serialno(ta_file_content: &[u8]) {
    const SERIAL_OFFSET: usize = 0x600B4;
    const SERIAL_SIZE: usize = 10;

    let serial_no = read_ta(ta_file_content, SERIAL_OFFSET, SERIAL_SIZE);
    println!("Serial no.: {}", std::str::from_utf8(serial_no).unwrap());
}

fn dump_sqlitedb(ta_file_content: &[u8]) -> Result<()> {
    const SQLITEDB_OFFSET: usize = 0x20044;
    const SQLITEDB_HEADER_SIZEVAL_OFF: usize = 16;

    let sqlitedb_len = read_ta(
        ta_file_content,
        SQLITEDB_OFFSET + SQLITEDB_HEADER_SIZEVAL_OFF,
        2,
    );

    let sqlitedb_len = sqlitedb_len[0] as usize + ((sqlitedb_len[1] as usize) << 8);
    println!(
        "SQLite DB size: 2^{} ({} B)",
        sqlitedb_len,
        2usize.pow(sqlitedb_len as u32)
    );

    let sqlitedb_len = 2usize.pow(sqlitedb_len as u32);
    let sqlitedb = &ta_file_content[SQLITEDB_OFFSET..SQLITEDB_OFFSET + sqlitedb_len];

    write("sqlite.db", sqlitedb).expect("Could not dump SQLite DB..");

    println!("Saved results to sqlite.db!");

    Ok(())
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    const TA_EXPECTED_SIZE_BYTES: usize = 2097152; /* TODO: SMxxxx devices seem to use a new format. */

    println!("Opening file: {:?}", opts.file);

    let content = read(opts.file)?;
    assert_eq!(
        content.len(),
        TA_EXPECTED_SIZE_BYTES,
        "TA size mismatch, got: {} expected: {}. Is your dump corrupted?",
        content.len(),
        TA_EXPECTED_SIZE_BYTES
    );

    // TA magic, seems to be common for all generations
    if content[0..2] != [0xC1, 0xE9] {
        eprintln!("TA header mismatch!");
        return Ok(());
    }

    match opts.func {
        Func::DumpBootlogs { platform } => dump_bootlogs(platform, &content),
        Func::DumpSqlitedb => dump_sqlitedb(&content)?,
        Func::ShowBuildid => show_build(&content),
        Func::ShowSerial => show_serialno(&content),
    }

    Ok(())
}
