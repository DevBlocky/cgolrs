use std::{
    cmp::Ordering,
    ops::{Add, Neg, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos2 {
    pub x: i32,
    pub y: i32,
}
impl Pos2 {
    #[inline]
    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
    #[inline]
    pub fn one() -> Self {
        Self { x: 1, y: 1 }
    }
}
impl Default for Pos2 {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}
impl PartialOrd for Pos2 {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Pos2 {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // compare y coordinate first, then x coordinate
        // i.e. if y coordinate is equal, then compare x coordinate
        Ord::cmp(&self.y, &other.y).then(Ord::cmp(&self.x, &other.x))
    }
}
impl Neg for Pos2 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}
impl Add for Pos2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Sub for Pos2 {
    type Output = Pos2;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
