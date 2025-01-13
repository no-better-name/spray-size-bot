use std::{collections::HashSet, num::NonZero};

use vector2::Vector2;

pub fn gcd(mut u: u32, mut v: u32) -> u32 {
    if u == 0 {
        return v;
    } else if v == 0 {
        return u;
    }

    let i = u.trailing_zeros(); u >>= i;
    let j = v.trailing_zeros(); v >>= j;
    let k = i.min(j);

    loop {
        if u > v {
            std::mem::swap(&mut u, &mut v);
        }

        v -= u;
        if v == 0 {
            return u << k;
        }
        v >>= v.trailing_zeros();
    }
}

pub const GOLDSRC_MULTIPLE: u32 = 16;
pub const SVEN_COOP_RELATIVE_SIZE: u32 = 56;
pub const HALF_LIFE_RELATIVE_SIZE: u32 = 42;

#[derive(Debug, Clone)]
pub struct GameData {
    multiple: NonZero<u32>,
    ratios: HashSet<Vector2<NonZero<u32>>>,
    max_relative_size: NonZero<u32>,
}

#[derive(Debug, Clone)]
pub struct RatioFittingResult {
    ratio: Vector2<NonZero<u32>>,
    as_is: Vector2<NonZero<u32>>,
    resized: Vec<Vector2<NonZero<u32>>>,
}

impl RatioFittingResult {
    fn new(ratio: Vector2<NonZero<u32>>, as_is: Vector2<NonZero<u32>>, resized: Vec<Vector2<NonZero<u32>>>) -> Self {
        Self {ratio, as_is, resized}
    }
    pub fn ratio(&self) -> Vector2<NonZero<u32>> {
        self.ratio
    }
    pub fn as_is(&self) -> Vector2<NonZero<u32>> {
        self.as_is
    }
    pub fn resized(&self) -> &[Vector2<NonZero<u32>>] {
        &self.resized
    }
}

impl GameData {
    pub fn new(multiple: u32, max_relative_size: u32) -> Self {
        let multiple = NonZero::new(multiple).expect("Multiple cannot be zero");
        let max_relative_size = NonZero::new(max_relative_size).expect("Size in multiples cannot be zero");

        let result = Self { multiple, ratios: HashSet::new(), max_relative_size };
        result.all_ratios()
    }

    pub fn all_ratios(mut self) -> Self {
        let mut set = HashSet::new();

        for x in 1 ..= self.max_relative_size.get() {
            for y in 1 ..= self.max_relative_size.get() {
                let size_sqrt = f64::from(self.max_relative_size.get()).sqrt().ceil();
                let (fx, fy) = (f64::from(x), f64::from(y));
                if fx > size_sqrt || fy > size_sqrt {
                    continue;
                }

                let vec = Vector2 { x, y } / Into::<Vector2<u32>>::into(gcd(x, y));
                let vec = vec.map(|x| NonZero::new(x).expect("Ratio cannot have zero as either numerator or denominator"));
                set.insert(vec);
            }
        }

        self.ratios = set;
        self
    }

    pub fn biggest_ratios(mut self) -> Self {
        let mut set = HashSet::new();

        for x in 1 ..= self.max_relative_size.get() {
            for y in 1 ..= self.max_relative_size.get() {
                let size_sqrt = f64::from(self.max_relative_size.get()).sqrt().ceil();
                let (fx, fy) = (f64::from(x), f64::from(y));
                if x * y != self.max_relative_size.get() || fx > size_sqrt || fy > size_sqrt {
                    continue;
                }

                let vec = Vector2 { x, y } / Into::<Vector2<u32>>::into(gcd(x, y));
                let vec = vec.map(|x| NonZero::new(x).expect("Ratio cannot have zero as either numerator or denominator"));
                set.insert(vec);
            }
        }

        self.ratios = set;
        self
    }

    pub fn multiple(mut self, new_multiple: u32) -> Self {
        self.multiple = NonZero::new(new_multiple).expect("Multiple cannot be zero");
        self
    }

    pub fn max_relative_size(mut self, new_size: u32) -> Self {
        self.max_relative_size = NonZero::new(new_size).expect("Size in multiples cannot be equal to zero");
        self.all_ratios()
    }

    pub fn fit_size(&self, original_size: Vector2<NonZero<u32>>) -> RatioFittingResult {
        let mut final_result: Option<RatioFittingResult> = None;
        let original_size = original_size.map(|x| x.get());

        if self.ratios.is_empty() {
            unreachable!();
        }

        for ratio_nonzero_wrapped in &self.ratios {
            let ratio = ratio_nonzero_wrapped.map(|x| x.get());

            let step = ratio * Vector2::from(self.multiple.get());
            let mut new_size = step;
            let mut last_size = Vector2::from(0);

            while last_size == Vector2::from(0) || original_size.symmetric_difference(new_size) <= original_size.symmetric_difference(last_size) {
                last_size = new_size;
                new_size += step;
            }

            let best_for_ratio = {
                if original_size.symmetric_difference(last_size) < original_size.symmetric_difference(new_size) {
                    last_size
                } else {
                    new_size
                }
            };
            let resized = {
                let mut temp = Vec::new();
                for n in 1.. {
                    let next_size = step * Vector2::from(n);
                    if next_size.fold(|x, y| x * y) > self.multiple.get().pow(2) * self.max_relative_size.get() { break; }
                    temp.push(next_size.map(|x| NonZero::new(x).unwrap()));
                }
                temp
            };
            let cycle_result = RatioFittingResult::new(
                *ratio_nonzero_wrapped, best_for_ratio.map(|x| NonZero::new(x).unwrap()), resized
            );

            match final_result {
                Some(result) => {
                    if original_size.symmetric_difference(best_for_ratio) < original_size.symmetric_difference(result.as_is.map(|x| x.get())) {
                        final_result = Some(cycle_result);
                    } else {
                        final_result = Some(result);
                    }
                },
                None => {
                    final_result = Some(cycle_result);
                }
            }
        }

        final_result.unwrap()
    }
}

pub trait SetDifference {
    type Output;
    fn set_difference(self, other: Self) -> (Self::Output, Self::Output);
}

impl SetDifference for Vector2<u32> {
    type Output = u32;
    fn set_difference(self, other: Self) -> (Self::Output, Self::Output) {
        let mul = |x, y| x * y;
        let min = |x: u32, y| x.min(y);
        (
            self.fold(mul) - self.apply_two(other, min).fold(mul),
            other.fold(mul) - self.apply_two(other, min).fold(mul),
        )
    }
}

pub trait SymmetricDifference: SetDifference {
    fn symmetric_difference(self, other: Self) -> Self::Output;
}

impl SymmetricDifference for Vector2<u32> {
    fn symmetric_difference(self, other: Self) -> Self::Output {
        let (a, b) = self.set_difference(other);
        a + b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcd_test() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(3, 0), 3);
        assert_eq!(gcd(0, 4), 4);
        assert_eq!(gcd(13, 7), 1);
    }

    #[test]
    fn ratio_creation_test() {
        let _ = dbg!(GameData::new(GOLDSRC_MULTIPLE, HALF_LIFE_RELATIVE_SIZE));
        let _ = dbg!(GameData::new(GOLDSRC_MULTIPLE, SVEN_COOP_RELATIVE_SIZE));
        let _ = dbg!(GameData::new(GOLDSRC_MULTIPLE, HALF_LIFE_RELATIVE_SIZE).biggest_ratios());
        let _ = dbg!(GameData::new(GOLDSRC_MULTIPLE, SVEN_COOP_RELATIVE_SIZE).biggest_ratios());
    }

    #[test]
    fn symmetric_difference_test() {
        let one = Vector2 { x: 4, y: 3 };
        let two = Vector2 { x: 5, y: 5 };
        assert_eq!(one.symmetric_difference(two), 13);

        let one = Vector2 { x: 4, y: 3 };
        let two = Vector2 { x: 2, y: 2 };
        assert_eq!(one.symmetric_difference(two), 8);

        let one = Vector2 { x: 16, y: 10 };
        let two = Vector2 { x: 12, y: 18 };
        assert_eq!(one.symmetric_difference(two), (16 - 12) * 10 + (18 - 10) * 12);
    }

    #[test]
    fn fitting_test() {
        let size = Vector2::new(1128, 1840);
        let mut game_state = GameData::new(GOLDSRC_MULTIPLE, HALF_LIFE_RELATIVE_SIZE);
        dbg!(game_state.fit_size(size.map(|x| NonZero::new(x).unwrap())));
        game_state = game_state.biggest_ratios();
        dbg!(game_state.fit_size(size.map(|x| NonZero::new(x).unwrap())));

        game_state = game_state.max_relative_size(SVEN_COOP_RELATIVE_SIZE);
        dbg!(game_state.fit_size(size.map(|x| NonZero::new(x).unwrap())));
        game_state = game_state.biggest_ratios();
        dbg!(game_state.fit_size(size.map(|x| NonZero::new(x).unwrap())));
    }
}
