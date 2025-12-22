//! 本地调试用 CLI（不依赖图形库）。
//!
//! 示例：
//! - `cargo run -p kairm_engine --bin debug -- --seed 123`
//! - `cargo run -p kairm_engine --bin debug -- --date 2025-12-18`

use std::env;

use kairm_engine::{
    date_to_seed_ymd, difficulty_report_native, generate_puzzle_grid, validate_state_native, Color,
};

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let mut seed: Option<u64> = None;

    while !args.is_empty() {
        let k = args.remove(0);
        match k.as_str() {
            "--seed" => {
                let v = args.remove(0);
                seed = Some(v.parse::<u64>().expect("seed 必须是 u64"));
            }
            "--date" => {
                let v = args.remove(0);
                seed = Some(date_to_seed_ymd(&v).expect("date 解析失败"));
            }
            _ => {
                eprintln!("未知参数：{k}");
                eprintln!("用法：--seed <u64> 或 --date <YYYY-MM-DD>");
                std::process::exit(2);
            }
        }
    }

    let seed = seed.unwrap_or(0);
    let grid = generate_puzzle_grid(seed, 5).expect("生成题目失败");

    let flat: Vec<u8> = grid.iter().flat_map(|r| r.iter().copied()).collect();
    let mut black_mask = 0u64;
    for (i, &c) in flat.iter().enumerate() {
        if c == Color::Black.to_u8() {
            black_mask |= 1u64 << i;
        }
    }

    let res = validate_state_native(black_mask, &flat).expect("校验失败");

    let diff = difficulty_report_native(&flat).expect("difficulty_report failed");

    println!("seed={seed}");
    println!(
        "grid(5x5,u8)={}",
        serde_json::to_string_pretty(&grid).unwrap()
    );
    println!("initial_black_mask={black_mask}");
    println!(
        "validate(initial)={}",
        serde_json::to_string_pretty(&res).unwrap()
    );
    println!(
        "difficulty_report={}",
        serde_json::to_string_pretty(&diff).unwrap()
    );
}
