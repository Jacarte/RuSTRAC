//! Core traits for DTW implementations
//!
//!

/// Type for the trace token
pub type TokenID = usize;
/// Operation type in the warp path
pub type OP = (usize, usize);

pub type DTWResult = (f64, Option<Vec<OP>>);

pub trait Distance {
    fn distance(&self, a: TokenID, b: TokenID) -> f64;
    fn gap_cost(&self) -> f64;
}

/// A trait for accessing the data to be compared
/// For example, for linear arrays this is just a wrapper for accesing the array elements.
pub trait Accesor {
    // fn allocate(&self, size: usize) -> Self;

    fn get(&self, idx: usize) -> TokenID;
    fn size(&self) -> usize;
}

pub trait DTW {
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> DTWResult;

    fn can_provide_alignment(&self) -> bool {
        false
    }

    fn get_warp_path(&self, map: &[Vec<f64>], window: Option<DynamicWindow>) -> Vec<OP> {
        // We always start in the end of the alignment
        let mut i = map.len() - 1;
        let mut j = map[0].len() - 1;
        let mut r = vec![];

        while i > 0 || j > 0 {
            let mut diagcost = 0.0;
            let mut leftcost = 0.0;
            let mut rightcost = 0.0;

            if i > 0 && j > 0 {
                match &window {
                    Some(window) => {
                        if window.is_in_range(i - 1, j - 1) {
                            diagcost = map[i - 1][j - 1];
                        } else {
                            diagcost = std::f64::INFINITY;
                        }
                    }
                    None => {
                        diagcost = map[i - 1][j - 1];
                    }
                }
            } else {
                diagcost = std::f64::INFINITY;
            }

            if i > 0 {
                match &window {
                    Some(window) => {
                        if window.is_in_range(i - 1, j) {
                            leftcost = map[i - 1][j];
                        } else {
                            leftcost = std::f64::INFINITY;
                        }
                    }
                    None => {
                        leftcost = map[i - 1][j];
                    }
                }
            } else {
                leftcost = std::f64::INFINITY;
            }

            if j > 0 {
                match &window {
                    Some(window) => {
                        if window.is_in_range(i, j - 1) {
                            rightcost = map[i][j - 1];
                        } else {
                            rightcost = std::f64::INFINITY;
                        }
                    }
                    None => {
                        rightcost = map[i][j - 1];
                    }
                }
            } else {
                rightcost = std::f64::INFINITY;
            }

            if diagcost <= leftcost && diagcost <= rightcost {
                // The diagonal is better
                i -= 1;
                j -= 1;
            } else if leftcost <= diagcost && leftcost <= rightcost {
                i -= 1;
            } else if rightcost <= diagcost && rightcost <= leftcost {
                j -= 1;
            } else if i <= j {
                j -= 1;
            } else {
                i -= 1;
            }

            // Push the operation
            r.push((i, j));
        }

        r
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
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> DTWResult {
        // Do slices
        // We do it with the max MEM possible
        let mut dtw = vec![vec![0.0; chain2.size() + 1]; chain1.size() + 1];
        let mut dtw = dtw.as_mut_slice();

        for i in 0..=chain1.size() {
            for j in 0..=chain2.size() {
                match (i, j) {
                    (0, 0) => dtw[0][0] = 0.0,
                    // First column
                    (0, _) => dtw[0][j] = self.distance.gap_cost() * j as f64,
                    // First row
                    (i, 0) => dtw[i][0] = self.distance.gap_cost() * i as f64,
                    _ => {
                        let a = chain1.get(i - 1);
                        let b = chain2.get(j - 1);

                        let diagcost = self.distance.distance(a, b) + dtw[i - 1][j - 1];
                        let leftcost = self.distance.gap_cost() + dtw[i - 1][j];
                        let rightcost = self.distance.gap_cost() + dtw[i][j - 1];

                        let min = diagcost.min(leftcost).min(rightcost);

                        dtw[i][j] = min;
                    }
                }
            }
        }

        let cost = dtw[chain1.size()][chain2.size()];
        let path = self.get_warp_path(&dtw, None);

        (cost, Some(path))
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
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> DTWResult {
        let mut dtw = vec![vec![0.0; chain2.size() + 1]; chain1.size() + 1];

        unsafe {
            for i in 0..=chain1.size() {
                for j in 0..=chain2.size() {
                    // Unsafe should disable the bounds check
                    match (i, j) {
                        (0, 0) => dtw[0][0] = 0.0,
                        // First column
                        (0, _) => dtw[0][j] = self.distance.gap_cost() * j as f64,
                        // First row
                        (i, 0) => dtw[i][0] = self.distance.gap_cost() * i as f64,
                        _ => {
                            let a = chain1.get(i - 1);
                            let b = chain2.get(j - 1);

                            let cost = self.distance.distance(a, b) + dtw[i - 1][j - 1];
                            let leftcost = self.distance.gap_cost() + dtw[i - 1][j];
                            let rightcost = self.distance.gap_cost() + dtw[i][j - 1];

                            let min = cost.min(leftcost).min(rightcost);

                            dtw[i][j] = cost;
                        }
                    }
                }
            }

            let cost = dtw[chain1.size()][chain2.size()];
            let path = self.get_warp_path(&dtw, None);

            (cost, Some(path))
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
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> DTWResult {
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

        // let mut progre = 0;

        for i in 1..=chain2.size() {
            let mut curr_row = vec![0.0; chain1.size() + 1];
            // TODO Check if the following actually helps
            let mut curr_row = curr_row.as_mut_slice();

            curr_row[0] = self.distance.gap_cost() * i as f64;

            for j in 1..curr_row.len() {
                let a = chain1.get(j - 1);
                let b = chain2.get(i - 1);

                let cost = self.distance.distance(a, b) + unsafe { prev_row[j - 1] };
                let leftcost = self.distance.gap_cost() + unsafe { prev_row[j] };
                let rightcost = self.distance.gap_cost() + curr_row[j - 1];

                let min = cost.min(leftcost).min(rightcost);

                unsafe { curr_row[j] = cost };
            }

            // Copy the memory
            unsafe { prev_row.copy_from_slice(&curr_row) };
        }

        unsafe { (prev_row[chain1.size()], None) }
    }
}

#[derive(Clone)]
pub struct DynamicWindow {
    min_values: Vec<i32>,
    max_values: Vec<i32>,
    width: usize,
}

impl DynamicWindow {
    #[inline]
    pub fn get_max(&self, row: usize) -> usize {
        self.max_values[row] as usize
    }

    #[inline]
    pub fn get_min(&self, row: usize) -> usize {
        self.min_values[row] as usize
    }

    #[inline]
    pub fn expand(&mut self, radius: i32) {
        for i in 0..self.min_values.len() {
            let val = self.min_values[i];
            self.min_values[i] = 0.max(val - radius);
            let maxval = self.max_values[i];
            self.max_values[i] = (maxval + radius).min(self.width as i32);
        }
    }

    #[inline]
    pub fn get_limits(&self, row: usize) -> (usize, usize) {
        (self.get_min(row), self.get_max(row))
    }

    #[inline]
    pub fn is_in_range(&self, row: usize, col: usize) -> bool {
        row < self.min_values.len()
            && self.min_values[row] <= col as i32
            && (col as i32) < self.max_values[row]
    }

    pub fn set(&mut self, row: usize, col: usize) {
        if row >= self.min_values.len() || col > self.width {
            return;
        }

        if self.min_values[row] == -1 && col >= 0 && col <= self.width {
            self.min_values[row] = col as i32;
            self.max_values[row] = col as i32;
        } else if self.min_values[row] > col as i32 && col >= 0 && col <= self.width
        // minimum range in the row is expanded
        {
            self.min_values[row] = col as i32;
        } else if self.max_values[row] < col as i32 && col >= 0 && col <= self.width
        // maximum range in the row is expanded
        {
            self.max_values[row] = col as i32;
        } // end if
    }

    #[inline]
    pub fn set_range(&mut self, min: i32, max: i32, row: usize) {
        self.min_values[row] = min;
        self.max_values[row] = max;
    }

    #[inline]
    pub fn init(&mut self, height: usize) -> &mut Self {
        for i in 0..height {
            if i < self.width {
                self.set_range(0, (self.width + i) as i32, i);
            } else if i >= self.width && i < height - self.width {
                self.set_range((i - self.width) as i32, (self.width + i) as i32, i);
            } else {
                self.set_range((i - self.width) as i32, height as i32, i);
            }
            //self.set_range(0, self.width as i32, i);
        }
        self
    }

    pub fn new(height: usize, width: usize, set: bool) -> Self {
        DynamicWindow {
            min_values: vec![if set { -1 } else { 0 }; height],
            max_values: vec![if set { -1 } else { width as i32 }; height],
            width,
        }
    }
}

pub struct WindowedDTW<'a> {
    window: usize,
    distance: &'a dyn Distance,
}

impl<'a> WindowedDTW<'a> {
    pub fn new(window: usize, distance: &'a dyn Distance) -> Self {
        WindowedDTW { window, distance }
    }
}

impl<'a> DTW for WindowedDTW<'a> {
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> DTWResult {
        // Do slices
        // We do it with the max MEM possible
        let mut dtw = vec![vec![std::f64::INFINITY; chain2.size() + 1]; chain1.size() + 1];
        let mut dtw = dtw.as_mut_slice();

        let mut dynamic_window = DynamicWindow {
            min_values: vec![0; chain1.size() + 1],
            max_values: vec![self.window as i32; chain1.size() + 1],
            width: self.window,
        };
        // Initialize the min part of the range
        let mut dynamic_window = dynamic_window.init(chain1.size() + 1);

        for i in 0..=chain1.size() {
            let (min, max) = dynamic_window.get_limits(i);
            for j in min..max.min(chain2.size() + 1) {
                // patch here ?
                match (i, j) {
                    (0, 0) => {
                        dtw[0][0] = if dynamic_window.is_in_range(0, 0) {
                            0.0
                        } else {
                            std::f64::INFINITY
                        }
                    }
                    // First column
                    (0, _) => {
                        dtw[0][j] = if dynamic_window.is_in_range(0, j) {
                            self.distance.gap_cost() * j as f64
                        } else {
                            std::f64::INFINITY
                        }
                    }
                    // First row
                    (i, 0) => {
                        dtw[i][0] = if dynamic_window.is_in_range(i, 0) {
                            self.distance.gap_cost() * i as f64
                        } else {
                            std::f64::INFINITY
                        }
                    }
                    _ => {
                        let a = chain1.get(i - 1);
                        let b = chain2.get(j - 1);

                        // If i - 1, j - 1 are outside the window then return INFINITE
                        let diagcost = if dynamic_window.is_in_range(i - 1, j - 1) {
                            self.distance.distance(a, b) + dtw[i - 1][j - 1]
                        } else {
                            std::f64::INFINITY
                        };

                        let leftcost = if dynamic_window.is_in_range(i - 1, j) {
                            self.distance.gap_cost() + dtw[i - 1][j]
                        } else {
                            std::f64::INFINITY
                        };

                        let rightcost = if dynamic_window.is_in_range(i, j - 1) {
                            self.distance.gap_cost() + dtw[i][j - 1]
                        } else {
                            std::f64::INFINITY
                        };

                        //let diagcost = self.distance.distance(a, b) + dtw[i - 1][j - 1];
                        // let leftcost = self.distance.gap_cost() + dtw[i - 1][j];
                        // let rightcost = self.distance.gap_cost() + dtw[i][j - 1];

                        let mini = diagcost.min(leftcost).min(rightcost);

                        dtw[i][j] = mini;
                    }
                }
            }
        }

        // Write the bidimensional matrix
        /*for i in 0..=chain1.size() {
            for j in 0..=chain2.size() {
                print!("{:3} ", dtw[i][j]);
            }
            println!();
        }*/

        let cost = dtw[chain1.size()][chain2.size()];
        let path = self.get_warp_path(&dtw, Some(dynamic_window.clone()));

        (cost, Some(path))
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
    // TODO create a random strategy as well
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
    radius: usize,
    default_dtw: &'a dyn DTW,
    min_size: usize,
    accesor_allocator: &'a dyn AccesorAllocator<T>,
}

impl<'a, T> FastDTW<'a, T> {
    pub fn new(
        distance: &'a dyn Distance,
        radius: usize,
        min_size: usize,
        default_dtw: &'a dyn DTW,
        accesor_allocator: &'a dyn AccesorAllocator<T>,
    ) -> FastDTW<'a, T> {
        FastDTW {
            distance,
            radius,
            min_size,
            default_dtw,
            accesor_allocator,
        }
    }
}

impl<T> DTW for FastDTW<'_, T> {
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> DTWResult {
        todo!();
        /*
        if chain1.size() <= self.min_size || chain2.size() <= self.min_size {
            return self.default_dtw.calculate(chain1, chain2);
        }


        let chain1_half = reduce_by_half(self.accesor_allocator, &mut chain1);
        let chain2_half = reduce_by_half(self.accesor_allocator, &mut chain2);

        // TODO move this to a queue. Yet, we do not have that many stack calls, log(n) at most
        let (cost, path) = self.calculate(chain1_half, chain2_half);


        // Expand the path
        //


        // expand
        todo!()*/
    }
}

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
        let (result, ops) = dtw.calculate(chain1, chain2);

        println!("{:?}", ops);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test1() {
        assert_eq!(2 + 2, 4);
        let distance = STRACDistance::default();
        let dtw = StandardDTW::new(&distance);
        let chain1 = Box::new(vec![1, 2, 3, 5]);
        let chain2 = Box::new(vec![1, 2, 4]);
        let (result, ops) = dtw.calculate(chain1, chain2);
        println!("{:?}", ops);
        assert_eq!(result, 3.0);
    }

    #[test]
    fn testwindow() {
        assert_eq!(2 + 2, 4);
        let distance = STRACDistance::default();
        let dtw = WindowedDTW::new(3, &distance);
        let chain1 = Box::new(vec![1, 2, 3, 5, 2, 3, 4]);
        let chain2 = Box::new(vec![1, 2, 4, 6, 7, 1, 2]);
        let (result, ops) = dtw.calculate(chain1, chain2);
        println!("{:?}", ops);
        assert_eq!(result, 8.0);
    }

    #[test]
    fn test_eq() {
        assert_eq!(2 + 2, 4);
        let distance = STRACDistance::default();
        let dtw = StandardDTW::new(&distance);
        let chain1 = Box::new(vec![1, 2, 3, 5]);
        let chain2 = Box::new(vec![1, 2, 4]);
        let (result, ops) = dtw.calculate(chain1.clone(), chain2.clone());

        let dtw1 = FixedDTW::new(&distance);
        let (result2, ops2) = dtw.calculate(chain1.clone(), chain2.clone());

        let dtw2 = UnsafeDTW::new(&distance);
        let (result3, ops3) = dtw.calculate(chain1.clone(), chain2.clone());

        assert_eq!(result, result2);
        assert_eq!(result2, result3);
        assert_eq!(ops, ops2);
        assert_eq!(ops2, ops3);
    }
}
