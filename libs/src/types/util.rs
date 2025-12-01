pub type Id = usize;

// ----- CSV types -------------------------------------------------------

pub trait CsvHeader {
   fn header(&self) -> String;
}

// ----- PARTITION type -------------------------------------------------------

pub type Partition<T> = (Vec<T>, Vec<T>);

// ----- Measurable types -----------------------------------------------------

pub trait Measurable {
   fn sz(&self) -> f32;
   fn aug(&self) -> f32;
}

pub fn size<T: Measurable>(v: &Vec<T>) -> f32 {
   v.iter().map(Measurable::sz).sum()
}

pub fn weight<T: Measurable>(v: &Vec<T>) -> f32 {
   let (au, s) =
      v.iter()
       .fold((0.0, 0.0), |(a,b), x| (a + x.aug(), b + x.sz()));
   au / s
}
