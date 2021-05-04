use std::env;

mod eval;
mod lex;
mod out;
mod parse;

//TODO Get these from arguments
static SCALE: i32 = 10;
static ORIGIN: (isize, isize, isize) = (0, 0, 4);

fn main() {
    let args: Vec<String> = env::args().collect();
    //TODO Move this to `parse_config()`: https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html
    let input_file_path = args.get(1).expect("No input file.");

    let lexed = lex::lex_file(input_file_path).unwrap();

    let parsed = parse::parse(lexed).unwrap();

    let out = eval::evaluate(parsed).unwrap();
    // println!("Out: {:?}", out);

    let scad_out = out::process_out_scad(&out);
    std::fs::write("out.scad", scad_out).unwrap();

    let mc_out = out::process_out_mc(&out, SCALE, Some(ORIGIN));
    std::fs::write("out.mccmd", mc_out.0).unwrap();
    std::fs::write("del.mccmd", mc_out.1).unwrap();
}
