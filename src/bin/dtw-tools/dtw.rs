
use clap::Parser;
use dtw_core::dtw::DTW;

/// Standard DTW implementation.

#[derive(Parser, Clone)]
pub struct Opts {
    #[clap(flatten)]
    io: dtw_tools::InputOutput,
}

impl Opts {
    pub fn general_opts(&self) -> &dtw_tools::GeneralOpts {
        self.io.general_opts()
    }

    pub fn io(&self) -> &dtw_tools::InputOutput {
        &self.io
    }

    pub fn run(&self, tr1: Box<dyn dtw::dtw::Accesor>, tr2: Box<dyn dtw::dtw::Accesor>, distance: Box<dyn dtw::dtw::Distance>) ->  dtw::dtw::DTWResult {
        // Initialize the DTWStandard
        let _general = &self.io;

        let dtw = dtw::dtw::StandardDTW::new(&*distance);
        dtw.calculate(tr1, tr2)
    }
}

