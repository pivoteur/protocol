use std::cmp::Ordering;

// ----- Measurable types -------------------------------------------------

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

pub fn sort_by_weight<M: Measurable>(a: &M, b: &M) -> Ordering {
   b.aug().total_cmp(&a.aug())
}

pub fn sort_by_size<M: Measurable>(a: &M, b: &M) -> Ordering {
   b.sz().total_cmp(&a.sz())
}

pub fn sort_descending<M: Measurable>(a: &M, b: &M) -> Ordering {
   sort_by_weight(a, b)
}

