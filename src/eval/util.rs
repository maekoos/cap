use super::*;

pub fn cast_number(a: &EvaluatedValue) -> f64 {
    match a {
        EvaluatedValue::None => 0.0,
        EvaluatedValue::True => 1.0,
        EvaluatedValue::Number(v) => *v,
        EvaluatedValue::Vector(_) => todo!("Vector can not be cast into a number (yet)"),
        EvaluatedValue::Native(_) => {
            unimplemented!("A native value (like polygon or extrude) can not be cast into a number")
        }
    }
}
