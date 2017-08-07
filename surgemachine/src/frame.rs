use std::ops::{Add,Mul};
use std::iter::Sum;

#[derive(Debug)]
pub struct Frame<T = f32> {
    pub l: T,
    pub r: T
}

impl<T: Default> Default for Frame<T> {
    fn default () -> Self {
        Self {
            l: Default::default(),
            r: Default::default()
        }
    }
}

impl<T: Add<Output=T>> Add for Frame<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            l: self.l + other.l,
            r: self.r + other.r,
        }
    }
}

impl<T: Mul<Output=T>> Mul for Frame<T> {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Frame {
            l: self.l * other.l,
            r: self.r * other.r,
        }
    }
}

impl<T: Mul<Output=T> + Copy> Mul<T> for Frame<T> {
    type Output = Self;
    fn mul(self, other: T) -> Self {
        Frame {
            l: self.l * other,
            r: self.r * other,
        }
    }
}

impl<T: Add<Output=T> + Copy> Add<T> for Frame<T> {
    type Output = Self;
    fn add(self, other: T) -> Self {
        Frame {
            l: self.l + other,
            r: self.r + other,
        }
    }
}

impl<T: Default + Add<Output=T>> Sum for Frame<T> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Default::default(), | a, b | a + b)
    }
}
