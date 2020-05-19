use sha2::{Sha256,Digest};
use std::path::{Path,PathBuf};
use std::collections::HashMap;
use std::fs::{OpenOptions,File};
use std::io::prelude::*;
use std::{env, fs};
use serde::{Serialize, Deserialize};
use nohash_hasher::BuildNoHashHasher;

const STAGE:&str = "./.rgit/stage/";
const OBJ:&str = "./.rgit/stage/obj.bin";
const STAGING_TREE:&str = "./.rgit/stage/tree.bin";

///represents files
#[derive(Debug)]
struct Blob{
    data:Vec<u8>,
}

impl Blob{
    fn new(data:Vec<u8>) -> Self{
        Self{
            data,
        }
    }
}

/// represents directories or files
/// maps file names to data
/// mapes dir names to the tree structure
#[derive(Debug)]
struct Tree{
    path:String,
    files:HashMap<String,Blob>,
    dir:HashMap<String,Box<Tree>>,
}

impl Tree{

    /// constructs the empty structure 
    fn new(name:String)-> Self{
        Self{
            path:name,
            files:HashMap::new(),
            dir:HashMap::new(),
        }
    }

    /// builds the tree
    fn build(&mut self) -> Result<(),std::io::Error>{
        for entry in fs::read_dir(&self.path)?{
            let entry = entry?;
            /// if file, map it by opening file and read_to_end
            /// if dir, construct a new tree, build it, add it to map
            let path = entry.path();
            let mut name = format!("{}/",&self.path);
            
            name.push_str(entry
                .file_name()
                .to_str()
                .unwrap());

            println!("{}",name);
            if fs::metadata(&path).unwrap().is_dir(){
                let mut sub_tree = Tree::new(name.clone());

                sub_tree.build();
                self.dir.insert(name, Box::new(sub_tree));
            }else{
                let mut file = OpenOptions::new()
                .read(true)
                .open(name.clone()).unwrap();
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).unwrap();
                self.files.insert(name,Blob::new(buffer));
            }
        }
        Ok(())
    }

}

struct Commit{
    parents:Vec<Box<Commit>>,
    snapshot:Tree,
    //author:String,
    message:String
}

pub fn set_tree(name:String){
    let mut tree = Tree::new(name);
    tree.build();

    println!("Dir listing:");
    println!("{:?}",tree.dir);

    println!("File listing:");
    println!("{:?}",tree.files);
}

pub fn insert(key:&String,data:Vec<u8>){

    let mut hasher = Sha256::new();
    hasher.input(key.as_bytes());

    let result = hasher.result();
    
    let mut k = String::new();
    for digit in &result[..7]{
        k.push_str(&digit.to_string());
    }
    let k = &k[..7];

    let mut obj = retrieve_obj();
    obj.insert(k.parse().unwrap(),data);

    let file = File::create(OBJ).unwrap();

    if let Err(e) = bincode::serialize_into(file,&obj){
        panic!("ERROR: {}",e);
    }

}

fn retrieve_obj() -> HashMap<u32,Vec<u8>,BuildNoHashHasher<u32>>{
       
    if Path::new(OBJ).exists(){
        let file = File::open(OBJ).unwrap();
        let x:HashMap<u32,Vec<u8>,BuildNoHashHasher<u32>> = 
            bincode::deserialize_from(&file).unwrap();
        
        assert_eq!(x.contains_key(&1662371),true);
        return x
    }

    HashMap::with_hasher(BuildNoHashHasher::default())
}
