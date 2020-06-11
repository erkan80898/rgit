use clap::Clap;
use std::fs::{OpenOptions};
use std::io::prelude::*;
use chrono::{Timelike};
///TODO: set up vim for commiting if no message is given
use subprocess;
use crate::object_map;

const OBJ:&str = ".rgit/obj.data";

#[derive(Clap)]
#[clap(version = "1.0", author = "Erkan U. <erkan808987@gmail.com>")]
struct Opts{
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand{
    Commit(Commit),
    Add(Tree),
    Init,
    Pull,
}

#[derive(Clap)]
struct Commit{
    message:String,
}

#[derive(Clap)]
struct Tree{
    dir_name:String,    
}
#[derive(Clap)]
struct Init;

#[derive(Clap)]
struct Pull;

pub fn cli_parser(){

    let time = chrono::offset::Utc::now(); 
    let date = time.date();
    let timestamp = format!("Timestamp: {} {:02}:{:02}:{:02}",
                        date,time.hour(),time.minute(),time.second());
    let opts = Opts::parse();
    match opts.subcmd{
        SubCommand::Init =>{object_map::init()}
        SubCommand::Commit(mut x) => {
            x.message = format!("{}\n\tMessage: {} \n",timestamp,x.message);
            object_map::commit(x.message);
        }
        SubCommand::Add(x) =>{
            object_map::set_tree(x.dir_name);
        }
        SubCommand::Pull =>{object_map::pull()}
    }
}
