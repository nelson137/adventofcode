fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = std::path::PathBuf::from(out_dir);
    let target_dir = out_dir.ancestors().nth(4).unwrap();
    let puzzle_inputs_dir = target_dir.join("puzzles/inputs");
    println!(
        "cargo::rustc-env=PUZZLE_INPUTS_DIR={}",
        puzzle_inputs_dir.display()
    );
}
