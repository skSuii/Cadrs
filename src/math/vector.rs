use std::ops::{Add, Sub, Mul, Div, Neg};

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    #[inline]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    #[inline]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> f64 {
        self.y
    }

    #[inline]
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    #[inline]
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self::new(self.x / len, self.y / len)
        } else {
            Self::zero()
        }
    }
    
    #[inline]
    pub fn angle_to(&self, other: &Self) -> f64 {
        let dot = self.x * other.x + self.y * other.y;
        let det = self.x * other.y - self.y * other.x;
        det.atan2(dot)
    }
    
    #[inline]
    pub fn magnitude(&self) -> f64 {
        self.length()
    }
    
    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
    
    #[inline]
    pub fn normalized(&self) -> Self {
        self.normalize()
    }
    
    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        *self + *other
    }
    
    #[inline]
    pub fn sub(&self, other: &Self) -> Self {
        *self - *other
    }
    
    #[inline]
    pub fn scale(&self, scalar: f64) -> Self {
        *self * scalar
    }
    
    #[inline]
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }

    #[inline]
    pub fn angle(&self) -> f64 {
        self.y.atan2(self.x)
    }
}

impl Add for Vector2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vector2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f64> for Vector2 {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

impl Div<f64> for Vector2 {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self::new(self.x / scalar, self.y / scalar)
    }
}

impl Neg for Vector2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl std::fmt::Display for Vector2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vector2({:.6}, {:.6})", self.x, self.y)
    }
}
