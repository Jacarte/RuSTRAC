use clap::Parser;
use dtw_core::dtw::FastDTW;
use dtw_core::dtw::StandardDTW;
use dtw_core::dtw::DTW;

/// Standard DTW implementation.
#[derive(Parser, Clone)]
pub struct Opts {
    #[clap(flatten)]
    io: dtw_tools::InputOutput,

    #[arg(long, default_value = "2")]
    window_size: usize,

    #[arg(long, default_value = "100")]
    min_dtw_size: usize,
}

impl Opts {
    pub fn general_opts(&self) -> &dtw_tools::GeneralOpts {
        self.io.general_opts()
    }

    pub fn io(&self) -> &dtw_tools::InputOutput {
        &self.io
    }

    pub fn run(
        &self,
        tr1: Box<dyn dtw::dtw::Accesor>,
        tr2: Box<dyn dtw::dtw::Accesor>,
        distance: Box<dyn dtw::dtw::Distance>,
    ) -> dtw::dtw::DTWResult {
        // Initialize the DTWStandard
        let dtw = StandardDTW::new(&*distance);

        let fastdtw = FastDTW::new(&*distance, self.window_size, self.min_dtw_size, &dtw);

        fastdtw.calculate(tr1, tr2)
    }
}
