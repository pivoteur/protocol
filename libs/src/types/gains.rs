use book::num::percentage::Percentage;

trait Gains {
   fn roi(&self) -> Percentage;
   fn apr(&self) -> Percentage;
}

