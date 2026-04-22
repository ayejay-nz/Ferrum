use std::{
    env,
    io::Result,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use std::sync::OnceLock;

use ferrum::tuning::spsa::{tune_full, tune_lazy};

#[derive(Copy, Clone)]
enum TuneMode {
    Full,
    Lazy,
}

fn print_usage(bin: &str) {
    eprintln!("Usage: {bin} [full|lazy] [epd_path]");
    eprintln!("Defaults: mode=full, epd_path=quiet-labeled.epd");
}

fn parse_cli() -> Result<(TuneMode, String)> {
    let mut args = env::args();
    let bin = args.next().unwrap_or_else(|| "tuner".to_owned());

    let mut mode = TuneMode::Full;
    let mut path = "quiet-labeled.epd".to_owned();

    let first = args.next();
    let second = args.next();

    if args.next().is_some() {
        print_usage(&bin);
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "too many arguments",
        ));
    }

    match (first.as_deref(), second.as_deref()) {
        (None, None) => {}
        (Some("--help" | "-h"), None) => {
            print_usage(&bin);
            std::process::exit(0);
        }
        (Some("full"), None) => {}
        (Some("lazy"), None) => mode = TuneMode::Lazy,
        (Some("full"), Some(epd_path)) => path = epd_path.to_owned(),
        (Some("lazy"), Some(epd_path)) => {
            mode = TuneMode::Lazy;
            path = epd_path.to_owned();
        }
        (Some(epd_path), None) => path = epd_path.to_owned(),
        _ => {
            print_usage(&bin);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid arguments",
            ));
        }
    }

    Ok((mode, path))
}

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

    let (mode, path) = parse_cli()?;
    let epd = Path::new(&path);

    match mode {
        TuneMode::Full => tune_full(epd, stop.as_ref())?,
        TuneMode::Lazy => tune_lazy(epd, stop.as_ref())?,
    }

    Ok(())
}
