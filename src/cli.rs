use clap::Clap;
use std::fs::{OpenOptions};
use std::io::prelude::*;
use chrono::{Timelike};
///TODO: set up vim for commiting if no message is given
use subprocess;
use crate::object_map;

const LOG:&str = ".rgit/log.txt";
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
    Hash(Object),
    Add(Tree),
}

#[derive(Clap)]
struct Commit{
    message:String,
}

#[derive(Clap)]
struct Object{
    file:String,    
}

#[derive(Clap)]
struct Tree{
    dir_name:String,    
}

pub fn cli_parser(){

    let time = chrono::offset::Utc::now(); 
    let date = time.date();
    let timestamp = format!("Timestamp: {} {:02}:{:02}:{:02}",
                        date,time.hour(),time.minute(),time.second());
    let opts = Opts::parse();
    match opts.subcmd{
        SubCommand::Commit(mut x) => {
            x.message = format!("{}\n\tMessage: {} \n",timestamp,x.message);
            match OpenOptions::new()
                                    .read(true)
                                    .append(true)
                                    .create(true)
                                    .open(LOG){
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
        SubCommand::Hash(mut x) => {
            //Add file to the object in .rgit folder
            match OpenOptions::new()
                                    .read(true)
                                    .open(&x.file){
                Ok(mut file) =>{
                    let mut buffer = Vec::new();
                    if let Err(re) = file.read_to_end(&mut buffer){
                        panic!("Issue reading the file data {}",re)
                    }
                    object_map::insert(&x.file,buffer)
                }
                Err(e) => panic!("{}",e)
            }
        }

        SubCommand::Add(x) =>{
            object_map::set_tree(x.dir_name);
        }
    }
}
