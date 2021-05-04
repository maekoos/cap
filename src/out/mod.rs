use super::eval;
mod mc;
mod scad;

pub fn process_out_scad(input: &Vec<eval::EvaluatedValue>) -> String {
    let mut scad_out = Vec::new();

    for a in input.iter().filter(|x| match x {
        eval::EvaluatedValue::Native(_) => true,
        _ => false,
    }) {
        scad_out.push(format!("// {:?}", a));
        scad_out.push(scad::process_native(match a {
            eval::EvaluatedValue::Native(n) => n,
            _ => unreachable!(),
        }));
    }

    scad_out.join("\n")
}

pub fn process_out_mc(
    input: &Vec<eval::EvaluatedValue>,
    scale: i32,
    origin: Option<(isize, isize, isize)>,
) -> (String, String) {
    let mut mc_out = Vec::new();

    for a in input.iter().filter(|x| match x {
        eval::EvaluatedValue::Native(_) => true,
        _ => false,
    }) {
        mc::process_native(
            match a {
                eval::EvaluatedValue::Native(n) => n,
                _ => unreachable!(),
            },
            scale,
            &mut mc_out,
        );
    }

    (
        mc::blocks_to_cmds(&mc_out, origin).join("\n"),
        mc::blocks_to_destroys(&mc_out, origin).join("\n"),
    )
}
