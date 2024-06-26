use crate::rng;
use plotters::prelude::*;

use crate::dist;
use crate::dist::Distribution;

fn ground_truth_prog(x: f32) -> f32
{
    if x > 3.5
    {
        return 4.2 * x;
    }

    return x * 2.1;
}

struct Holes
{
    par_1: dist::Categorical,
    par_2: dist::Normal,
    par_3: dist::Categorical,
    par_4: dist::Normal,
    par_5: dist::Categorical,
    par_6: dist::Normal
}

fn hole_1(x: f32, sign: f32, val: f32) -> bool
{
    if sign == 0.0 {return x > val;}
    if sign == 1.0 {return x < val;}
    return x == val;
}

fn hole_1_str(sign: f32) -> String
{
    if sign == 0.0 {return ">".to_owned();}
    if sign == 1.0 {return "<".to_owned();}
    return "==".to_owned();
}

fn hole_2(x: f32, sign: f32, val: f32) -> f32
{
    if sign == 0.0 {return x + val;}
    if sign == 1.0 {return x - val;}
    if sign == 2.0 {return x * val;}
    return x / val;
}

fn hole_2_str(sign: f32) -> String
{
    if sign == 0.0 {return "+".to_owned();}
    if sign == 1.0 {return "-".to_owned();}
    if sign == 2.0 {return "*".to_owned();}
    return "/".to_owned();
}

fn synth_prog(x: f32, props: &Vec<f32>) -> f32
{
    if hole_1(x, props[0], props[1])
    {
        return hole_2(x, props[2], props[3]);
    }

    return hole_2(x, props[4], props[5]);
}

fn square(x: f32) -> f32
{
    return x * x;
}

pub fn run_exp1() {
    let root = BitMapBackend::new("charts/simple.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption("Simple Program Induction", ("sans-serif", 30).into_font())
        .margin(40)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(0f32..10000f32, (0f32..1000f32).log_scale()).unwrap();

    chart
        .configure_mesh()
        .x_desc("Iterations")
        .y_desc("Loss")
        .x_label_style(("sans-serif", 20).into_font())
        .y_label_style(("sans-serif", 20).into_font())
        .x_labels(10)
        .y_labels(10)
        .x_label_formatter(&|x| format!("{:.0}", x))
        .y_label_formatter(&|x| format!("{:.0}", x))
        .draw().unwrap();
    
    let mut rng = rng::RNG::new(10);

    let test_inputs = vec![1.0, 2.0, 4.0, 5.0];    
    let mut test_outputs = Vec::new();
    for t in test_inputs.clone()
    {
        test_outputs.push(ground_truth_prog(t));
    }
    
    let num_mutations = 50;
    let num_iters = 10000;
    let rate = 0.1;
    
    let par_1 = dist::Categorical::new(false, vec![0.0; 3]);
    let par_2 = dist::Normal::new(0.0, 1.0);
    let par_3 = dist::Categorical::new(false, vec![0.0; 4]);
    let par_4 = dist::Normal::new(0.0, 1.0);
    let par_5 = dist::Categorical::new(false, vec![0.0; 4]);
    let par_6 = dist::Normal::new(0.0, 1.0);
    
    let mut holes = Holes{par_1, par_2, par_3, par_4, par_5, par_6};
    let mut logger = Vec::new();

    for i in 0..num_iters
    {
        let mut trace_h1 = Vec::new();
        let mut trace_h2 = Vec::new();
        let mut trace_h3 = Vec::new();
        let mut trace_h4 = Vec::new();
        let mut trace_h5 = Vec::new();
        let mut trace_h6 = Vec::new();
        
        let mut objective = 0.0;
        let mut M = 0.0;
        let mut S = 0.0;

        for j in 0..num_mutations
        {
            let prop_1 = holes.par_1.sample(&mut rng) as f32;
            let prop_2 = holes.par_2.sample(&mut rng) as f32;
            let prop_3 = holes.par_3.sample(&mut rng) as f32;
            let prop_4 = holes.par_4.sample(&mut rng) as f32;
            let prop_5 = holes.par_5.sample(&mut rng) as f32;
            let prop_6 = holes.par_6.sample(&mut rng) as f32;
            
            let props = vec![prop_1, prop_2, prop_3, prop_4, prop_5, prop_6];
        
            let mut score = 0.0;
            for t in 0..test_inputs.len()
            {
                score += square(synth_prog(test_inputs[t], &props) - test_outputs[t]);
            }

            score /= test_inputs.len() as f32;
            let old_M = M;
            M += (score - M) / (j as f32 + 1.0);
            S += (score - M) * (score - old_M);

            trace_h1.push((props[0] as usize, score));
            trace_h2.push((props[1], score));
            trace_h3.push((props[2] as usize, score)); 
            trace_h4.push((props[3], score));
            trace_h5.push((props[4] as usize, score));
            trace_h6.push((props[5], score));
            
            objective += score;
        }

        S = S / (num_mutations as f32 - 1.0);
        S = S.sqrt();

        for j in 0..num_mutations
        {
            let score = trace_h1[j].1;
            trace_h1[j].1 = (score - M) / S;

            let score = trace_h2[j].1;
            trace_h2[j].1 = (score - M) / S;

            let score = trace_h3[j].1;
            trace_h3[j].1 = (score - M) / S;

            let score = trace_h4[j].1;
            trace_h4[j].1 = (score - M) / S;

            let score = trace_h5[j].1;
            trace_h5[j].1 = (score - M) / S;

            let score = trace_h6[j].1;
            trace_h6[j].1 = (score - M) / S;
        }

        objective /= num_mutations as f32;
        logger.push((i as f32, objective));
        //println!("Iteration: {}, Loss: {}", i + 1, objective);

        let grad_h1 = holes.par_1.grad(trace_h1);
        let grad_h2 = holes.par_2.grad(trace_h2);
        let grad_h3 = holes.par_3.grad(trace_h3);
        let grad_h4 = holes.par_4.grad(trace_h4);
        let grad_h5 = holes.par_5.grad(trace_h5);
        let grad_h6 = holes.par_6.grad(trace_h6);

        holes.par_1.update(grad_h1, rate);
        holes.par_2.update(grad_h2, rate);
        holes.par_3.update(grad_h3, rate);
        holes.par_4.update(grad_h4, rate);
        holes.par_5.update(grad_h5, rate);
        holes.par_6.update(grad_h6, rate);
    }

    chart.draw_series(LineSeries::new(logger, &BLUE))
        .unwrap()
        .label("NES")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    
    let mut synth_outputs = Vec::new();
    
    let prop_1 = holes.par_1.argmax() as f32;
    let prop_2 = holes.par_2.argmax() as f32;
    let prop_3 = holes.par_3.argmax() as f32;
    let prop_4 = holes.par_4.argmax() as f32;
    let prop_5 = holes.par_5.argmax() as f32;
    let prop_6 = holes.par_6.argmax() as f32;
            
    let props = vec![prop_1, prop_2, prop_3, prop_4, prop_5, prop_6];

    for t in test_inputs.clone()
    {
        synth_outputs.push(synth_prog(t, &props));
    }

    println!("===== Natural Evolution Strategies =====");
    println!("Ground truth outputs: {:?}", test_outputs);
    println!("Induction outputs: {:?}", synth_outputs);

    let op_1 = hole_1_str(prop_1);
    let op_2 = hole_2_str(prop_3);
    let op_3 = hole_2_str(prop_5);

    let prog = format!(r#"
    fn synth_prog(x: f32) -> f32
    {{
        if x {op_1} {prop_2}
        {{
            return {prop_4} {op_2} x;
        }}
        
        return x {op_3} {prop_6};
    }}"#);

    let gt = r#"
    fn ground_truth_prog(x: f32) -> f32
    {
        if x > 3.5
        {
            return 4.2 * x;
        }

        return x * 2.1;
    }"#;
    
    println!("{}", prog);
    println!("{}", gt);

    let par_1 = dist::Categorical::new(true, vec![0.0; 3]);
    let par_2 = dist::Normal::new(0.0, 1.0);
    let par_3 = dist::Categorical::new(true, vec![0.0; 4]);
    let par_4 = dist::Normal::new(0.0, 1.0);
    let par_5 = dist::Categorical::new(true, vec![0.0; 4]);
    let par_6 = dist::Normal::new(0.0, 1.0);
    
    let mut holes = Holes{par_1, par_2, par_3, par_4, par_5, par_6};
    let mut logger = Vec::new();

    for i in 0..num_iters
    {
        let mut trace_h1 = Vec::new();
        let mut trace_h2 = Vec::new();
        let mut trace_h3 = Vec::new();
        let mut trace_h4 = Vec::new();
        let mut trace_h5 = Vec::new();
        let mut trace_h6 = Vec::new();
        
        let mut objective = 0.0;
        let mut M = 0.0;
        let mut S = 0.0;

        for j in 0..num_mutations
        {
            let prop_1 = holes.par_1.sample(&mut rng) as f32;
            let prop_2 = holes.par_2.sample(&mut rng) as f32;
            let prop_3 = holes.par_3.sample(&mut rng) as f32;
            let prop_4 = holes.par_4.sample(&mut rng) as f32;
            let prop_5 = holes.par_5.sample(&mut rng) as f32;
            let prop_6 = holes.par_6.sample(&mut rng) as f32;
            
            let props = vec![prop_1, prop_2, prop_3, prop_4, prop_5, prop_6];
        
            let mut score = 0.0;
            for t in 0..test_inputs.len()
            {
                score += square(synth_prog(test_inputs[t], &props) - test_outputs[t]);
            }

            score /= test_inputs.len() as f32;
            let old_M = M;
            M += (score - M) / (j as f32 + 1.0);
            S += (score - M) * (score - old_M);

            trace_h1.push((props[0] as usize, score));
            trace_h2.push((props[1], score));
            trace_h3.push((props[2] as usize, score)); 
            trace_h4.push((props[3], score));
            trace_h5.push((props[4] as usize, score));
            trace_h6.push((props[5], score));
            
            objective += score;
        }

        S = S / (num_mutations as f32 - 1.0);
        S = S.sqrt();

        for j in 0..num_mutations
        {
            let score = trace_h1[j].1;
            trace_h1[j].1 = (score - M) / S;

            let score = trace_h2[j].1;
            trace_h2[j].1 = (score - M) / S;

            let score = trace_h3[j].1;
            trace_h3[j].1 = (score - M) / S;

            let score = trace_h4[j].1;
            trace_h4[j].1 = (score - M) / S;

            let score = trace_h5[j].1;
            trace_h5[j].1 = (score - M) / S;

            let score = trace_h6[j].1;
            trace_h6[j].1 = (score - M) / S;
        }

        objective /= num_mutations as f32;
        logger.push((i as f32, objective));
        //println!("Iteration: {}, Loss: {}", i + 1, objective);

        let grad_h1 = holes.par_1.grad(trace_h1);
        let grad_h2 = holes.par_2.grad(trace_h2);
        let grad_h3 = holes.par_3.grad(trace_h3);
        let grad_h4 = holes.par_4.grad(trace_h4);
        let grad_h5 = holes.par_5.grad(trace_h5);
        let grad_h6 = holes.par_6.grad(trace_h6);

        holes.par_1.update(grad_h1, rate);
        holes.par_2.update(grad_h2, rate);
        holes.par_3.update(grad_h3, rate);
        holes.par_4.update(grad_h4, rate);
        holes.par_5.update(grad_h5, rate);
        holes.par_6.update(grad_h6, rate);
    }

    chart.draw_series(LineSeries::new(logger, &RED))
        .unwrap()
        .label("VO")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    
    let mut synth_outputs = Vec::new();
    
    let prop_1 = holes.par_1.argmax() as f32;
    let prop_2 = holes.par_2.argmax() as f32;
    let prop_3 = holes.par_3.argmax() as f32;
    let prop_4 = holes.par_4.argmax() as f32;
    let prop_5 = holes.par_5.argmax() as f32;
    let prop_6 = holes.par_6.argmax() as f32;
            
    let props = vec![prop_1, prop_2, prop_3, prop_4, prop_5, prop_6];

    for t in test_inputs.clone()
    {
        synth_outputs.push(synth_prog(t, &props));
    }

    println!("");
    println!("======= Variational Optimation =========");
    println!("Ground truth outputs: {:?}", test_outputs);
    println!("Induction outputs: {:?}", synth_outputs);

    let op_1 = hole_1_str(prop_1);
    let op_2 = hole_2_str(prop_3);
    let op_3 = hole_2_str(prop_5);

    let prog = format!(r#"
    fn synth_prog(x: f32) -> f32
    {{
        if x {op_1} {prop_2}
        {{
            return {prop_4} {op_2} x;
        }}
        
        return x {op_3} {prop_6};
    }}"#);

    let gt = r#"
    fn ground_truth_prog(x: f32) -> f32
    {
        if x > 3.5
        {
            return 4.2 * x;
        }

        return x * 2.1;
    }"#;
    
    println!("{}", prog);
    println!("{}", gt);

    chart.configure_series_labels()
        .background_style(&WHITE)
        .border_style(&BLACK)
        .legend_area_size(50)
        .label_font(("sans-serif", 20).into_font())
        .draw()
        .unwrap();
    
    root.present().unwrap();
}