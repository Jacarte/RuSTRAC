use anyhow::Result;
use clap::Parser;
use dtw_core::dtw::DTW;

/// Standard DTW implementation.
#[derive(Parser, Clone)]
pub struct Opts {
    #[clap(flatten)]
    io: dtw_tools::InputOutput,

    #[arg(long, default_value = "2")]
    window_size: usize,
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
        let general = &self.io;

        let dtw = dtw::dtw::FixedDTW::new(&*distance);
        dtw.calculate(tr1, tr2)
    }
}
