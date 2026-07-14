use serde::{ Deserializer, Deserialize, Serializer };
use crate::types::util::Id;

pub fn deserialize_semicolon_list<'de, D>(deserializer: D)
      -> Result<Vec<Id>, D::Error> where D: Deserializer<'de> {
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.trim().is_empty() { return Ok(Vec::new()); }
    s.split(|c| c == ';' || c == ',')
     .map(|val| val.trim().parse::<Id>().map_err(serde::de::Error::custom))
     .collect()
}  
   
pub fn serialize_semicolon_list<S>(data: &Vec<Id>, serializer: S)
      -> Result<S::Ok, S::Error> where S: Serializer {

   // 1. Convert each usize to a String
   let parts: Vec<String> = data.iter().map(|x| x.to_string()).collect();

   // 2. Join the elements using a semicolon
   let joined = parts.join(";");
        
   // 3. Serialize as a single string primitive
   serializer.serialize_str(&joined)
}
