use crate::{state, Expr, Expression, Object, Result, TypeDef, Value};

#[derive(Debug, Clone)]
pub struct Block {
    expressions: Vec<Expr>,
}

impl Block {
    pub fn new(expressions: Vec<Expr>) -> Self {
        Self { expressions }
    }
}

impl Expression for Block {
    fn execute(&self, state: &mut state::Program, object: &mut dyn Object) -> Result<Value> {
        self.expressions
            .iter()
            .map(|expr| expr.execute(state, object))
            .last()
            .unwrap_or(Ok(Value::Null))
    }

    fn type_def(&self, state: &state::Compiler) -> TypeDef {
        let mut type_defs = self
            .expressions
            .iter()
            .map(|e| e.type_def(state))
            .collect::<Vec<_>>();

        // If any of the stored expressions is fallible, the entire block is
        // fallible.
        let fallible = type_defs.iter().any(TypeDef::is_fallible);

        // The last expression determines the resulting value of the block.
        let mut type_def = type_defs.pop().unwrap_or(TypeDef {
            optional: true,
            ..Default::default()
        });

        type_def.fallible = fallible;
        type_def
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        expression::{Arithmetic, Literal},
        test_type_def,
        value::Kind,
        Operator,
    };

    test_type_def![
        no_expression {
            expr: |_| Block::new(vec![]),
            def: TypeDef { optional: true, ..Default::default() },
        }

        one_expression {
            expr: |_| Block::new(vec![Literal::from(true).into()]),
            def: TypeDef { kind: Kind::Boolean, ..Default::default() },
        }

        multiple_expressions {
            expr: |_| Block::new(vec![
                        Literal::from("foo").into(),
                        Literal::from(true).into(),
                        Literal::from(1234).into(),
            ]),
            def: TypeDef { kind: Kind::Integer, ..Default::default() },
        }

        last_one_fallible {
            expr: |_| Block::new(vec![
                        Literal::from(true).into(),
                        Arithmetic::new(
                          Box::new(Literal::from(12).into()),
                          Box::new(Literal::from(true).into()),
                          Operator::Multiply,
                        ).into(),
            ]),
            def: TypeDef {
                fallible: true,
                kind: Kind::String | Kind::Integer | Kind::Float,
                ..Default::default()
            },
        }

        any_fallible {
            expr: |_| Block::new(vec![
                        Literal::from(true).into(),
                        Arithmetic::new(
                          Box::new(Literal::from(12).into()),
                          Box::new(Literal::from(true).into()),
                          Operator::Multiply,
                        ).into(),
                        Literal::from(vec![1]).into(),
            ]),
            def: TypeDef {
                fallible: true,
                kind: Kind::Array,
                ..Default::default()
            },
        }
    ];
}
