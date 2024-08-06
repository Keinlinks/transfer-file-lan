use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MetadataSeriable {
   pub len: u64,
   pub is_dir: bool,
   pub is_file: bool,
   pub name: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Device {
   pub id: i32,
    pub name: String,
    pub ip: String,
    pub port: u16,
}