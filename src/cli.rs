use clap::Clap;
use std::error::Error;
use std::fs::{File,OpenOptions};
use std::path::Path;
use std::io::prelude::*;

const LOG:&str = ".rgit/log.txt";

#[derive(Clap)]
#[clap(version = "1.0", author = "Erkan U. <erkan808987@gmail.com>")]
struct Opts{
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand{
    Commit(Commit),
}

#[derive(Clap)]
struct Commit{
    message:String,
}

pub fn cli_parser(){

    let opts = Opts::parse();

    match opts.subcmd{
        SubCommand::Commit(mut x) => {
            x.message = x.message + "\n";
            println!("Your commit: {}",x.message);
            match OpenOptions::new()
                                    .read(true)
                                    .append(true)
                                    .create(true)
                                    .open(LOG)
            {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(x.message.as_bytes()){
                        panic!("Couldn't the message! Error: {}",e);
                    }
                }
                Err(e) => {
                    panic!("{}",e)
                }
            }

        }
         _ => (),
    }
}
