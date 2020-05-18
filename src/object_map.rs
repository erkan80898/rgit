use sha2::{Sha256,Digest};
use std::path::Path;
use std::collections::HashMap;
use std::fs::File;
use nohash_hasher::BuildNoHashHasher;

const STAGE:&str = "./.rgit/stage/";
const OBJ:&str = "./.rgit/stage/obj";

pub fn insert(key:&String,data:Vec<u8>){

    let mut hasher = Sha256::new();
    hasher.input(key.as_bytes());

    let result = hasher.result();
    
    let mut k = String::new();
    for digit in &result[..7]{
        k.push_str(&digit.to_string());
    }
    let mut obj = retrieve_obj();

    println!("KEY: {}",k);
    obj.insert(k.parse().unwrap(),data);

    bincode::serialize_into(File::create(OBJ).unwrap(),&obj);
}

fn retrieve_obj() -> HashMap<u32,Vec<u8>,BuildNoHashHasher<u32>>{
        
    if Path::new(OBJ).exists(){
        let file = File::open(OBJ).unwrap();
        bincode::deserialize_from(file).unwrap()
    }

    HashMap::with_hasher(BuildNoHashHasher::default())
}
