//! Core traits for DTW implementations
//!
//!

/// Type for the trace token
pub type TokenID = usize;
/// Operation type in the warp path
pub type OP = (usize, usize);

pub type DTWResult = (f64, Option<(Vec<OP>, usize, usize)>);

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

    fn get_half(&self) -> Box<dyn Accesor>;
}

pub trait DTW {
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> DTWResult;

    fn can_provide_alignment(&self) -> bool {
        false
    }

    fn get_warp_path(
        &self,
        map: &[Vec<f64>],
        window: Option<DynamicWindow>,
    ) -> (Vec<OP>, usize, usize) {
        // We always start in the end of the alignment
        let mut i = map.len() - 1;
        let mut j = map[0].len() - 1;
        let mut r = vec![];

        let mut minI = map.len() - 1;
        let mut minJ = map[0].len() - 1;

        while i > 0 || j > 0 {
            let mut diagcost = std::f64::INFINITY;
            let mut leftcost = std::f64::INFINITY;
            let mut rightcost = std::f64::INFINITY;

            if i > 0 && j > 0 {
                match &window {
                    Some(window) => {
                        if window.is_in_range(i - 1, j - 1) {
                            diagcost = map[i - 1][j - 1];
                        }
                    }
                    None => {
                        diagcost = map[i - 1][j - 1];
                    }
                }
            }

            if i > 0 {
                match &window {
                    Some(window) => {
                        if window.is_in_range(i - 1, j) {
                            leftcost = map[i - 1][j];
                        }
                    }
                    None => {
                        leftcost = map[i - 1][j];
                    }
                }
            }

            if j > 0 {
                match &window {
                    Some(window) => {
                        if window.is_in_range(i, j - 1) {
                            rightcost = map[i][j - 1];
                        }
                    }
                    None => {
                        rightcost = map[i][j - 1];
                    }
                }
            }

            if i > 0 && j > 0 && diagcost <= leftcost && diagcost <= rightcost {
                // The diagonal is better
                i -= 1;
                j -= 1;
            } else if i > 0 && leftcost <= diagcost && leftcost <= rightcost {
                i -= 1;
            } else if j > 0 && rightcost <= diagcost && rightcost <= leftcost {
                j -= 1;
            } else if j > 0 && i <= j {
                j -= 1;
            } else if i > 0 {
                i -= 1;
            } else {
                break;
            }

            if i < minI {
                minI = i;
            }

            if j < minJ {
                minJ = j;
            }
            // Push the operation
            r.push((i, j));
        }

        (r, minI, minJ)
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

    fn get_half(&self) -> Box<dyn Accesor> {
        let mut v = Vec::with_capacity(self.len() / 2);

        // We take the first of the consecutive pairs
        for i in 0..self.len() {
            if i % 2 == 0 {
                v.push(self[i]);
            }
        }

        Box::new(v)
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

#[derive(Clone, Debug)]
pub struct DynamicWindow {
    min_values: Vec<Option<usize>>,
    max_values: Vec<Option<usize>>,
    width: usize,
}

impl DynamicWindow {
    #[inline]
    pub fn get_max(&self, row: usize) -> Option<usize> {
        self.max_values[row]
    }

    #[inline]
    pub fn get_min(&self, row: usize) -> Option<usize> {
        self.min_values[row]
    }

    #[inline]
    pub fn get_limits(&self, row: usize) -> (Option<usize>, Option<usize>) {
        (self.get_min(row), self.get_max(row))
    }

    #[inline]
    pub fn is_in_range(&self, row: usize, col: usize) -> bool {
        match (self.min_values[row], self.max_values[row]) {
            (Some(min), Some(max)) => min <= col && col <= max,
            _ => false,
        }
    }

    #[inline]
    pub fn expand(&mut self, row: usize, newlim: usize) {
        if row >= self.min_values.len() {
            return;
        }
        match (self.min_values[row], self.max_values[row]) {
            (None, None) => {
                self.min_values[row] = Some(newlim);
                self.max_values[row] = Some(newlim);
            }
            (Some(minval), None) => {
                if minval > newlim {
                    self.min_values[row] = Some(newlim);
                }
                //self.max_values[row] = Some(newlim);
            }
            (None, Some(maxval)) => {
                if maxval < newlim {
                    self.max_values[row] = Some(newlim);
                }
                //self.min_values[row] = Some(newlim);
            }
            (Some(minval), Some(maxval)) => {
                if maxval < newlim {
                    self.max_values[row] = Some(newlim);
                }
                if minval > newlim {
                    self.min_values[row] = Some(newlim);
                }
            }
        }
    }

    #[inline]
    pub fn set_min(&mut self, min: usize, row: usize) {
        self.min_values[row] = Some(min);
    }

    #[inline]
    pub fn set_max(&mut self, max: usize, row: usize) {
        self.max_values[row] = Some(max);
    }

    #[inline]
    pub fn set_range(&mut self, min: usize, max: usize, row: usize) {
        self.min_values[row] = Some(min);
        self.max_values[row] = Some(max);
    }

    #[inline]
    pub fn init(&mut self, height: usize) -> &mut Self {
        for i in 0..height {
            self.set_range(0, self.width, i);
        }
        self
    }

    pub fn new(height: usize, width: usize) -> Self {
        DynamicWindow {
            min_values: vec![None; height],
            max_values: vec![None; height],
            width,
        }
    }

    pub fn replicate_prev(&mut self, i: usize) -> &mut Self {
        self.min_values[i] = self.min_values[i - 1];
        self.max_values[i] = self.max_values[i - 1];

        self
    }
    pub fn replicate_last_row(&mut self) -> &mut Self {
        let last_row = self.min_values.len() - 1;
        let prev_last_row = last_row - 1;
        self.min_values[last_row] = self.min_values[prev_last_row];
        self.max_values[last_row] = self.max_values[prev_last_row];

        self
    }

    pub fn display(&self) {
        println!("---------------");
        for i in 0..self.min_values.len() {
            // log::info!("{i}{}-{}", self.min_values[i], self.max_values[i]);
            print!("{i}");
            for j in 0..self.min_values[i].or(Some(0)).unwrap() {
                print!(" ");
            }
            for j in
                self.min_values[i].or(Some(0)).unwrap()..self.max_values[i].or(Some(0)).unwrap()
            {
                print!("█");
            }
            for j in self.max_values[i].or(Some(self.width)).unwrap()..self.width {
                print!(" ");
            }
            println!("");
        }
        println!("--------------")
    }
}

pub struct WindowedDTW<'a> {
    window: DynamicWindow,
    distance: &'a dyn Distance,
}

impl<'a> WindowedDTW<'a> {
    pub fn new(window: DynamicWindow, distance: &'a dyn Distance) -> Self {
        WindowedDTW { window, distance }
    }
}

impl<'a> DTW for WindowedDTW<'a> {
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> DTWResult {
        // Do slices
        // We do it with the max MEM possible
        let mut dtw = vec![vec![std::f64::INFINITY; chain2.size() + 1]; chain1.size() + 1];
        let mut dtw = dtw.as_mut_slice();

        for i in 0..=chain1.size() {
            let (min, max) = self.window.get_limits(i);

            match (min, max) {
                (Some(min), Some(max)) => {
                    for j in min..=max.min(chain2.size()) {
                        // patch here ?

                        match (i, j) {
                            (0, 0) => dtw[0][0] = 0.0,
                            // First column
                            (0, _) => dtw[0][j] = self.distance.gap_cost() * j as f64,
                            // First row
                            (i, 0) => dtw[i][0] = self.distance.gap_cost() * i as f64,
                            _ => {
                                let a = chain1.get(i - 1);
                                let b = chain2.get(j - 1);

                                // If i - 1, j - 1 are outside the window then return INFINITE
                                let diagcost = if self.window.is_in_range(i - 1, j - 1)
                                    || (i - 1) <= chain1.size()
                                    || (j - 1) <= chain2.size()
                                {
                                    self.distance.distance(a, b) + dtw[i - 1][j - 1]
                                } else {
                                    std::f64::INFINITY / 2.0
                                };

                                let leftcost = if self.window.is_in_range(i - 1, j)
                                    || (i - 1) <= chain1.size()
                                    || (j) <= chain2.size()
                                {
                                    self.distance.gap_cost() + dtw[i - 1][j]
                                } else {
                                    std::f64::INFINITY / 2.0
                                };

                                let rightcost = if self.window.is_in_range(i, j - 1)
                                    || (i) <= chain1.size()
                                    || (j - 1) <= chain2.size()
                                {
                                    self.distance.gap_cost() + dtw[i][j - 1]
                                } else {
                                    std::f64::INFINITY / 2.0
                                };

                                let mini = diagcost.min(leftcost).min(rightcost);

                                dtw[i][j] = mini;
                            }
                        }
                    }
                }
                _ => {
                    // Do nothing
                }
            }
        }

        /*
        for i in 0..dtw.len() {
            for j in 0..dtw[i].len() {
                //print!("{} ", dtw[i][j]);
            }
            ://println!();
        }*/

        let cost = dtw[chain1.size() - 1][chain2.size() - 1];
        let path = self.get_warp_path(&dtw, Some(self.window.clone()));

        (cost, Some(path))
    }
}

/*
fn reduce_by_half<'a, T>(allocator: &'a dyn AccesorAllocator<T>, accessor: &mut Box<T>) -> T
where
    T: Accesor,
{
    // TODO create a random strategy as well
    let mut r = allocator.allocate(accessor.size() / 2);
    for i in 0..r.size() {
        allocator.set(i, accessor.get(i * 2), &mut r);
    }
    r
}*/

pub struct FastDTW<'a> {
    distance: &'a dyn Distance,
    radius: usize,
    default_dtw: &'a dyn DTW,
    min_size: usize,
}

impl<'a> FastDTW<'a> {
    pub fn new(
        distance: &'a dyn Distance,
        radius: usize,
        min_size: usize,
        default_dtw: &'a dyn DTW,
    ) -> FastDTW<'a> {
        FastDTW {
            distance,
            radius,
            min_size,
            default_dtw,
        }
    }

    // expands the path
    fn expand(
        ops: Vec<OP>,
        radius: usize,
        len1: usize,
        len2: usize,
        op_count: usize,
        mini: usize,
        minj: usize,
    ) -> DynamicWindow {
        let mut dynamic_window = DynamicWindow::new(len2 + 1, len1);
        //let mut dynamic_window = dynamic_window.init(len1);

        let mut lastwarpedi = usize::MAX;
        let mut lastwarpedj = usize::MAX;

        let blocksize = 2;
        let mut currenti = mini;
        let mut currentj = minj;

        //println!("{} {}", len1, len2);
        for i in (0..op_count).rev() {
            let (warpedi, warpedj) = ops[i];
            //println!("{} {}", warpedi + 1, warpedj + 1);

            if warpedi > lastwarpedi {
                currenti += blocksize;
            }

            if warpedj > lastwarpedj {
                currentj += blocksize;
            }

            if (warpedj > lastwarpedj) && (warpedi > lastwarpedi) {
                dynamic_window.expand(currenti - 1, currentj);
                dynamic_window.expand(currenti, currentj - 1);
            }

            for x in 0..blocksize {
                dynamic_window.expand(currenti + x, currentj);
                dynamic_window.expand(currenti + x, currentj + blocksize - 1);
            }

            lastwarpedi = warpedi;
            lastwarpedj = warpedj;
        }

        if len2 % 2 == 1 && len2 > 0 {
            dynamic_window.replicate_last_row();
        }


        let mut scaled = DynamicWindow::new(len2 + 1, len1);

        for i in 0..len2 + 1 {
            let min_col = dynamic_window.get_min(i);
            let max_col = dynamic_window.get_max(i);

            match (min_col, max_col) {
                (Some(min_col), Some(max_col)) => {
                    let up_row = 0.max(i as i32 - radius as i32);
                    let down_row = len1.min(i + radius);

                    scaled.expand(i, min_col);
                    scaled.expand(i, max_col);

                    let (sub, s) = min_col.overflowing_sub(radius);

                    if s {
                        scaled.expand(i, 0);
                    } else {
                        scaled.expand(i, sub);
                    }

                    let (val, s) = max_col.overflowing_add(radius);
                    if s {
                        scaled.expand(i, len1);
                    } else {
                        scaled.expand(i, len1.min(val));
                    }

                    for j in (up_row as usize..=i).rev() {
                        scaled.expand(j, max_col);
                        scaled.expand(j, scaled.get_min(j).or(None).unwrap());
                    }

                    for j in i..=down_row {
                        scaled.expand(j, min_col);
                        scaled.expand(j, scaled.get_max(j).or(None).unwrap());
                    }
                }
                (_, _) => {
                    scaled.replicate_prev(i);
                }
            }

            //scaled.set(i, max_col.overflowing_add(radius).min(len1).0);
        }


        scaled.clone()
    }
}

impl DTW for FastDTW<'_> {
    fn calculate(&self, chain1: Box<dyn Accesor>, chain2: Box<dyn Accesor>) -> DTWResult {
        if chain1.size() <= self.min_size || chain2.size() <= self.min_size {
            log::info!(
                "Min trace size reached in FastDTW {} {}",
                chain1.size(),
                chain2.size()
            );
            let r = self.default_dtw.calculate(chain1, chain2);

            log::info!("Returning from basic DTW");

            return r;
        }

        let chain1_half = chain1.get_half();
        let chain2_half = chain2.get_half();

        // TODO move this to a queue. Yet, we do not have that many stack calls, log(n) at most
        let (_, path) = self.calculate(chain1_half, chain2_half);

        // Expand the path

        if let Some((path, mini, minj)) = path {
            log::info!("Windowed fdtw ");
            let opcount = path.len();
            let window = FastDTW::expand(
                path,
                self.radius,
                chain1.size(),
                chain2.size(),
                opcount,
                mini,
                minj,
            );

            log::info!("{:?}", window);
            return WindowedDTW::new(window, self.distance).calculate(chain1, chain2);
        }

        // expand
        panic!("This point should not be reached")
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
    fn testfast() {
        assert_eq!(2 + 2, 4);
        let distance = STRACDistance::default();
        let dtw = StandardDTW::new(&distance);

        let fastdtw = FastDTW::new(&distance, 2, 100, &dtw);
        let chain1 = Box::new(vec![1, 2, 3, 5, 1, 2, 3]);
        let chain2 = Box::new(vec![1, 2, 4, 5, 6, 7, 8]);
        let (result, ops) = fastdtw.calculate(chain1, chain2);
        println!("{:?}", ops);
        assert_eq!(result, 8.0);
    }

    #[test]
    fn testfast2() {
        assert_eq!(2 + 2, 4);
        let distance = STRACDistance::default();
        let dtw = StandardDTW::new(&distance);

        let fastdtw = FastDTW::new(&distance, 2, 10, &dtw);
        let chain2 = Box::new(vec![
            18446744071612399615,
            2692975965417482751,
            2676586395008836901,
            18446743133734905125,
            18446744072261074943,
            12225488737194213375,
            18446648525771614633,
            4991472727824007167,
            288230376151711743,
            18446744073703912105,
            18446648713712893951,
            6872316420869324799,
            2676586395008836901,
            18385141895277126949,
            18446744073709551615,
            4557430888798879743,
            4557430888798830399,
            4557430888798830399,
            3026418949580341055,
            18446744073709551615,
            18446648673895972863,
            18422331914565189631,
            12225307061520433151,
            18446744073709551615,
            18446744073703867717,
            2676650416768245673,
            2676586395008836901,
            18446744070037775653,
            18446744073709551615,
            4557430888798830399,
            4557430888798830399,
            4557430888798830399,
            18446743154586550271,
            18446744073709551615,
            12225489209634929663,
            18446744071688380255,
            6893227250828290363,
            2676586395012652895,
            2676586395008836901,
            2089670227099909925,
            18446744073709551615,
            18446744073709496831,
            12197231332752883711,
            6872398473367388159,
            2676586395008851807,
            2676586395008836901,
            18446744073709551615,
            18392488947314851839,
            18446718784928088063,
            18446744073709551615,
            18446743133734905125,
            18446744073709551615,
            4557430888798830399,
            4557430888798830399,
            18446744070475693887,
            4557430888798879743,
            4557430888798830399,
            4557430888798830399,
            3026418949580341055,
            18446744073709551615,
            12225488738637053951,
            2692975965417482665,
            2676586395008836901,
            18446743133734905125,
            18446744073694674943,
            18446508778221207551,
        ]);
        let chain1 = Box::new(vec![
            18446744073709551615,
            2676827028518338559,
            18446743133734905125,
            18446744073709551615,
            4557430888798830399,
            18446743245841973055,
            18446743274845634559,
            12225489207950704639,
            12249790986447749119,
            1849195666009074089,
            18446744073709529369,
            4557431168072762879,
            4557430888798830399,
            4557430888798830399,
            18386508428693421887,
            18446744073709551615,
            9765923333140381695,
            18446611614896981895,
            18394389728041369599,
            12225378830423949311,
            18446744073709551529,
            12225489209634929577,
            1873497191583055871,
        ]);
        let (result, ops) = fastdtw.calculate(chain1, chain2);
        println!("{:?}", ops);
        assert_eq!(result, 73.0);
    }

    #[test]
    fn testfast3() {
        assert_eq!(2 + 2, 4);
        let distance = STRACDistance::default();
        let dtw = StandardDTW::new(&distance);

        let fastdtw = FastDTW::new(&distance, 2, 10, &dtw);
        let chain2 = Box::new(vec![
            12184914412422823935,
            5044031582654955519,
            18374686479671601477,
            18446744072261052675,
            18422331914565189631,
            6872316740139745279,
            6872316169532286303,
            2676586395008836901,
            18385141895277126949,
            2676827028518338559,
            2676586395008836901,
            18446744073709551615,
            18446744073709551615,
        ]);
        let chain1 = Box::new(vec![
            6872398473367388159,
            2676586395008851807,
            2676586395008836901,
            18446744073709551615,
            4557430892032688127,
            4557430888798830399,
            6872398102544727871,
            2676586395008851807,
            2676586395008836901,
            18382849253996232703,
            18446744073709551615,
            18446744073323675433,
            18446744073709551615,
        ]);
        let (result, ops) = fastdtw.calculate(chain1, chain2);
        println!("{:?}", ops);
        assert_eq!(result, 18.0);
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
