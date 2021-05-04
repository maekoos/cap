use std::collections::HashMap;
use std::fmt;

use super::parse;

mod util;

#[derive(Debug)]
pub enum EvalError {
  UnexpectedExpression(parse::Value),
}

//TODO Add optional description/documentation string
#[derive(Clone)]
enum EvalFunction {
  LispFunction(LispFunction),
  NativeFunction(fn(&mut EvalEnvironment, &[parse::Value]) -> Result<EvaluatedValue, EvalError>),
}

impl fmt::Debug for EvalFunction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      EvalFunction::LispFunction(lf) => write!(f, "LispFunction({:?})", lf),
      EvalFunction::NativeFunction(_) => write!(f, "NativeFunction"),
    }
  }
}

#[derive(Debug, Clone)]
struct LispFunction {
  arguments: Vec<(String, Option<parse::Value>)>,
  body: Vec<parse::Value>,
}

impl LispFunction {
  pub fn new(arguments: Vec<(String, Option<parse::Value>)>, body: Vec<parse::Value>) -> Self {
    Self { arguments, body }
  }
}

#[derive(Debug, Clone)]
struct EvalEnvironment {
  functions: HashMap<String, EvalFunction>,
  variables: HashMap<String, EvaluatedValue>,
  locals: Vec<HashMap<String, EvaluatedValue>>,
}

impl Default for EvalEnvironment {
  fn default() -> Self {
    let mut env = Self {
      functions: HashMap::new(),
      variables: HashMap::new(),
      locals: Vec::new(),
    };

    env.set_variable("nil".to_owned(), EvaluatedValue::None);
    env.set_variable("t".to_owned(), EvaluatedValue::True);

    env.functions.insert(
      "print".to_owned(),
      EvalFunction::NativeFunction(|env, args| {
        let mut out = Vec::new();
        for a in args {
          out.push(format!("{:?}", evaluate_expression(a, env)?));
        }

        println!("{}", out.join(" "));

        Ok(EvaluatedValue::None)
      }),
    );

    env.functions.insert(
      "+".to_owned(),
      EvalFunction::NativeFunction(|env, args| match args.get(0) {
        None => unimplemented!("+ needs some arguments"),
        Some(v) => match evaluate_expression(v, env)? {
          EvaluatedValue::Vector(_vector) => {
            panic!("+ function does not support vectors. Consider v+ and vv+");
          }
          v => {
            let mut sum = 0.0;
            sum += util::cast_number(&v);

            for a in &args[1..] {
              let a = evaluate_expression(a, env)?;
              sum += util::cast_number(&a);
            }

            Ok(EvaluatedValue::Number(sum))
          }
        },
      }),
    );

    env.functions.insert(
      "-".to_owned(),
      EvalFunction::NativeFunction(|env, args| match args.get(0) {
        None => unimplemented!("- needs some arguments"),
        Some(v) => match evaluate_expression(v, env)? {
          EvaluatedValue::Vector(_vector) => {
            panic!("- function does not support vectors. Consider v- and vv-");
          }
          v => {
            //TODO Rename sum :P
            let mut sum = 0.0;
            sum += util::cast_number(&v);

            for a in &args[1..] {
              let a = evaluate_expression(a, env)?;
              sum -= util::cast_number(&a);
            }

            if args.len() == 1 {
              sum = -sum;
            }

            Ok(EvaluatedValue::Number(sum))
          }
        },
      }),
    );

    env.functions.insert(
      "?".to_owned(),
      EvalFunction::NativeFunction(|env, args| {
        if args.len() != 2 && args.len() > 3 {
          unimplemented!("Please, pass a condition, a handler and maybe an else. No more for now.");
        }

        let condition = match args.first() {
          None => unreachable!(),
          Some(v) => match evaluate_expression(v, env)? {
            EvaluatedValue::None => false,
            EvaluatedValue::True => true,
            EvaluatedValue::Number(n) => n != 0.0,
            EvaluatedValue::Vector(_) => true,
            EvaluatedValue::Native(_) => {
              unimplemented!("native value (like polygon or extrude)")
            }
          },
        };

        Ok(if condition {
          match args.get(1) {
            None => unreachable!("No if-handler"),
            Some(v) => evaluate_expression(v, env)?,
          }
        } else {
          match args.get(2) {
            None => EvaluatedValue::None,
            Some(v) => evaluate_expression(v, env)?,
          }
        })
      }),
    );

    env.functions.insert(
      "==".to_owned(),
      EvalFunction::NativeFunction(|env, args| {
        if args.len() != 2 {
          unimplemented!("Please, pass a 2 values to the '=='-function. No more for now.");
        }

        let is_true = |a| match a {
          EvaluatedValue::None => false,
          EvaluatedValue::True => true,
          EvaluatedValue::Number(n) => n != 0.0,
          EvaluatedValue::Vector(v) => v.len() != 0,
          EvaluatedValue::Native(_) => {
            unimplemented!("native value (like polygon or extrude)")
          }
        };

        let res = match args.first() {
          None => unreachable!(),
          Some(v) => {
            let arg2 = match args.get(1) {
              None => unreachable!(),
              Some(v) => evaluate_expression(v, env)?,
            };
            match evaluate_expression(v, env)? {
              EvaluatedValue::None => !is_true(arg2),
              EvaluatedValue::True => is_true(arg2),
              EvaluatedValue::Number(n) => match arg2 {
                EvaluatedValue::None => n == 0.0,
                EvaluatedValue::True => n != 0.0,
                EvaluatedValue::Number(n1) => n == n1,
                EvaluatedValue::Vector(v) => v.len() == 0 && n == 0.0,
                EvaluatedValue::Native(_) => {
                  unimplemented!("native value (like polygon or extrude)")
                }
              },
              EvaluatedValue::Vector(v1) => match arg2 {
                EvaluatedValue::None => v1.len() == 0,
                EvaluatedValue::True => v1.len() != 0,
                EvaluatedValue::Number(n) => n == 0.0 && v1.len() == 0,
                EvaluatedValue::Vector(v2) => v1 == v2,
                EvaluatedValue::Native(_) => {
                  unimplemented!("native value (like polygon or extrude)")
                }
              },
              EvaluatedValue::Native(_) => {
                unimplemented!("native value (like polygon or extrude)")
              }
            }
          }
        };

        Ok(if res {
          EvaluatedValue::Number(1.0)
        } else {
          EvaluatedValue::None
        })
      }),
    );

    env.functions.insert(
      "nth".to_owned(),
      EvalFunction::NativeFunction(|env, args| {
        if args.len() != 2 {
          panic!("nth function needs two arguments. First is index, second is the vector.");
        }

        Ok(match args.first() {
          None => unreachable!(),
          Some(v) => match evaluate_expression(v, env)? {
            EvaluatedValue::Number(n) => {
              let idx = n.floor() as usize;

              match args.get(1) {
                None => unreachable!(),
                Some(v) => match evaluate_expression(v, env)? {
                  EvaluatedValue::Vector(v) => match v.get(idx) {
                    None => EvaluatedValue::None,
                    Some(v) => v.clone(),
                  },
                  _ => todo!(),
                },
              }
            }
            _ => todo!("'nth' index must be a number"),
          },
        })
      }),
    );

    env.set_variable(
      "CURRENT_POSITION".to_owned(),
      EvaluatedValue::Vector(vec![
        EvaluatedValue::Number(0.0),
        EvaluatedValue::Number(0.0),
        EvaluatedValue::Number(0.0),
      ]),
    );
    env.functions.insert(
      "polygon".to_owned(),
      EvalFunction::NativeFunction(|env, args| {
        if args.len() != 1 && args.len() != 2 {
          unimplemented!("make-polygon: Please, pass a list of points and maybe a list of the order of these points. No more for now.");
        }

        let cur_pos = match env.get_variable("CURRENT_POSITION") {
          Some(EvaluatedValue::Vector(v)) => {
            if !v.len() == 3 {
              unimplemented!("CURRENT_POSITION has to be a 3D-vector. Got a {}D-vector", v.len())
            }

            let mut out = Vec::new();
            for item in v {
              match item {
                EvaluatedValue::Number(n)=>out.push(*n),
                _ => unimplemented!("CURRENT_POSITION has to be a 3D-vector of numbers. Got {:?}", v),
              }
            }

            (out[0], out[1], out[2])
          },
          v => unimplemented!("CURRENT_POSITION has to be a 3D-vector. Got {:?}", v),
        };

        let order = match args.get(1) {
            None => None,
            Some(v) => match evaluate_expression(v, env)? {
              EvaluatedValue::Vector(values) => {
                //TODO Make sure all of the values are floats
                let mut order = Vec::new();
                for v in values {
                  order.push(match v {
                    EvaluatedValue::Number(n) => n.floor() as usize,
                    _ => unimplemented!("Expected numbers in order (order of polygons)"),
                  });
                }

                Some(order)
              }
              _ => unimplemented!("Expected a vector (order of polygon)"),
            }
          };

        let points = match args.get(0) {
          None => unimplemented!("Polygon needs its points"),
          Some(v) => match evaluate_expression(v, env)? {
            EvaluatedValue::Vector(values) => {
              if values.len() < 3 {
                panic!("Polygon needs at least three values");
              }

              let mut points: Vec<(f64, f64)> = Vec::new();
              for v in values {
                points.push(match v {
                  EvaluatedValue::Vector(values) => {
                    let x: f64 = match values.get(0) {
                      None => unimplemented!("A point has two values (in a polygon). Got none."),
                      Some(val) => match val {
                        EvaluatedValue::Number(val) => *val,
                        _ => unimplemented!("A point is a vector of NUMBERS (got {:?})", values),
                      }
                    };
                    let y: f64 = match values.get(1) {
                      None => unimplemented!("A point has two values (in a polygon). Got one."),
                      Some(v) => match v {
                        EvaluatedValue::Number(v) => *v,
                        _ => unimplemented!("A point is a vector of NUMBERS"),
                      }
                    };

                    (x, y)
                  }
                  _ => unimplemented!("Points are two or three dimensional vectors (make-polygon)"),
                });
              }

              points
            }
            _ => unimplemented!("polygon needs a vector as points"),
          }
        };


        Ok(EvaluatedValue::Native(NativeValue::Polygon(Polygon (points, order, cur_pos))))
      }),
    );

    env.functions.insert(
      "extrude".to_owned(),
      EvalFunction::NativeFunction(|env, args| {
        if args.len() != 2 {
          unimplemented!("extrude: Please, pass a height and a polygon. No more for now.");
        }

        match args.get(0) {
          None => unimplemented!(),
          Some(v) => match evaluate_expression(v, env)? {
            EvaluatedValue::Number(h) => match args.get(1) {
              None => unreachable!(),
              Some(v) => match evaluate_expression(v, env)? {
                EvaluatedValue::Native(nv) => match nv {
                  NativeValue::Polygon(p) => Ok(EvaluatedValue::Native(NativeValue::Extrude(p, h))),
                  NativeValue::Extrude(_, _) => {
                    unimplemented!("Can not extrude a NativeValue(extrude)")
                  }
                },
                _ => unimplemented!("extrude: Expected a native polygon"),
              },
            },
            _ => unimplemented!("extrude: Expected a number (height)"),
          },
        }
      }),
    );

    env
  }
}

impl EvalEnvironment {
  pub fn set_variable(&mut self, name: String, value: EvaluatedValue) {
    self.variables.insert(name, value);
  }

  pub fn set_function(&mut self, name: String, function: EvalFunction) {
    self.functions.insert(name, function);
  }

  pub fn get_variable(&self, name: &str) -> Option<&EvaluatedValue> {
    for c in self.locals.iter().rev() {
      match c.get(name) {
        None => continue,
        Some(v) => return Some(v),
      }
    }

    self.variables.get(name)
  }

  pub fn set_local(&mut self, name: String, value: EvaluatedValue) {
    match self.locals.last_mut() {
      None => todo!(),
      Some(v) => {
        v.insert(name, value);
      }
    }
  }

  pub fn push_defs(&mut self) {
    self.locals.push(HashMap::new());
  }
  pub fn pop_defs(&mut self) {
    self.locals.pop();
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvaluatedValue {
  Number(f64),
  Vector(Vec<EvaluatedValue>),
  None,
  True,
  Native(NativeValue),
}

#[derive(Clone, Debug, PartialEq)]
pub enum NativeValue {
  Polygon(Polygon),
  Extrude(Polygon, f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Polygon(
  pub Vec<(f64, f64)>,
  pub Option<Vec<usize>>,
  pub (f64, f64, f64),
);

pub fn evaluate(parsed: Vec<parse::Value>) -> Result<Vec<EvaluatedValue>, EvalError> {
  let mut env = EvalEnvironment::default();

  let mut out = Vec::new();
  for expr in &parsed {
    out.push(evaluate_expression(expr, &mut env)?);
  }

  Ok(out)
}

fn evaluate_expression(
  expression: &parse::Value,
  env: &mut EvalEnvironment,
) -> Result<EvaluatedValue, EvalError> {
  Ok(match expression {
    parse::Value::SExpression(sexpr) => {
      if sexpr.is_first(&parse::Value::Identifier("set".to_owned())) == Some(true) {
        let body = sexpr.body();
        if body.len() != 3 {
          unimplemented!();
        }

        match body.get(1) {
          None => unreachable!(),
          Some(parse::Value::Identifier(name)) => {
            let value = match body.get(2) {
              None => unreachable!(),
              Some(v) => evaluate_expression(v, env)?,
            };

            env.set_variable(name.clone(), value);
          }
          Some(e) => return Err(EvalError::UnexpectedExpression(e.to_owned())),
        }

        EvaluatedValue::None
      } else if sexpr.is_first(&parse::Value::Identifier("fn".to_owned())) == Some(true) {
        let body = sexpr.body();

        match body.get(1) {
          None => unimplemented!(),
          Some(parse::Value::Identifier(name)) => {
            let args = match body.get(2) {
              Some(parse::Value::SExpression(args_expr)) => {
                let args = args_expr.body();
                let mut out = Vec::new();

                for a in args {
                  match a {
                    parse::Value::Identifier(name) => out.push((name.to_owned(), None)),
                    parse::Value::SExpression(sexp) => {
                      let body = sexp.body();
                      if body.len() != 2 {
                        unimplemented!();
                      }

                      match body.get(0) {
                        Some(parse::Value::Identifier(name)) => match body.get(1) {
                          None => unreachable!(),
                          Some(v) => out.push((name.to_owned(), Some(v.clone()))),
                        },
                        None => unreachable!(),
                        Some(_) => unimplemented!(),
                      }
                    }
                    _ => unimplemented!(),
                  }
                }

                out
              }
              None => unimplemented!(),
              Some(_) => unimplemented!(),
            };

            let f_body: Vec<parse::Value> = (&body[3..]).iter().map(|x| x.clone()).collect();

            let f = LispFunction::new(args, f_body);
            env.set_function(name.clone(), EvalFunction::LispFunction(f));
          }
          Some(_) => unimplemented!(),
        }

        EvaluatedValue::None
      } else {
        match sexpr.first() {
          // Empty SExpression
          None => todo!("Empty sexpr, treat as array (or nil?)"),
          // SExpression where the first item is an identifier (may be a variable or function)
          Some(parse::Value::Identifier(id)) => {
            match env.functions.get(id) {
              // SExpression where the first item is a function
              Some(f) => {
                //Do we really need to clone here?
                let f = f.clone();

                let body = sexpr.body();
                call_function(env, &f, &body[1..])?
              }
              // SExpression where the first item is an identifier but not a function
              None => {
                let mut out = Vec::new();
                for itm in sexpr.body() {
                  out.push(evaluate_expression(itm, env)?);
                }
                EvaluatedValue::Vector(out)
              }
            }
          }
          // SExpression where the first item is not an identifier
          Some(_) => {
            let mut out = Vec::new();
            for itm in sexpr.body() {
              out.push(evaluate_expression(itm, env)?);
            }
            EvaluatedValue::Vector(out)
          }
        }
      }
    }
    parse::Value::Identifier(name) => match env.get_variable(name) {
      None => {
        panic!("Unknown variable: {:?}", name);
      }
      Some(value) => value.clone(),
    },
    parse::Value::Number(n) => EvaluatedValue::Number(*n),
  })
}

fn call_function(
  env: &mut EvalEnvironment,
  f: &EvalFunction,
  args: &[parse::Value],
) -> Result<EvaluatedValue, EvalError> {
  env.push_defs();

  let rv = match f {
    EvalFunction::LispFunction(lf) => {
      for (i, arg) in lf.arguments.iter().enumerate() {
        let value = match args.get(i) {
          None => match &arg.1 {
            None => {
              println!("Did not pass a value to an argument with no default value (Kinda not supposed to happen?) @ {:?}", f);
              EvaluatedValue::None
            }
            Some(v) => evaluate_expression(&v, env)?,
          },
          Some(v) => evaluate_expression(v, env)?,
        };
        env.set_local(arg.0.to_owned(), value)
      }

      let mut out = Vec::new();
      for expr in &lf.body {
        let res = evaluate_expression(expr, env)?;
        out.push(res);
      }

      //TODO Aviod cloning
      out.last().unwrap_or(&EvaluatedValue::None).to_owned()
    }
    EvalFunction::NativeFunction(nf) => nf(env, args)?,
  };

  env.pop_defs();

  Ok(rv)
}
