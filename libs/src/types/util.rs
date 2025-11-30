pub type Id = usize;

// ----- CSV types -------------------------------------------------------

pub trait CsvHeader {
   fn header(&self) -> String;
}

// ----- PARTITION type -------------------------------------------------------

pub type Partition<T> = (Vec<T>, Vec<T>);
