use std::{
    fs::OpenOptions,
    io::{Error, ErrorKind, Result, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    evaluate::{evaluate_with, lazy_evaluate_with},
    params::{DEFAULT_LAZY_PARAMS, DEFAULT_PARAMS, LazyParams, Params},
    tuning::{
        dataset::load_epd_samples,
        emit::{
            dump_full_params, dump_lazy_params, fmt_score, render_full_params, render_lazy_params,
        },
        texel::{fit_k, loss},
        types::{FullTuningConfig, LazyTuningConfig, ParamMeta, Sample, TuningConfig},
    },
};

fn calibrate_a<C, F>(
    config: &C,
    theta: &[i32],
    c: f64,
    a_cap: f64,
    alpha: f64,
    desired_first_step: f64,
    meta: &[ParamMeta],
    loss_fn: F,
) -> f64
where
    C: TuningConfig,
    F: Fn(&C::ParamType) -> f64,
{
    let trials = 8;
    let mut grad_sum = 0.0;
    let mut grad_n = 0usize;

    let mut rng = SmallRng::seed_from_u64(67);
    let mut delta = vec![0; theta.len()];
    let mut plus = vec![0; theta.len()];
    let mut minus = vec![0; theta.len()];

    for _ in 0..trials {
        let c_i = c.round() as i32;

        for i in 0..theta.len() {
            if !meta[i].active {
                plus[i] = theta[i];
                minus[i] = theta[i];
                continue;
            }
            delta[i] = if rng.random_bool(0.5) { 1 } else { -1 };

            let step = c_i * delta[i];
            plus[i] = (theta[i] + step).clamp(meta[i].bounds.min, meta[i].bounds.max);
            minus[i] = (theta[i] - step).clamp(meta[i].bounds.min, meta[i].bounds.max);
        }

        let mut params_plus = C::unpack(&plus);
        let mut params_minus = C::unpack(&minus);
        config.project(&mut params_plus);
        config.project(&mut params_minus);

        let loss_plus = loss_fn(&params_plus);
        let loss_minus = loss_fn(&params_minus);

        for i in 0..theta.len() {
            if !meta[i].active {
                continue;
            }

            let g_i = (loss_plus - loss_minus) / (2.0 * c * delta[i] as f64);
            grad_sum += g_i.abs();
            grad_n += 1;
        }
    }

    let mean_abs_grad = grad_sum / grad_n as f64;

    if mean_abs_grad == 0.0 {
        return 0.1;
    }

    desired_first_step * (a_cap + 1.0).powf(alpha) / mean_abs_grad
}

pub fn tune_full(path: &Path, stop: &AtomicBool) -> Result<()> {
    let config = FullTuningConfig::default();

    // let samples = load_samples(path).unwrap();
    let samples = load_epd_samples(path)?;

    // Fitted k value
    // let k = fit_k(&samples, &DEFAULT_PARAMS, &evaluate_with);
    let k = 1.392;
    println!("fitted k value: {k}");

    let loss_fn = |p: &Params| loss(&samples, p, &evaluate_with, k);

    let describe = |p: &Params| {
        format!(
            "pawn: {}, knight: {}, bishop: {}, rook: {}, queen: {}",
            fmt_score(p.pawn_value),
            fmt_score(p.knight_value),
            fmt_score(p.bishop_value),
            fmt_score(p.rook_value),
            fmt_score(p.queen_value),
        )
    };

    let dump = dump_full_params;
    let render = render_full_params;

    optimise(&config, &samples, stop, loss_fn, describe, dump, render)
}

pub fn tune_lazy(path: &Path, stop: &AtomicBool) -> Result<()> {
    let config = LazyTuningConfig::default();

    // let samples = load_samples(path).unwrap();
    let samples = load_epd_samples(path)?;

    // Fitted k value
    // let k = fit_k(&samples, &DEFAULT_LAZY_PARAMS, &lazy_evaluate_with);
    let k = 1.662;
    println!("fitted k value: {k}");

    let loss_fn = |p: &LazyParams| loss(&samples, p, &lazy_evaluate_with, k);

    let describe = |p: &LazyParams| {
        format!(
            "pawn: {}, knight: {}, bishop: {}, rook: {}, queen: {}",
            fmt_score(p.pawn_value),
            fmt_score(p.knight_value),
            fmt_score(p.bishop_value),
            fmt_score(p.rook_value),
            fmt_score(p.queen_value),
        )
    };

    let dump = dump_lazy_params;
    let render = render_lazy_params;

    optimise(&config, &samples, stop, loss_fn, describe, dump, render)
}

fn optimise<C, F, G, H, R>(
    config: &C,
    samples: &[Sample],
    stop: &AtomicBool,
    loss_fn: F,
    describe: G,
    dump: H,
    render: R,
) -> Result<()>
where
    C: TuningConfig,
    F: Fn(&C::ParamType) -> f64 + Copy,
    G: Fn(&C::ParamType) -> String,
    H: Fn(&str, &[i32], f64),
    R: Fn(&str, &[i32], f64) -> String,
{
    if samples.is_empty() {
        return Err(Error::new(ErrorKind::InvalidData, "empty tuning dataset"));
    }

    let iterations = 50_000;
    let alpha = 0.602;
    let gamma = 0.101;
    let c = 2.0;
    let A = 0.1 * iterations as f64;

    let meta = config.flat_param_meta();
    let snapshot_every = std::env::var("SPSA_SNAPSHOT_EVERY")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(1000);
    let mut snapshot_file = std::env::var("SPSA_SNAPSHOT_FILE")
        .ok()
        .map(|path| {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let snapshot_path = format!("{path}.{ts}.txt");

            OpenOptions::new()
                .create(true)
                .append(true)
                .open(snapshot_path)
        })
        .transpose()?;

    let mut params = config.default_params();
    config.project(&mut params);
    let mut theta = config.pack(&params);

    debug_assert_eq!(theta.len(), meta.len());

    let a = calibrate_a(config, &theta, c, A, alpha, 0.25, &meta, loss_fn);
    println!("a value: {a}");

    let baseline_loss = loss_fn(&params);

    let mut best_theta = theta.clone();
    let mut current_loss = baseline_loss;
    let mut best_loss = baseline_loss;

    let mut rng = SmallRng::seed_from_u64(67);
    let mut delta = vec![0; theta.len()];
    let mut plus = vec![0; theta.len()];
    let mut minus = vec![0; theta.len()];

    for t in 0..iterations {
        if stop.load(Ordering::Relaxed) {
            println!("Interrupted at iter={t}");
            break;
        }

        let a_t = a / (t as f64 + 1.0 + A).powf(alpha);
        let c_t = c / (t as f64 + 1.0).powf(gamma);
        let c_t_round = c_t.round() as i32;
        let should_snapshot = snapshot_file.is_some() && (t + 1) % snapshot_every == 0;

        for i in 0..theta.len() {
            if !meta[i].active {
                plus[i] = theta[i];
                minus[i] = theta[i];
                continue;
            }
            delta[i] = if rng.random_bool(0.5) { 1 } else { -1 };

            let step = c_t_round * delta[i];
            plus[i] = (theta[i] + step).clamp(meta[i].bounds.min, meta[i].bounds.max);
            minus[i] = (theta[i] - step).clamp(meta[i].bounds.min, meta[i].bounds.max);
        }

        // Apply projection to plus/minus to ensure values are logical
        let mut params_plus = C::unpack(&plus);
        let mut params_minus = C::unpack(&minus);
        config.project(&mut params_plus);
        config.project(&mut params_minus);

        let loss_plus = loss_fn(&params_plus);
        if stop.load(Ordering::Relaxed) {
            println!("Interrupted at iter={t} after loss_plus");
            break;
        }

        let loss_minus = loss_fn(&params_minus);
        if stop.load(Ordering::Relaxed) {
            println!("Interrupted at iter={t} after loss_minus");
            break;
        }

        for i in 0..theta.len() {
            if !meta[i].active {
                continue;
            }

            let g_i = (loss_plus - loss_minus) / (2.0 * c_t * delta[i] as f64);
            theta[i] = (theta[i] as f64 - a_t * g_i).round() as i32;
            theta[i] = theta[i].clamp(meta[i].bounds.min, meta[i].bounds.max);
        }

        // Apply project to theta to ensure values are logical
        params = C::unpack(&mut theta);
        config.project(&mut params);
        theta = config.pack(&params);

        // Only calculate exact loss every 25 iterations as
        // this saves ~30% of optimisation compute time.
        // Otherwise, use an approximation for current loss
        if t % 25 == 0 || should_snapshot {
            current_loss = loss_fn(&params);

            println!("{}", describe(&params));
            println!("iter={t}, current_loss={current_loss}, best_loss={best_loss}");
        } else {
            current_loss = (loss_plus + loss_minus) / 2.0;
        }

        if current_loss < best_loss {
            best_loss = current_loss;
            best_theta.copy_from_slice(&theta);
        }

        if should_snapshot {
            let iter = t + 1;
            let snapshot = format!(
                "# iter={iter}, current_loss={current_loss}, best_loss={best_loss}\n{}\n\n{}\n\n",
                render("current", &theta, current_loss),
                render("best", &best_theta, best_loss)
            );

            if let Some(file) = &mut snapshot_file {
                file.write_all(snapshot.as_bytes())?;
                file.flush()?;
            }
        }
    }

    dump("current", &theta, current_loss);
    dump("best", &best_theta, best_loss);

    Ok(())
}
