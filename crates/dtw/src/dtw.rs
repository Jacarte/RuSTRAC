//! Core traits for DTW implementations
//!
//!

/// Type for the trace token
pub type TokenID = usize;

pub trait Distance {
    fn distance(&self, a: TokenID, b: TokenID) -> f64;
    fn gap_cost(&self) -> f64;
}

/// A trait for accessing the data to be compared
/// For example, for linear arrays this is just a wrapper for accesing the array elements.
pub trait Accesor {
    fn get(&self, idx: usize) -> TokenID;
    fn size(&self) -> usize;
}

pub trait DTW {
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> f64;
}

struct StandardDTW<'a> {
    pub distance: &'a dyn Distance,
}
struct STRACDistance;

impl Distance for STRACDistance {
    fn distance(&self, a: TokenID, b: TokenID) -> f64 {
        if a == b {
            0.0
        } else {
            5.0
        }
    }

    fn gap_cost(&self) -> f64 {
        1.0
    }
}

impl<'a> StandardDTW<'a> {
    fn new(distance: &'a dyn Distance) -> StandardDTW {
        StandardDTW { distance }
    }
}

impl DTW for StandardDTW<'_> {
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> f64 {
        let mut dtw = vec![vec![0.0; chain2.size() + 1]; chain1.size() + 1];

        for i in 0..=chain1.size() {
            for j in 0..=chain2.size() {
                match (i, j) {
                    (0, 0) => dtw[0][0] = 0.0,
                    // First column
                    (0, _) => dtw[0][j] = self.distance.gap_cost() + dtw[0][j - 1],
                    // First row
                    (i, 0) => dtw[i][0] = self.distance.gap_cost() + dtw[i - 1][0],
                    _ => {
                        let a = chain1.get(i - 1);
                        let b = chain2.get(j - 1);

                        let cost = self.distance.distance(a, b);
                        let min = dtw[i - 1][j].min(dtw[i][j - 1]).min(dtw[i - 1][j - 1]);

                        dtw[i][j] = cost + min;
                    }
                }
            }
        }

        dtw[chain1.size()][chain2.size()]
    }
}

impl Accesor for Vec<TokenID> {
    fn get(&self, idx: usize) -> TokenID {
        self[idx]
    }

    fn size(&self) -> usize {
        self.len()
    }
}

// Traditional DTW O(nn) space and time
//

// Windowed DTW O(nn) space and O(nn) time

// Memoized DTW O(n) space and O(nn) time
//

// FastDTW O(n) space and O(n log n) time
//

// Wavefront DTW O(n) space and O(nn/SIMD size) time
//
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
        let dtw = StandardDTW::new(&STRACDistance);
        let chain1 = Box::new(vec![1, 2, 3]);
        let chain2 = Box::new(vec![1, 2, 3]);
        let result = dtw.calculate(chain1, chain2);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test1() {
        assert_eq!(2 + 2, 4);
        let dtw = StandardDTW::new(&STRACDistance);
        let chain1 = Box::new(vec![1, 2, 3]);
        let chain2 = Box::new(vec![1, 2, 4]);
        let result = dtw.calculate(chain1, chain2);
        assert_eq!(result, 5.0);
    }
}
