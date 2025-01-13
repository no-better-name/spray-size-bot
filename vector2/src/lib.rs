#[derive(Debug)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

pub trait VectorFrom<T> {
    fn vector_from(other: T) -> Self;
}

impl<T, U: From<T>> VectorFrom<Vector2<T>> for Vector2<U> {
    fn vector_from(other: Vector2<T>) -> Self {
        Vector2 { x: other.x.into(), y: other.y.into() }
    }
}

pub trait VectorTryFrom<T>: Sized {
    type Error;
    fn vector_try_from(other: T) -> Result<Self, Self::Error>;
}

impl<T, U: TryFrom<T>> VectorTryFrom<Vector2<T>> for Vector2<U> {
    type Error = <U as TryFrom<T>>::Error;
    fn vector_try_from(other: Vector2<T>) -> Result<Self, Self::Error> {
        let x_result: Result<U, Self::Error> = other.x.try_into();
        let y_result: Result<U, Self::Error> = other.y.try_into();

        match x_result {
            Ok(x) => match y_result {
                Ok(y) => return Ok(Vector2 { x, y }),
                Err(y) => return Err(y),
            },
            Err(x) => return Err(x),
        }
    }
}

impl<T> Vector2<T> {
    pub fn new(new_x: T, new_y: T) -> Vector2<T> {
        Vector2 {x: new_x, y: new_y}
    }

    pub fn fold<U: Fn(T, T) -> T>(self, func: U) -> T {
        func(self.x, self.y)
    }

    pub fn map<U, F: Fn(T) -> U>(self, func: F) -> Vector2<U> {
        Vector2 { x: func(self.x), y: func(self.y) }
    }
    
    pub fn apply_two<U, S, F: Fn(T, U) -> S>(self, other: Vector2<U>, func: F) -> Vector2<S> {
        Vector2 { x: func(self.x, other.x), y: func(self.y, other.y) }
    }
    
    pub fn reciprocate_in_place(&mut self) {
        std::mem::swap(&mut self.x, &mut self.y);
    }
}

impl<T: Clone> Vector2<T> {
    pub fn apply_n<U, F: Fn(&[T]) -> U>(vecs: &[Vector2<T>], func: F) -> Vector2<U> {
        let mut xs: Vec<T> = vec!();
        let mut ys: Vec<T> = vec!();
        for vec in vecs {
           let (x, y) = Into::<(T, T)>::into(vec);
           xs.push(x);
           ys.push(y);
        }

        Vector2 { x: func(&xs), y: func(&ys) }
    }

    pub fn reciprocal(&self) -> Vector2<T> {
        let (new_x, new_y) = self.into();
        Vector2 { x: new_y, y: new_x }
    }
}

impl<T: Clone> Clone for Vector2<T> {
    fn clone(&self) -> Self {
        Vector2 { x: self.x.clone(), y: self.y.clone() }
    }
}

impl<T: Copy> Copy for Vector2<T> { }

impl<T: std::hash::Hash> std::hash::Hash for Vector2<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl<T> From<[T; 2]> for Vector2<T> {
    fn from(value: [T; 2]) -> Self {
        let [new_x, new_y, ..] = value;
        Vector2 { x: new_x, y: new_y }
    }
}

impl<T> From<Vector2<T>> for [T; 2] {
    fn from(value: Vector2<T>) -> Self {
        [value.x, value.y]
    }
}

impl<T> From<Vector2<T>> for (T, T) {
    fn from(value: Vector2<T>) -> Self {
        (value.x, value.y)
    }
}

impl<T: Clone> From<&Vector2<T>> for [T; 2] {
    fn from(value: &Vector2<T>) -> Self {
        [value.x.clone(), value.y.clone()]
    }
}

impl<T: Clone> From<&Vector2<T>> for (T, T) {
    fn from(value: &Vector2<T>) -> Self {
        (value.x.clone(), value.y.clone())
    }
}

impl<T: Clone> From<&[T; 2]> for Vector2<T> {
    fn from(value: &[T; 2]) -> Self {
        Vector2 { x: value[0].clone(), y: value[1].clone() }
    }
}

impl<T: Clone> From<T> for Vector2<T> {
    fn from(value: T) -> Self {
        Vector2 { x: value.clone(), y: value.clone() }
    }
}

impl<T: PartialEq> PartialEq for Vector2<T> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl<T: Eq> Eq for Vector2<T> { }

impl<T: Default> Default for Vector2<T> {
    fn default() -> Self {
        Vector2 { x: T::default(), y: T::default() }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Vector2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<T: std::ops::Add<U, Output = S>, U, S> std::ops::Add<Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn add(self, rhs: Vector2<U>) -> Self::Output {
        Self::Output { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl<T: std::ops::Add<U, Output = S>, U: Clone, S> std::ops::Add<&Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn add(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x + rhs.x.clone(), y: self.y + rhs.y.clone() }
    }
}

impl<T: std::ops::Add<U, Output = S> + Clone, U: Clone, S> std::ops::Add<&Vector2<U>> for &Vector2<T> {
    type Output = Vector2<S>;
    fn add(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x.clone() + rhs.x.clone(), y: self.y.clone() + rhs.y.clone() }
    }
}

impl<T: std::ops::AddAssign<U>, U> std::ops::AddAssign<Vector2<U>> for Vector2<T> {
    fn add_assign(&mut self, rhs: Vector2<U>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: std::ops::AddAssign<U>, U: Clone> std::ops::AddAssign<&Vector2<U>> for Vector2<T> {
    fn add_assign(&mut self, rhs: &Vector2<U>) {
        self.x += rhs.x.clone();
        self.y += rhs.y.clone();
    }
}

impl<T: std::ops::BitAnd<U, Output = S>, U, S> std::ops::BitAnd<Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn bitand(self, rhs: Vector2<U>) -> Self::Output {
        Self::Output { x: self.x & rhs.x, y: self.y & rhs.y }
    }
}

impl<T: std::ops::BitAnd<U, Output = S>, U: Clone, S> std::ops::BitAnd<&Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn bitand(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x & rhs.x.clone(), y: self.y & rhs.y.clone() }
    }
}

impl<T: std::ops::BitAnd<U, Output = S> + Clone, U: Clone, S> std::ops::BitAnd<&Vector2<U>> for &Vector2<T> {
    type Output = Vector2<S>;
    fn bitand(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x.clone() & rhs.x.clone(), y: self.y.clone() & rhs.y.clone() }
    }
}

impl<T: std::ops::BitAndAssign<U>, U> std::ops::BitAndAssign<Vector2<U>> for Vector2<T> {
    fn bitand_assign(&mut self, rhs: Vector2<U>) {
        self.x &= rhs.x;
        self.y &= rhs.y;
    }
}

impl<T: std::ops::BitAndAssign<U>, U: Clone> std::ops::BitAndAssign<&Vector2<U>> for Vector2<T> {
    fn bitand_assign(&mut self, rhs: &Vector2<U>) {
        self.x &= rhs.x.clone();
        self.y &= rhs.y.clone();
    }
}

impl<T: std::ops::BitOr<U, Output = S>, U, S> std::ops::BitOr<Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn bitor(self, rhs: Vector2<U>) -> Self::Output {
        Self::Output { x: self.x | rhs.x, y: self.y | rhs.y }
    }
}

impl<T: std::ops::BitOr<U, Output = S>, U: Clone, S> std::ops::BitOr<&Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn bitor(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x | rhs.x.clone(), y: self.y | rhs.y.clone() }
    }
}

impl<T: std::ops::BitOr<U, Output = S> + Clone, U: Clone, S> std::ops::BitOr<&Vector2<U>> for &Vector2<T> {
    type Output = Vector2<S>;
    fn bitor(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x.clone() | rhs.x.clone(), y: self.y.clone() | rhs.y.clone() }
    }
}

impl<T: std::ops::BitOrAssign<U>, U> std::ops::BitOrAssign<Vector2<U>> for Vector2<T> {
    fn bitor_assign(&mut self, rhs: Vector2<U>) {
        self.x |= rhs.x;
        self.y |= rhs.y;
    }
}

impl<T: std::ops::BitOrAssign<U>, U: Clone> std::ops::BitOrAssign<&Vector2<U>> for Vector2<T> {
    fn bitor_assign(&mut self, rhs: &Vector2<U>) {
        self.x |= rhs.x.clone();
        self.y |= rhs.y.clone();
    }
}

impl<T: std::ops::BitXor<U, Output = S>, U, S> std::ops::BitXor<Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn bitxor(self, rhs: Vector2<U>) -> Self::Output {
        Self::Output { x: self.x ^ rhs.x, y: self.y ^ rhs.y }
    }
}

impl<T: std::ops::BitXor<U, Output = S>, U: Clone, S> std::ops::BitXor<&Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn bitxor(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x ^ rhs.x.clone(), y: self.y ^ rhs.y.clone() }
    }
}

impl<T: std::ops::BitXor<U, Output = S> + Clone, U: Clone, S> std::ops::BitXor<&Vector2<U>> for &Vector2<T> {
    type Output = Vector2<S>;
    fn bitxor(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x.clone() ^ rhs.x.clone(), y: self.y.clone() ^ rhs.y.clone() }
    }
}

impl<T: std::ops::BitXorAssign<U>, U> std::ops::BitXorAssign<Vector2<U>> for Vector2<T> {
    fn bitxor_assign(&mut self, rhs: Vector2<U>) {
        self.x ^= rhs.x;
        self.y ^= rhs.y;
    }
}

impl<T: std::ops::BitXorAssign<U>, U: Clone> std::ops::BitXorAssign<&Vector2<U>> for Vector2<T> {
    fn bitxor_assign(&mut self, rhs: &Vector2<U>) {
        self.x ^= rhs.x.clone();
        self.y ^= rhs.y.clone();
    }
}

impl<T: std::ops::Div<U, Output = S>, U, S> std::ops::Div<Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn div(self, rhs: Vector2<U>) -> Self::Output {
        Self::Output { x: self.x / rhs.x, y: self.y / rhs.y }
    }
}

impl<T: std::ops::Div<U, Output = S>, U: Clone, S> std::ops::Div<&Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn div(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x / rhs.x.clone(), y: self.y / rhs.y.clone() }
    }
}

impl<T: std::ops::Div<U, Output = S> + Clone, U: Clone, S> std::ops::Div<&Vector2<U>> for &Vector2<T> {
    type Output = Vector2<S>;
    fn div(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x.clone() / rhs.x.clone(), y: self.y.clone() / rhs.y.clone() }
    }
}

impl<T: std::ops::DivAssign<U>, U> std::ops::DivAssign<Vector2<U>> for Vector2<T> {
    fn div_assign(&mut self, rhs: Vector2<U>) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl<T: std::ops::DivAssign<U>, U: Clone> std::ops::DivAssign<&Vector2<U>> for Vector2<T> {
    fn div_assign(&mut self, rhs: &Vector2<U>) {
        self.x /= rhs.x.clone();
        self.y /= rhs.y.clone();
    }
}

impl<T: std::ops::Mul<U, Output = S>, U, S> std::ops::Mul<Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn mul(self, rhs: Vector2<U>) -> Self::Output {
        Self::Output { x: self.x * rhs.x, y: self.y * rhs.y }
    }
}

impl<T: std::ops::Mul<U, Output = S>, U: Clone, S> std::ops::Mul<&Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn mul(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x * rhs.x.clone(), y: self.y * rhs.y.clone() }
    }
}

impl<T: std::ops::Mul<U, Output = S> + Clone, U: Clone, S> std::ops::Mul<&Vector2<U>> for &Vector2<T> {
    type Output = Vector2<S>;
    fn mul(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x.clone() * rhs.x.clone(), y: self.y.clone() * rhs.y.clone() }
    }
}

impl<T: std::ops::MulAssign<U>, U> std::ops::MulAssign<Vector2<U>> for Vector2<T> {
    fn mul_assign(&mut self, rhs: Vector2<U>) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl<T: std::ops::MulAssign<U>, U: Clone> std::ops::MulAssign<&Vector2<U>> for Vector2<T> {
    fn mul_assign(&mut self, rhs: &Vector2<U>) {
        self.x *= rhs.x.clone();
        self.y *= rhs.y.clone();
    }
}

impl<T: std::ops::Neg<Output = U>, U> std::ops::Neg for Vector2<T> {
    type Output = Vector2<U>;
    fn neg(self) -> Self::Output {
        Self::Output { x: -self.x, y: -self.y }
    }
}

impl<T: std::ops::Not<Output = U>, U> std::ops::Not for Vector2<T> {
    type Output = Vector2<U>;
    fn not(self) -> Self::Output {
        Self::Output { x: !self.x, y: !self.y }
    }
}

impl<T: std::ops::Rem<U, Output = S>, U, S> std::ops::Rem<Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn rem(self, rhs: Vector2<U>) -> Self::Output {
        Self::Output { x: self.x % rhs.x, y: self.y % rhs.y }
    }
}

impl<T: std::ops::Rem<U, Output = S>, U: Clone, S> std::ops::Rem<&Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn rem(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x % rhs.x.clone(), y: self.y % rhs.y.clone() }
    }
}

impl<T: std::ops::Rem<U, Output = S> + Clone, U: Clone, S> std::ops::Rem<&Vector2<U>> for &Vector2<T> {
    type Output = Vector2<S>;
    fn rem(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x.clone() % rhs.x.clone(), y: self.y.clone() % rhs.y.clone() }
    }
}

impl<T: std::ops::RemAssign<U>, U> std::ops::RemAssign<Vector2<U>> for Vector2<T> {
    fn rem_assign(&mut self, rhs: Vector2<U>) {
        self.x %= rhs.x;
        self.y %= rhs.y;
    }
}

impl<T: std::ops::RemAssign<U>, U: Clone> std::ops::RemAssign<&Vector2<U>> for Vector2<T> {
    fn rem_assign(&mut self, rhs: &Vector2<U>) {
        self.x %= rhs.x.clone();
        self.y %= rhs.y.clone();
    }
}

impl<T: std::ops::Shl<U, Output = S>, U, S> std::ops::Shl<Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn shl(self, rhs: Vector2<U>) -> Self::Output {
        Self::Output { x: self.x << rhs.x, y: self.y << rhs.y }
    }
}

impl<T: std::ops::Shl<U, Output = S>, U: Clone, S> std::ops::Shl<&Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn shl(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x << rhs.x.clone(), y: self.y << rhs.y.clone() }
    }
}

impl<T: std::ops::Shl<U, Output = S> + Clone, U: Clone, S> std::ops::Shl<&Vector2<U>> for &Vector2<T> {
    type Output = Vector2<S>;
    fn shl(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x.clone() << rhs.x.clone(), y: self.y.clone() << rhs.y.clone() }
    }
}

impl<T: std::ops::ShlAssign<U>, U> std::ops::ShlAssign<Vector2<U>> for Vector2<T> {
    fn shl_assign(&mut self, rhs: Vector2<U>) {
        self.x <<= rhs.x;
        self.y <<= rhs.y;
    }
}

impl<T: std::ops::ShlAssign<U>, U: Clone> std::ops::ShlAssign<&Vector2<U>> for Vector2<T> {
    fn shl_assign(&mut self, rhs: &Vector2<U>) {
        self.x <<= rhs.x.clone();
        self.y <<= rhs.y.clone();
    }
}

impl<T: std::ops::Shr<U, Output = S>, U, S> std::ops::Shr<Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn shr(self, rhs: Vector2<U>) -> Self::Output {
        Self::Output { x: self.x >> rhs.x, y: self.y >> rhs.y }
    }
}

impl<T: std::ops::Shr<U, Output = S>, U: Clone, S> std::ops::Shr<&Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn shr(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x >> rhs.x.clone(), y: self.y >> rhs.y.clone() }
    }
}

impl<T: std::ops::Shr<U, Output = S> + Clone, U: Clone, S> std::ops::Shr<&Vector2<U>> for &Vector2<T> {
    type Output = Vector2<S>;
    fn shr(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x.clone() >> rhs.x.clone(), y: self.y.clone() >> rhs.y.clone() }
    }
}

impl<T: std::ops::ShrAssign<U>, U> std::ops::ShrAssign<Vector2<U>> for Vector2<T> {
    fn shr_assign(&mut self, rhs: Vector2<U>) {
        self.x >>= rhs.x;
        self.y >>= rhs.y;
    }
}

impl<T: std::ops::ShrAssign<U>, U: Clone> std::ops::ShrAssign<&Vector2<U>> for Vector2<T> {
    fn shr_assign(&mut self, rhs: &Vector2<U>) {
        self.x >>= rhs.x.clone();
        self.y >>= rhs.y.clone();
    }
}

impl<T: std::ops::Sub<U, Output = S>, U, S> std::ops::Sub<Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn sub(self, rhs: Vector2<U>) -> Self::Output {
        Self::Output { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl<T: std::ops::Sub<U, Output = S>, U: Clone, S> std::ops::Sub<&Vector2<U>> for Vector2<T> {
    type Output = Vector2<S>;
    fn sub(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x - rhs.x.clone(), y: self.y - rhs.y.clone() }
    }
}

impl<T: std::ops::Sub<U, Output = S> + Clone, U: Clone, S> std::ops::Sub<&Vector2<U>> for &Vector2<T> {
    type Output = Vector2<S>;
    fn sub(self, rhs: &Vector2<U>) -> Self::Output {
        Self::Output { x: self.x.clone() - rhs.x.clone(), y: self.y.clone() - rhs.y.clone() }
    }
}

impl<T: std::ops::SubAssign<U>, U> std::ops::SubAssign<Vector2<U>> for Vector2<T> {
    fn sub_assign(&mut self, rhs: Vector2<U>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T: std::ops::SubAssign<U>, U: Clone> std::ops::SubAssign<&Vector2<U>> for Vector2<T> {
    fn sub_assign(&mut self, rhs: &Vector2<U>) {
        self.x -= rhs.x.clone();
        self.y -= rhs.y.clone();
    }
}

#[cfg(test)]
mod tests {
    use crate::{Vector2, VectorFrom, VectorTryFrom};

    #[test]
    fn creation_test() {
        let vec = Vector2 {x: 1, y: 2};
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);

        let vec = Vector2::new(1, 2);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);

        let vec = Vector2::<i32>::from(&[1, 2]);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);

        let vec = Vector2::<i32>::from([1, 2]);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);

        let vec = Vector2::from(1);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 1);
    }

    #[test]
    fn conversion_test() {
        let vec = Vector2::<i32> {x: 1, y: 2};
        let vec_2: Vector2<i64> = VectorFrom::vector_from(vec);
        let vec_3: Result<Vector2<f64>, _> = VectorTryFrom::vector_try_from(vec);

        assert_eq!(Into::<i64>::into(vec.x), vec_2.x);
        assert_eq!(Into::<i64>::into(vec.y), vec_2.y);
        assert_eq!(Into::<f64>::into(vec.x), vec_3.unwrap().x);
        assert_eq!(Into::<f64>::into(vec.y), vec_3.unwrap().y);
    }

    #[test]
    fn arith_test() {
        let lhs = Vector2 {x: 3, y: 4};
        let rhs = Vector2 {x: 7, y: 8};

        assert_eq!(lhs + rhs, Vector2 {x: 3 + 7, y: 4 + 8});
        assert_eq!(lhs - rhs, Vector2 {x: 3 - 7, y: 4 - 8});
        assert_eq!(lhs * rhs, Vector2 {x: 3 * 7, y: 4 * 8});
        assert_eq!(lhs / rhs, Vector2 {x: 3 / 7, y: 4 / 8});
        assert_eq!(lhs % rhs, Vector2 {x: 3 % 7, y: 4 % 8});

        assert_eq!(-lhs, Vector2 {x: -3, y: -4});
    }

    #[test]
    fn bit_test() {
        let lhs = Vector2 {x: 3, y: 4};
        let rhs = Vector2 {x: 7, y: 8};

        assert_eq!(lhs & rhs, Vector2 {x: 3 & 7, y: 4 & 8});
        assert_eq!(lhs | rhs, Vector2 {x: 3 | 7, y: 4 | 8});
        assert_eq!(lhs ^ rhs, Vector2 {x: 3 ^ 7, y: 4 ^ 8});

        let lhs = dbg!(Vector2 {x: true, y: false});
        let rhs = dbg!(Vector2 {x: true, y: true});

        assert_eq!(lhs & rhs, dbg!(Vector2 {x: true & true, y: false & true}));
        assert_eq!(lhs | rhs, dbg!(Vector2 {x: true | true, y: false | true}));
        assert_eq!(lhs ^ rhs, dbg!(Vector2 {x: true ^ true, y: false ^ true}));
    }

    #[test]
    fn fold_test() {
        let vec = Vector2 { x: 3, y: 4 };

        assert_eq!(vec.fold(|lhs, rhs| lhs.min(rhs)), 3.min(4));
        assert_eq!(vec.fold(|lhs, rhs| lhs.max(rhs)), 3.max(4));
        assert_eq!(vec.fold(|lhs, rhs| lhs + rhs), 3 + 4);
        assert_eq!(vec.fold(|lhs, rhs| lhs - rhs), 3 - 4);
        assert_eq!(vec.fold(|lhs, rhs| lhs * rhs), 3 * 4);
        assert_eq!(vec.fold(|lhs, rhs| lhs / rhs), 3 / 4);
    }

    #[test]
    fn reciprocal_test() {
        let mut vec = Vector2 { x: 3, y: 4 };
        vec.reciprocate_in_place();
        
        assert_eq!(vec, Vector2 { x: 4, y: 3 });

        let vec = Vector2 { x: 3, y: 4 };
        assert_eq!(vec.reciprocal(), Vector2 { x: 4, y: 3 })
    }
}
