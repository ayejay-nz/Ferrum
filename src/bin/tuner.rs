use std::{
    io::Result,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use ferrum::tuning::spsa::tune;
use std::sync::OnceLock;

pub fn init_thread_pool() {
    static INIT: OnceLock<()> = OnceLock::new();

    INIT.get_or_init(|| {
        let threads = std::env::var("RAYON_NUM_THREADS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&n| n > 0)
            .unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(usize::from)
                    .unwrap_or(1)
            });

        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .expect("failed to initialize rayon global thread pool");
    });
}

fn main() -> Result<()> {
    init_thread_pool();

    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = Arc::clone(&stop);
        ctrlc::set_handler(move || {
            stop.store(true, Ordering::Relaxed);
        })
        .expect("failed to install Ctrl+C handler");
    }

    // let fens = Path::new("fens.txt");
    let fens = Path::new("quiet-labeled.epd");
    tune(fens, stop.as_ref())?;
    Ok(())
}
