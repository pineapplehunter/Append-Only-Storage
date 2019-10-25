use structopt::StructOpt;

use hash_storage::storage::Storage;
use std::error::Error;
use std::fs::{File, FileType};
use std::io;

use ignore::{Walk, WalkBuilder, WalkParallel, WalkState};
use std::sync::Arc;

#[derive(StructOpt, Debug)]
#[structopt(version = "1.0", author = "Shogo Takata")]
struct Opts {
    #[structopt(short = "o", long = "out_dir")]
    out_dir: String,
    #[structopt(short = "i", long = "input_dir")]
    input_dir: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::from_args();
    let storage = Arc::new(Storage::new(&opts.out_dir));

    WalkBuilder::new(&opts.input_dir)
        .hidden(true)
        .build_parallel()
        .run(|| {
            let storage = storage.clone();
            Box::new(move |result| {
                match result {
                    Ok(entry) => {
                        if entry.path().is_dir() {
                            return WalkState::Continue;
                        }

                        let mut writer = storage.new_file_writer();
                        if let Ok(mut input) = File::open(dbg!(entry.path())) {
                            io::copy(&mut input, &mut writer).expect("could not write");
                        } else {
                            dbg!("could not open file");
                        }
                    }
                    Err(_) => {}
                }
                WalkState::Continue
            })
        });
    Ok(())
}
