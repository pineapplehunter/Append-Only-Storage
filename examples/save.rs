use structopt::StructOpt;

use hash_storage::storage::Storage;
use std::error::Error;
use std::fs::File;
use std::io;

#[derive(StructOpt, Debug)]
#[structopt(version = "1.0", author = "Shogo Takata")]
struct Opts {
    #[structopt(short = "o", long = "out_dir")]
    out_dir: String,
    #[structopt(short = "i", long = "input_file")]
    input_file: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::from_args();

    let storage = Storage::new(&opts.out_dir);
    let mut writer = storage.new_file_writer();

    let mut input = File::open(&opts.input_file)?;

    io::copy(&mut input, &mut writer)?;

    Ok(())
}
