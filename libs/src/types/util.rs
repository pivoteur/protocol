pub type Id = usize;

// ----- CSV types -------------------------------------------------------

pub trait CsvHeader {
   fn header(&self) -> String;
}

