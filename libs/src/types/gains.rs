use book::num::percentage::Percentage;

pub trait Gains {
   fn roi(&self) -> Percentage;
   fn apr(&self) -> Percentage;
}

