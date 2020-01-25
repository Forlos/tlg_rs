extern crate rayon;
extern crate structopt;

use rayon::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

use tlg_rs::formats::tlg6::Tlg6;
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Files to process
    #[structopt(required = true, name = "FILES", parse(from_os_str))]
    files: Vec<PathBuf>,
    // Set verbose output
    #[structopt(short, long)]
    verbose: bool,
}

fn main() {
    let mut opt = Opt::from_args();
    opt.files.par_iter_mut().for_each(|file| {
        let tlg = Tlg6::from_file(&file.to_str().expect("Invalid path"))
            .expect("Could not parse from file");
        if let Ok(image) = tlg.to_rgba_image() {
            file.set_extension("png");
            image.save(file).expect("Could not write to file");
        }
    });
}
