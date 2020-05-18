use std::collections::HashMap;
pub mod cli;
pub mod object_map;

/**
 * Use servo and bincode for seralization of all the states
 *
 */

struct Blob{
    data:Vec<u8>,
}

struct Tree{
    files:HashMap<String,Blob>,
    dir:HashMap<String,Option<Box<Tree>>>,
}


struct Commit{
//    parents:Vec<Box>
    root:Tree,
    //time_stamp:String,
    //author:String,
    //message:String
}


fn main() {
    cli::cli_parser();

}

