extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;

use tlg_rs::formats::tlg6::Tlg6;
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Files to process
    #[structopt(required = true, name = "FILES", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();
    for mut file in opt.files {
        let tlg = Tlg6::from_file(&file).unwrap();
        println!("Converting: {:?}", file);
        if let Ok(image) = tlg.to_rgba_image() {
            file.set_extension("png");
            image.save(file).unwrap();
        }
    }
}
