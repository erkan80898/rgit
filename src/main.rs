use std::collections::HashMap;
use bytes::Bytes;


struct Blob{
    data:Bytes,
}

struct Tree{
    files:HashMap<String,Blob>,
    dir:HashMap<String,Option<Box<Tree>>>,
}


struct Root{

}


fn main() {
    println!("z");
}
