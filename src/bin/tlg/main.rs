extern crate indicatif;
extern crate rayon;
extern crate structopt;

use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;

use tlg_rs::formats::constants::{TLG0_MAGIC, TLG5_MAGIC, TLG6_MAGIC, TLG_MAGIC_SIZE};
use tlg_rs::formats::{tlg0::Tlg0, tlg6::Tlg6};
#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Files to process
    #[structopt(required = true, name = "FILES", parse(from_os_str))]
    files: Vec<PathBuf>,
    /// Set verbose output
    #[structopt(short, long)]
    verbose: bool,
}

fn main() -> Result<(), failure::Error> {
    let opt = Opt::from_args();
    let progress_bar = init_progressbar(&opt);
    opt.files
        .par_iter()
        .progress_with(progress_bar)
        .filter(|file| file.is_file())
        .for_each(|file| {
            let mut contents = Vec::with_capacity(1 << 20);

            File::open(&file)
                .unwrap()
                .read_to_end(&mut contents)
                .expect("Could not read file contents");

            if contents.len() < TLG_MAGIC_SIZE {
                println!("File too small: {:?}", file);
                return;
            }

            let magic: [u8; TLG_MAGIC_SIZE] = contents[0..TLG_MAGIC_SIZE].try_into().unwrap();
            match magic {
                TLG0_MAGIC => {
                    let tlg = Tlg0::from_bytes(&contents).expect("Could not parse from file");
                    if let Ok(image) = tlg.to_rgba_image() {
                        let mut output_file = file.clone();
                        output_file.set_extension("png");
                        if opt.verbose {
                            println!("Converting: {:?}", &file);
                        }
                        image.save(output_file).expect("Could not write to file");
                    }
                }
                TLG5_MAGIC => println!("Parsing tlg5 is not yet implemented: {:?}", file),
                TLG6_MAGIC => {
                    let tlg = Tlg6::from_bytes(&contents).expect("Could not parse from file");
                    if let Ok(image) = tlg.to_rgba_image() {
                        let mut output_file = file.clone();
                        output_file.set_extension("png");
                        if opt.verbose {
                            println!("Converting: {:?}", &file);
                        }
                        image.save(output_file).expect("Could not write to file");
                    }
                }
                _ => println!("Not a valid tlg file: {:?}", file),
            }
        });

    Ok(())
}

fn init_progressbar(opt: &Opt) -> ProgressBar {
    let progress_bar = ProgressBar::new(opt.files.len() as u64).with_style(
        ProgressStyle::default_bar()
            .template(" {spinner} {msg} {wide_bar:} {pos:>6}/{len:6} ETA:[{eta}]"),
    );
    progress_bar.set_message("Converting...");
    progress_bar
}
