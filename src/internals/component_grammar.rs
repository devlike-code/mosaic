use pest::iterators::Pair;
use pest_derive::*;

use super::datatypes::{ComponentField, ComponentType, Datatype};
use crate::pest::Parser;

#[derive(Parser)]
#[grammar = "internals/component_grammar.pest"]
pub struct ComponentParser;

#[derive(Debug, PartialEq, Eq)]
enum ComponentTypeKindNames {
    Product,
    Sum,
    Alias,
}

impl ComponentParser {
    fn parse_base_type(v: &str) -> Option<Datatype> {
        match v {
            "void" => Some(Datatype::VOID),
            "i32" => Some(Datatype::I32),
            "i64" => Some(Datatype::I64),
            "u32" => Some(Datatype::U32),
            "u64" => Some(Datatype::U64),
            "f32" => Some(Datatype::F32),
            "f64" => Some(Datatype::F64),
            "id" => Some(Datatype::EID),
            "s32" => Some(Datatype::S32),
            "b256" => Some(Datatype::B256),
            _ => None,
        }
    }

    fn parse_field(pair: Pair<'_, Rule>) -> Result<ComponentField, String> {
        let mut subs = pair.into_inner();
        let mut val = subs.next().unwrap();
        let name = val.as_str().trim().into();

        val = subs.next().unwrap();
        match val.as_rule() {
            Rule::datatype_expr | Rule::field_datatype_expr => {
                let v = val.as_str();
                let typ = Self::parse_base_type(v);

                if let Some(t) = typ {
                    Ok(ComponentField { name, datatype: t })
                } else {
                    Ok(ComponentField {
                        name,
                        datatype: Datatype::COMP(v.into()),
                    })
                }
            }

            Rule::identifier => Ok(ComponentField {
                name,
                datatype: Datatype::COMP(val.as_str().trim().into()),
            }),

            e => {
                Err(format!("[Error][component_grammar.rs][parse_field] Expected datatype or identifier when parsing field '{:?}', {:?} found.", name, e))
            }
        }
    }

    fn check_keywords(name: &str) -> Result<(), String> {
        if name == "product" {
            Err("[Error][component_grammar.rs][check_keywords] Keyword 'product' can't be used as an identifier.".to_string())
        } else if name == "sum" {
            Err("[Error][component_grammar.rs][check_keywords] Keyword 'sum' can't be used as an identifier.".to_string())
        } else {
            Ok(())
        }
    }

    fn parse_product(pair: Pair<'_, Rule>) -> Result<ComponentType, String> {
        let mut pairs = pair.into_inner();
        let mut val = pairs.next().unwrap();
        let name = val.as_str().trim();
        val = pairs.next().unwrap();

        let kind = match val.as_rule() {
            Rule::product_type_expr => ComponentTypeKindNames::Product,
            Rule::sum_type_expr => ComponentTypeKindNames::Sum,
            Rule::datatype_expr => ComponentTypeKindNames::Alias,
            e => {
                return Err(format!("[Error][component_grammar.rs][parse_struct] Unexpected rule {:?} found where record, sum, or simple datatype expected.", e));
            }
        };

        return if kind == ComponentTypeKindNames::Alias {
            let v = val.as_str();
            Self::check_keywords(v)?;
            let typ = Self::parse_base_type(v);
            if let Some(t) = typ {
                Ok(ComponentType::Alias({
                    ComponentField {
                        name: name.into(),
                        datatype: t,
                    }
                }))
            } else {
                Ok(ComponentType::Alias({
                    ComponentField {
                        name: name.into(),
                        datatype: Datatype::COMP(v.into()),
                    }
                }))
            }
        } else {
            let subs = val.into_inner();
            let mut fields = vec![];

            for n in subs {
                let field = Self::parse_field(n.clone());
                if field.is_err() {
                    return Err(field.err().unwrap());
                }
                fields.push(field.unwrap());
            }

            if kind == ComponentTypeKindNames::Product {
                Ok(ComponentType::Product {
                    name: name.into(),
                    fields,
                })
            } else {
                Ok(ComponentType::Sum {
                    name: name.into(),
                    fields,
                })
            }
        };
    }

    pub fn parse_type<S: AsRef<str>>(s: S) -> Result<ComponentType, String> {
        match Self::parse(Rule::struct_expr, s.as_ref()) {
            Ok(pairs) => {
                let pair = pairs.into_iter().next().unwrap();
                match pair.as_rule() {
                    Rule::struct_expr => Self::parse_product(pair),
                    _ => Err(
                        "[Error][component_grammar.rs][parse_type] Wrong structure found!"
                            .to_string(),
                    ),
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn parse_types<S: AsRef<str>>(s: S) -> Vec<Result<ComponentType, String>> {
        match Self::parse(Rule::structures_expr, s.as_ref()) {
            Ok(pairs) => pairs
                .into_iter()
                .map(|pair| match pair.as_rule() {
                    Rule::struct_expr => {
                        let typ = Self::parse_product(pair).unwrap();
                        Ok(typ)
                    }

                    e => Err(format!(
                        "[Error][component_grammar.rs][parse_types] Wrong structure found: {:?}!",
                        e
                    )),
                })
                .collect(),

            Err(err) => vec![Err(err.to_string())],
        }
    }

    pub fn parse_all<S: AsRef<str>>(s: S) -> Result<Vec<ComponentType>, String> {
        let result = Self::parse_types(s);
        if result.iter().all(|x| x.is_ok()) {
            let result: Vec<ComponentType> = result.into_iter().map(|x| x.unwrap()).collect();
            Ok(result)
        } else {
            Err(result
                .into_iter()
                .filter(|x| x.is_err())
                .map(|x| x.err().unwrap())
                .collect::<Vec<String>>()
                .join(";"))
        }
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod component_grammar_testing {
    use crate::internals::datatypes::{ComponentField, ComponentType, Datatype};

    use super::ComponentParser;

    #[test]
    fn test_parse_basic_alias() {
        let input = "Float : f32;";
        let expected = ComponentType::Alias({
            ComponentField {
                name: "Float".into(),
                datatype: Datatype::F32,
            }
        });

        assert_eq!(Ok(expected), ComponentParser::parse_type(input));
    }

    #[test]
    fn test_parse_comp_alias() {
        let input = "Position : Point;";
        let expected = ComponentType::Alias({
            ComponentField {
                name: "Position".into(),
                datatype: Datatype::COMP("Point".into()),
            }
        });

        assert_eq!(Ok(expected), ComponentParser::parse_type(input));
    }

    #[test]
    fn test_parse_product_type() {
        let input = "Position : product { x: i32, y: i32 };";
        let expected = ComponentType::Product {
            name: "Position".into(),
            fields: vec![
                ComponentField {
                    name: "x".into(),
                    datatype: Datatype::I32,
                },
                ComponentField {
                    name: "y".into(),
                    datatype: Datatype::I32,
                },
            ],
        };

        assert_eq!(Ok(expected), ComponentParser::parse_type(input));
    }

    #[test]
    fn test_parse_product_type_with_comp_field() {
        let input = "Position : product { x: i32, y: Foo };";
        // println!("{}", ComponentParser::parse_type(input).unwrap_err());
        assert!(ComponentParser::parse_type(input).is_err());
    }

    #[test]
    fn test_parse_sum_type() {
        let input = "Position : sum { x: i32, y: i32 };";
        let expected = ComponentType::Sum {
            name: "Position".into(),
            fields: vec![
                ComponentField {
                    name: "x".into(),
                    datatype: Datatype::I32,
                },
                ComponentField {
                    name: "y".into(),
                    datatype: Datatype::I32,
                },
            ],
        };

        assert_eq!(Ok(expected), ComponentParser::parse_type(input));
    }
}
