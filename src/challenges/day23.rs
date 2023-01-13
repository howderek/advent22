use clap;

#[derive(clap::Args, Debug)]
pub struct Args {
    file: Option<String>,
}

pub fn entrypoint(args: &Args) {
    println!("{:?}", args.file);
}
