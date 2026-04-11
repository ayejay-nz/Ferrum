use std::{
    io::{Error, ErrorKind, Result},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    evaluate::{evaluate_with, lazy_evaluate_with},
    tune::{DEFAULT_LAZY_PARAMS, DEFAULT_PARAMS, LazyParams, ParamBounds, Params, TunableParams},
    tuning::{
        dataset::load_epd_samples,
        emit::{dump_full_params, dump_lazy_params, fmt_score},
        texel::{fit_k, loss},
        types::Sample,
    },
};

fn calibrate_a<P, F>(
    theta: &[i32],
    c: f64,
    a_cap: f64,
    alpha: f64,
    desired_first_step: f64,
    bounds: &[ParamBounds],
    loss_fn: F,
) -> f64
where
    P: TunableParams,
    F: Fn(&P) -> f64,
{
    let trials = 8;
    let mut grad_sum = 0.0;
    let mut grad_n = 0usize;

    for _ in 0..trials {
        let delta: Vec<i32> = (0..theta.len())
            .map(|_| if rand::random::<bool>() { 1 } else { -1 })
            .collect();

        let mut plus = theta.to_vec();
        let mut minus = theta.to_vec();

        let c_i = c.round() as i32;

        for i in 0..theta.len() {
            plus[i] = (plus[i] + c_i * delta[i]).clamp(bounds[i].min, bounds[i].max);
            minus[i] = (minus[i] - c_i * delta[i]).clamp(bounds[i].min, bounds[i].max);
        }

        let mut params_plus = P::unpack(&plus);
        let mut params_minus = P::unpack(&minus);
        params_plus.project();
        params_minus.project();

        let loss_plus = loss_fn(&params_plus);
        let loss_minus = loss_fn(&params_minus);

        for i in 0..theta.len() {
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
    // let samples = load_samples(path).unwrap();
    let samples = load_epd_samples(path)?;

    // Fitted k value
    // let k = fit_k(&samples, &DEFAULT_PARAMS, &evaluate_with);
    let k = 1.377;
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

    optimise(&samples, stop, loss_fn, describe, dump)
}

pub fn tune_lazy(path: &Path, stop: &AtomicBool) -> Result<()> {
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

    optimise(&samples, stop, loss_fn, describe, dump)
}

fn optimise<P, F, G, H>(
    samples: &[Sample],
    stop: &AtomicBool,
    loss_fn: F,
    describe: G,
    dump: H,
) -> Result<()>
where
    P: TunableParams,
    F: Fn(&P) -> f64 + Copy,
    G: Fn(&P) -> String,
    H: Fn(&str, &[i32], f64),
{
    if samples.is_empty() {
        return Err(Error::new(ErrorKind::InvalidData, "empty tuning dataset"));
    }

    let iterations = 50_000;
    let alpha = 0.602;
    let gamma = 0.101;
    let c = 2.0;
    let A = 0.1 * iterations as f64;

    let bounds = P::flat_bounds();

    let mut params = P::default();
    params.project();
    let mut theta = params.pack();

    debug_assert_eq!(theta.len(), bounds.len());

    let a = calibrate_a(&theta, c, A, alpha, 1.0, &bounds, loss_fn);
    println!("a value: {a}");

    let baseline_loss = loss_fn(&params);

    let mut best_theta = theta.clone();
    let mut current_loss = baseline_loss;
    let mut best_loss = baseline_loss;

    for t in 0..iterations {
        if stop.load(Ordering::Relaxed) {
            println!("Interrupted at iter={t}");
            break;
        }

        let a_t = a / (t as f64 + 1.0 + A).powf(alpha);
        let c_t = c / (t as f64 + 1.0).powf(gamma);

        let delta: Vec<i32> = (0..theta.len())
            .map(|_| if rand::random::<bool>() { 1 } else { -1 })
            .collect();

        let mut plus = theta.clone();
        let mut minus = theta.clone();

        for i in 0..theta.len() {
            plus[i] += (c_t.round() as i32) * delta[i];
            minus[i] -= (c_t.round() as i32) * delta[i];
            plus[i] = plus[i].clamp(bounds[i].min, bounds[i].max);
            minus[i] = minus[i].clamp(bounds[i].min, bounds[i].max);
        }

        // Apply projection to theta to ensure values are logical
        let mut params = P::unpack(&theta);
        let mut params_plus = P::unpack(&plus);
        let mut params_minus = P::unpack(&minus);
        params.project();
        params_plus.project();
        params_minus.project();
        plus = params_plus.pack();
        minus = params_minus.pack();
        theta = params.pack();

        let loss_plus = loss_fn(&P::unpack(&plus));
        if stop.load(Ordering::Relaxed) {
            println!("Interrupted at iter={t} after loss_plus");
            break;
        }

        let loss_minus = loss_fn(&P::unpack(&minus));
        if stop.load(Ordering::Relaxed) {
            println!("Interrupted at iter={t} after loss_minus");
            break;
        }

        for i in 0..theta.len() {
            let g_i = (loss_plus - loss_minus) / (2.0 * c_t * delta[i] as f64);
            theta[i] = (theta[i] as f64 - a_t * g_i).round() as i32;
            theta[i] = theta[i].clamp(bounds[i].min, bounds[i].max);
        }

        // Apply project to theta to ensure values are logical
        let mut params = P::unpack(&theta);
        params.project();
        theta = params.pack();

        current_loss = loss_fn(&params);

        if current_loss < best_loss {
            best_loss = current_loss;
            best_theta = theta.clone();
        }

        if t % 100 == 0 {
            println!("{}", describe(&params));
            println!("iter={t}, current_loss={current_loss}, best_loss={best_loss}");
        }
    }

    dump("current", &theta, current_loss);
    dump("best", &best_theta, best_loss);

    Ok(())
}
