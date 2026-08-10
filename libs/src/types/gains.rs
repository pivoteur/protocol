use book::{ currency::usd::USD, num::percentage::Percentage };

pub trait Gains {
   fn roi(&self) -> Percentage;
   fn apr(&self) -> Percentage;
   fn gain(&self) -> f32;
   fn gain_usd(&self) -> USD;
}
