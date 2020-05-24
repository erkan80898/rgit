use sha2::{Sha256,Digest};
use std::path::{Path,PathBuf};
use std::collections::HashMap;
use std::fs::{OpenOptions,File};
use std::io::prelude::*;
use std::fs;
use serde::{Serialize, Deserialize};
use nohash_hasher::BuildNoHashHasher;

const OBJ:&str = "./.rgit/stage/obj.bin";
const NODE:&str = "./.rgit/stage/node.bin";
const STAGING_TREE:&str = "./.rgit/stage/tree.bin";
const LOG:&str = ".rgit/log.txt";

///represents files
#[derive(Debug,Serialize, Deserialize)]
struct Blob{
    data:Vec<u8>,
}

/// represents directories or files
/// maps file names to data
/// mapes dir names to the tree structure
#[derive(Debug,Serialize, Deserialize)]
struct Tree{
    path:String,
    files:HashMap<String,Blob>,
    dir:HashMap<String,Box<Tree>>,
}

#[derive(Debug,Serialize, Deserialize)]
struct Commit{
    parent:Option<Box<Commit>>,
    snapshot:Tree,
    //author:String,
    message:String
}

impl Blob{
    fn new(data:Vec<u8>) -> Self{
        Self{
            data,
        }
    }
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
            let path = entry.path();
            let mut name = format!("{}/",&self.path);
            
            name.push_str(entry
                .file_name()
                .to_str()
                .unwrap());

            println!("{}",name);
            if fs::metadata(&path).unwrap().is_dir(){
                let mut sub_tree = Tree::new(name.clone());

                sub_tree.build()?;
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

    fn read(&self,buffer:&mut Vec<u8>){
        
        fn helper(dir:&Tree,buffer:&mut Vec<u8>){
            for (_,val) in dir.files.iter(){
                buffer.append(&mut (*val).data.clone());
            }

            for (_,tree) in dir.dir.iter(){
                helper(&tree,buffer);
            }
        }

        for (_,val) in self.files.iter(){
            buffer.append(&mut (*val).data.clone());
        }

        for (_,mut tree) in self.dir.iter(){
            helper(&mut tree,buffer);
        }
        
    }
}


impl Commit{

    fn new(message:String,snapshot:Tree) -> Self{
        Self{
            parent:None,
            snapshot,
            message,
        }
    }
}

pub fn set_tree(name:String){
    let mut tree = Tree::new(name);
    tree.build();

    /// seralize tree into staging area
    let file = File::create(STAGING_TREE).unwrap();
    if let Err(e) = bincode::serialize_into(file,&tree){
        panic!("ERROR: {}",e);
    }

    let testTree = retrieve_tree();

    println!("{:?}",testTree);
}

fn retrieve_tree() -> Tree{
       
    let file = File::open(STAGING_TREE);
    if file.is_err(){
        panic!("Nothing in staging area! Add the directory to commit");
    }
    let file = file.unwrap();
    let x:Tree = bincode::deserialize_from(&file).unwrap();
        
    return x
}
 
/// commit objects in staging area
pub fn commit(message:String){
    /// TODO FIX SAME KEY ISSUE
    let stage_tree:Tree = retrieve_tree();
    let mut commit:Commit = Commit::new(message,stage_tree);
    
    if Path::new(NODE).exists(){
        match OpenOptions::new()
        .read(true)
        .write(true)
        .open(NODE){
            Ok(file) => {
                let old_commit:Commit = bincode::deserialize_from(&file).unwrap();
                commit.parent = Some(Box::new(old_commit));

                if let Err(e) = bincode::serialize_into(file,&commit){
                    panic!("ERROR: {}",e);
                }
            }
            Err(e) => panic!("ERROR OPENING: {}",e)
        }
    }else{
        match File::create(NODE){
            Ok(file) => {
                if let Err(e) = bincode::serialize_into(file,&commit){
                    panic!("ERROR: {}",e);
                }
            }
            Err(e) => panic!("ISSUE CREATING FILE: {}", e)
        }
        
    }

    insert(commit);
}

fn log(message:String){
    match OpenOptions::new()
                        .read(true)
                        .append(true)
                        .create(true)
                        .open(LOG){
                Ok(mut file) => {
                    if let Err(e) = file.write_all(message.as_bytes()){
                        panic!("Couldn't the message! Error: {}",e);
                    }
                }
                Err(e) => {
                    panic!("{}",e)
                }
    }
}

/// REFACTOR TO COMMITS INSTEAD
/// KEY WILL BE THE BASED OFF tree.
fn insert(commit:Commit){

    let mut hasher = Sha256::new();
    let mut buffer:Vec<u8> = Vec::new();
    commit.snapshot.read(&mut buffer);

    hasher.input(buffer);

    let result = hasher.result();
    
    let mut k = String::new();
    for digit in &result[..7]{
        k.push_str(&digit.to_string());
    }
    let k = &k[..7];

    log(format!("{}KEY: {}\n",commit.message,k));

    let mut obj = retrieve_obj();

    obj.insert(k.parse().unwrap(),commit);

    let file = File::create(OBJ).unwrap();

    if let Err(e) = bincode::serialize_into(file,&obj){
        panic!("ERROR: {}",e);
    }

}

fn retrieve_obj() -> HashMap<u32,Commit,BuildNoHashHasher<u32>>{
       
    if Path::new(OBJ).exists(){
        let file = File::open(OBJ).unwrap();
        let x:HashMap<u32,Commit,BuildNoHashHasher<u32>> = 
            bincode::deserialize_from(&file).unwrap();
        return x
    }

    HashMap::with_hasher(BuildNoHashHasher::default())
}
