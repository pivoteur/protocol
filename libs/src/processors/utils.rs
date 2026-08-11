use chrono::NaiveDate;
use serde::{ Deserializer, Deserialize, Serializer, de::Error };
use book::string_utils::s;
use crate::types::util::Id;

// ----- Deserializers ------------------------------------------------------

pub fn deserialize_semicolon_list<'de, D>(deserializer: D)
      -> Result<Vec<Id>, D::Error> where D: Deserializer<'de> {
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.trim().is_empty() { return Ok(Vec::new()); }
    s.split(|c| c == ';' || c == ',')
     .map(|val| val.trim().parse::<Id>().map_err(serde::de::Error::custom))
     .collect()
}  

pub fn deserialize_optional_date<'de, D>(deserializer: D)
      -> Result<Option<NaiveDate>, D::Error> where D: Deserializer<'de> {
   let s: String = Deserialize::deserialize(deserializer)?;
   if &s == "n/a" {
      Ok(None)
   } else {
      Ok(Some(s.parse().map_err(D::Error::custom)?))
   }
}

// ----- Serializers --------------------------------------------------------

pub fn serialize_semicolon_list<S>(data: &Vec<Id>, serializer: S)
      -> Result<S::Ok, S::Error> where S: Serializer {

   // 1. Convert each usize to a String
   let parts: Vec<String> = data.iter().map(|x| x.to_string()).collect();

   // 2. Join the elements using a semicolon
   let joined = parts.join(";");
        
   // 3. Serialize as a single string primitive
   serializer.serialize_str(&joined)
}

pub fn serialize_optional_date<S>(data: &Option<NaiveDate>, serializer: S)
      -> Result<S:: Ok, S::Error> where S: Serializer {
   let opened = match data {
      None => s("n/a"),
      Some(dt) => format!("{dt}")
   };
   serializer.serialize_str(&opened)
}

