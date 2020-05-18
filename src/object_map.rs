use sha2::{Sha256,Digest};
use std::path::Path;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use serde::{Serialize, Deserialize};
use nohash_hasher::BuildNoHashHasher;

const STAGE:&str = "./.rgit/stage/";
const OBJ:&str = "./.rgit/stage/obj.bin";

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
