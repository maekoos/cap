use conv::prelude::*;

use super::eval;

#[derive(Clone)]
pub struct Block {
    pub pos: (isize, isize, isize),
    pub texture: Option<String>,
}

impl Block {
    fn translate(&mut self, x: isize, y: isize, z: isize) {
        self.pos.0 += x;
        self.pos.1 += y;
        self.pos.2 += z;
    }

    fn translated(&self, x: isize, y: isize, z: isize) -> Self {
        let mut a = self.clone();
        a.translate(x, y, z);
        a
    }
}

fn apply_scale(v: f64, scale: i32) -> isize {
    (v * f64::from(scale)).approx().unwrap()
}

pub fn process_native(native: &eval::NativeValue, scale: i32, out: &mut Vec<Block>) {
    let polygon_2_blocks = |p: &eval::Polygon, scale: i32| {
        // - Make a square that fits the whole of the polygon (max and min x and y)
        // - Loop through each and every one of these blocks and see which ones we should "keep" using
        //      this Ray Casting Algorithm from wikipedia: https://en.wikipedia.org/wiki/Point_in_polygon#/media/File:RecursiveEvenPolygon.svg

        let mut out = Vec::new();

        let xmin = p.0.iter().map(|x| apply_scale(x.0, scale)).min().unwrap();
        let xmax = p.0.iter().map(|x| apply_scale(x.0, scale)).max().unwrap();

        let ymin = p.0.iter().map(|x| apply_scale(x.1, scale)).min().unwrap();
        let ymax = p.0.iter().map(|x| apply_scale(x.1, scale)).max().unwrap();

        for y in (ymin - 1)..(ymax + 1) {
            // Find coordinates for lines we intersect in this polygon
            let intersections = {
                let mut out = Vec::new();

                let mut p_idx = 0;
                while p_idx < p.0.len() {
                    let (p1, p2) = if p_idx == 0 {
                        let p1 = (p.0)[p.0.len() - 1];
                        let p2 = (p.0)[p_idx];
                        (
                            (apply_scale(p1.0, scale), apply_scale(p1.1, scale)),
                            (apply_scale(p2.0, scale), apply_scale(p2.1, scale)),
                        )
                    } else {
                        let p1 = (p.0)[p_idx - 1];
                        let p2 = (p.0)[p_idx];
                        (
                            (apply_scale(p1.0, scale), apply_scale(p1.1, scale)),
                            (apply_scale(p2.0, scale), apply_scale(p2.1, scale)),
                        )
                    };

                    // Make sure this line is relevant
                    if (p2.1 <= y && y < p1.1) || (p1.1 <= y && y < p2.1) {
                        let dx = p1.0 - p2.0;
                        let _dy = p1.1 - p2.1;

                        let ip = if dx == 0 {
                            p1.0
                        } else {
                            todo!("Lines that are not parallel with the x or y axis are not supported yet.");
                            // let k = dy / dx;
                            // let m = p1.1 - k * p1.0;
                            // let ip = if k == 0 { y - m } else { (y - m) / k };
                            // println!("k={} m={} (dy={} ; dx = {})", k, m, dy, dx);
                            // ip
                        };

                        out.push(ip);
                    }

                    p_idx += 1;
                }

                //TODO Order this list to find the number of total intersections without looking through the whole list
                out
            };

            for x in (xmin - 1)..(xmax + 1) {
                let mut total_lines_intersected = 0;
                //TODO If intersections were ordered we would be able to break on ip > x
                for ip in &intersections {
                    if *ip <= x {
                        total_lines_intersected += 1;
                    }
                }

                if total_lines_intersected % 2 == 1 {
                    out.push(Block {
                        pos: (
                            x + apply_scale((p.2).0, scale),
                            y + apply_scale((p.2).1, scale),
                            0,
                        ),
                        texture: None,
                    })
                }
            }
        }

        out
    };

    match native {
        eval::NativeValue::Polygon(p) => out.extend(polygon_2_blocks(p, scale)),
        eval::NativeValue::Extrude(p, h) => {
            let polygon = polygon_2_blocks(p, scale);

            let height: isize = (h * f64::from(scale)).approx().unwrap();
            for i in 0..height {
                out.extend(
                    polygon
                        .iter()
                        .map(|b| b.translated(0, 0, i + apply_scale((p.2).2, scale))),
                );
            }
        }
    }
}

pub fn blocks_to_cmds(blocks: &Vec<Block>, origin: Option<(isize, isize, isize)>) -> Vec<String> {
    let origin = if let Some(o) = origin { o } else { (0, 0, 0) };

    println!("Blocks: {}", blocks.len());
    blocks
        .iter()
        .map(|b| {
            format!(
                "/setblock {} {} {} {} replace",
                b.pos.0 + origin.0,
                b.pos.2 + origin.2,
                b.pos.1 + origin.1,
                if b.texture == None {
                    "birch_planks"
                } else {
                    todo!("Add texture")
                }
            )
        })
        .collect()
}

pub fn blocks_to_destroys(
    blocks: &Vec<Block>,
    origin: Option<(isize, isize, isize)>,
) -> Vec<String> {
    let origin = if let Some(o) = origin { o } else { (0, 0, 0) };

    println!("Blocks (destroy): {}", blocks.len());
    blocks
        .iter()
        .map(|b| {
            format!(
                "/setblock {} {} {} air replace",
                b.pos.0 + origin.0,
                b.pos.2 + origin.2,
                b.pos.1 + origin.1
            )
        })
        .collect()
}
