use super::eval;

pub fn process_native(native: &eval::NativeValue) -> String {
    let process_polygon = |p: &eval::Polygon| {
        format!(
            "translate([{},{},{}]) polygon(points=[{}]);",
            (p.2).0,
            (p.2).1,
            (p.2).2,
            p.0.iter()
                .map(|x| format!("[{}, {}]", x.0, x.1))
                .collect::<Vec<String>>()
                .join(", ")
        )
    };

    match native {
        eval::NativeValue::Polygon(p) => process_polygon(p),
        eval::NativeValue::Extrude(p, h) => {
            format!(
                "translate([0, 0, {translation_z}]) linear_extrude(height = {height}, center = false, twist = 0, scale = 1.0) {body}",
                translation_z = (p.2).2,
                height = h,
                body = process_polygon(p)
            )
        }
    }
}
