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
    // fn allocate(&self, size: usize) -> Self;

    fn get(&self, idx: usize) -> TokenID;

    // fn set(&self, idx: usize, value: TokenID);

    fn size(&self) -> usize;
}

pub trait DTW {
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> f64;

    fn can_provide_alignment(&self) -> bool {
        false
    }
}

pub struct StandardDTW<'a> {
    pub distance: &'a dyn Distance,
}

pub struct STRACDistance {
    // Default to 1
    pub gap_cost: f64,
    pub match_cost: f64,
    pub mismatch_cost: f64,
}

impl Default for STRACDistance {
    fn default() -> Self {
        STRACDistance {
            gap_cost: 1.0,
            match_cost: 0.0,
            mismatch_cost: 3.0,
        }
    }
}

impl Distance for STRACDistance {
    #[inline]
    fn distance(&self, a: TokenID, b: TokenID) -> f64 {
        if a == b {
            self.match_cost
        } else {
            self.mismatch_cost
        }
    }

    #[inline]
    fn gap_cost(&self) -> f64 {
        self.gap_cost
    }
}

impl STRACDistance {
    pub fn new(gap_cost: f64, mismatch_cost: f64, match_cost: f64) -> Self {
        STRACDistance {
            gap_cost,
            match_cost,
            mismatch_cost,
        }
    }
}

impl<'a> StandardDTW<'a> {
    pub fn new(distance: &'a dyn Distance) -> StandardDTW {
        StandardDTW { distance }
    }
}

impl DTW for StandardDTW<'_> {
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> f64 {
        // Do slices
        // We do it with the max MEM possible
        let mut dtw = vec![vec![0.0; chain2.size() + 1]; chain1.size() + 1];
        let mut dtw = dtw.as_mut_slice();

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

// Implement one based on memory file mapping

impl Accesor for Vec<TokenID> {
    #[inline]
    fn get(&self, idx: usize) -> TokenID {
        self[idx]
    }

    #[inline]
    fn size(&self) -> usize {
        self.len()
    }
}

pub struct UnsafeDTW<'a> {
    distance: &'a dyn Distance,
}

impl<'a> UnsafeDTW<'a> {
    pub fn new(distance: &'a dyn Distance) -> UnsafeDTW {
        UnsafeDTW { distance }
    }
}

impl DTW for UnsafeDTW<'_> {
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> f64 {
        let mut dtw = vec![vec![0.0; chain2.size() + 1]; chain1.size() + 1];

        unsafe {
            for i in 0..=chain1.size() {
                for j in 0..=chain2.size() {
                    // Unsafe should disable the bounds check
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
}

pub struct FixedDTW<'a> {
    distance: &'a dyn Distance,
}

impl<'a> FixedDTW<'a> {
    pub fn new(distance: &'a dyn Distance) -> FixedDTW {
        FixedDTW { distance }
    }
}

impl DTW for FixedDTW<'_> {
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> f64 {
        // Swap the chains if the first one is smaller
        let (chain1, chain2) = if chain1.size() > chain2.size() {
            (chain2, chain1)
        } else {
            (chain1, chain2)
        };

        let mut prev_row = vec![0.0; chain1.size() + 1];
        let prev_row = prev_row.as_mut_slice();

        for i in 0..prev_row.len() {
            unsafe {
                prev_row[i] = self.distance.gap_cost() * i as f64;
            }
        }

        for i in 1..=chain2.size() {
            let mut curr_row = vec![0.0; chain1.size() + 1];
            // TODO Check if the following actually helps
            let mut curr_row = curr_row.as_mut_slice();

            curr_row[0] = self.distance.gap_cost() * i as f64;

            for j in 1..curr_row.len() {
                let a = chain1.get(j - 1);
                let b = chain2.get(i - 1);

                let cost = self.distance.distance(a, b);
                let min = unsafe { prev_row[j] }
                    .min(curr_row[j - 1])
                    .min(prev_row[j - 1]);

                unsafe { curr_row[j] = cost + min };
            }

            // Copy the memory
            unsafe { prev_row.copy_from_slice(&curr_row) };
        }

        unsafe { prev_row[chain1.size()] }
    }
}

trait AccesorAllocator<T>
where
    T: Accesor,
{
    fn allocate(&self, size: usize) -> T;

    fn set(&self, idx: usize, val: TokenID, accesor: &mut T);
}

fn reduce_by_half<'a, T>(allocator: &'a dyn AccesorAllocator<T>, accessor: &mut T) -> T
where
    T: Accesor,
{
    let mut r = allocator.allocate(accessor.size() / 2);
    for i in 0..r.size() {
        allocator.set(i, accessor.get(i * 2), &mut r);
    }
    r
}

pub struct InMemoryVectorAllocator;

impl AccesorAllocator<Vec<TokenID>> for InMemoryVectorAllocator {
    fn allocate(&self, size: usize) -> Vec<TokenID> {
        vec![0; size]
    }

    fn set(&self, idx: usize, val: TokenID, accesor: &mut Vec<TokenID>) {
        accesor[idx] = val;
    }
}

pub struct FastDTW<'a, T> {
    distance: &'a dyn Distance,
    window_size: usize,
    default_dtw: &'a dyn DTW,
    min_size: usize,
    accesor_allocator: &'a dyn AccesorAllocator<T>,
}

impl<'a, T> FastDTW<'a, T> {
    pub fn new(
        distance: &'a dyn Distance,
        window_size: usize,
        min_size: usize,
        default_dtw: &'a dyn DTW,
        accesor_allocator: &'a dyn AccesorAllocator<T>,
    ) -> FastDTW<'a, T> {
        FastDTW {
            distance,
            window_size,
            min_size,
            default_dtw,
            accesor_allocator,
        }
    }
}

impl<T> DTW for FastDTW<'_, T> {
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> f64 {
        todo!();

        /*if chain1.size() <= self.min_size || chain2.size() <= self.min_size {
            return self.default_dtw.calculate(chain1, chain2);
        }


        let chain1_half = reduce_by_half(self.accesor_allocator, &mut chain1);
        let chain2_half = reduce_by_half(self.accesor_allocator, &mut chain2);

        let path = self.calculate(chain1_half, chain2_half);

        // expand
        0.0*/
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
        let distance = STRACDistance::default();
        let dtw = StandardDTW::new(&distance);
        let chain1 = Box::new(vec![1, 2, 3]);
        let chain2 = Box::new(vec![1, 2, 3]);
        let result = dtw.calculate(chain1, chain2);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test1() {
        assert_eq!(2 + 2, 4);
        let distance = STRACDistance::default();
        let dtw = StandardDTW::new(&distance);
        let chain1 = Box::new(vec![1, 2, 3]);
        let chain2 = Box::new(vec![1, 2, 4]);
        let result = dtw.calculate(chain1, chain2);
        assert_eq!(result, 3.0);
    }
}
