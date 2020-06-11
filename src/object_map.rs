use sha2::{Sha256,Digest};
use std::path::{Path,PathBuf};
use std::collections::HashMap;
use std::fs::{OpenOptions,File};
use std::fs;
use std::io::prelude::*;
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
/// maps dir names to the tree structure
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
 
/// Build the rgit directories 
/// TODO have a check if dir already exist, so not to wipe
pub fn init(){

    fs::create_dir("./.rgit");
    fs::create_dir("./.rgit/stage");
}

pub fn pull(){

    /// Retrieve tree
    /// First iter through tree, build dir if it doesnt exit
    /// add all blobs to their dirs
    

    fn helper(tree:&Tree){
        if !Path::new(&tree.path).exists(){
            fs::create_dir(&tree.path);
        }
        for files in tree.files.iter(){
            match OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(files.0){
                Ok(mut file) =>{
                    file.write_all(&files.1.data);
                }
                Err(err) => {
                    panic!("Error from pulling: {}",err);
                }
            }
        }
    }
    let tree = retrieve_tree();  

    helper(&tree);
    for (_,current_tree) in tree.dir.iter(){
        helper(current_tree);
    }

}

/// commit objects in staging area
pub fn commit(message:String){
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

    let mut obj = retrieve_obj();

    if obj.contains_key(&k.parse().unwrap()){
        println!("No changes to commit.");
    }else{
        log(format!("{}KEY: {}\n",commit.message,k));
        obj.insert(k.parse().unwrap(),commit);
        let file = File::create(OBJ).unwrap();

        if let Err(e) = bincode::serialize_into(file,&obj){
            panic!("ERROR: {}",e);
        }
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
