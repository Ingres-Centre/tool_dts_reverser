use crate::ast::{Branch, BranchEntry, FileEntry, Serialize, Value};
use bimap::{BiHashMap, BiMap};
use lalrpop_util::{lalrpop_mod, ParseError};

lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[rustfmt::skip]
    pub dts
);

pub mod ast;
pub mod lexer;

pub fn parse(
    data: &'_ str,
) -> Result<ast::File<'_>, ParseError<usize, lexer::Token<'_>, lexer::LexicalError<'_>>> {
    let lexer = lexer::Lexer::new(data);
    let parser = dts::FileParser::new();

    parser.parse(data, lexer)
}

pub fn first_pass_walker(map: &mut BiMap<String, i64>, parent: &str, branch: &Branch) {
    let me = if parent.is_empty() {
        branch.ident.to_string()
    } else if parent == "/" {
        format!("{}{}", parent, branch.ident)
    } else {
        format!("{}/{}", parent, branch.ident)
    };

    for entry in &branch.entries {
        match &entry {
            BranchEntry::Branch(child) => {
                first_pass_walker(map, &me, child);
            }
            BranchEntry::KeyValue { key, value } => {
                if *key != "phandle" {
                    continue;
                }

                match &value {
                    Value::IntegerList(integers) => {
                        map.insert(me.clone(), *integers.first().unwrap());
                    }
                    _ => unreachable!(),
                }
            }
            _ => continue,
        }
    }
}

pub fn second_pass_walker<'a>(
    map: &'a BiHashMap<String, i64>,
    branch: &mut Branch<'a>,
    args: &[String],
) {
    let entries: Vec<_> = branch
        .entries
        .iter()
        .map(|entry| match entry {
            BranchEntry::Branch(child) => {
                let mut child = child.clone();
                second_pass_walker(map, &mut child, args);
                BranchEntry::Branch(child)
            }
            BranchEntry::KeyValue { key, value } => BranchEntry::KeyValue {
                key,
                value: match &value {
                    Value::IntegerList(integers) => {
                        if !args.iter().any(|arg| key.contains(arg.as_str())) || *key == "phandle" {
                            value.clone()
                        } else {
                            let numbers_str: Vec<String> =
                                integers.iter().map(|i| format!("0x{:x}", i)).collect();
                            let numbers_str = numbers_str.join(", ");

                            let norm = if numbers_str.len() > 25 {
                                &numbers_str[0..25]
                            } else {
                                &numbers_str
                            };

                            println!("{:25}: {}", key, norm);

                            Value::StringList(
                                integers
                                    .iter()
                                    .map(|i| {
                                        map.get_by_right(i)
                                            .map(|str| str.as_str())
                                            .unwrap_or("##UNKNOWN##")
                                    })
                                    .collect(),
                            )
                        }
                    }
                    value => (*value).clone(),
                },
            },
            v => v.clone(),
        })
        .collect();

    branch.entries = entries;
}
fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() < 3 {
        eprintln!("Usage: {} <in-dts> <out-dts> [str]+", args[0]);
        return;
    }

    let in_filename = args[1].clone();
    let out_filename = args[2].clone();
    let search_idents = args.split_at(3).1;

    let data = std::fs::read_to_string(in_filename).unwrap();
    let result = parse(&data);

    match result {
        Err(e) => match e {
            ParseError::UnrecognizedToken { token, .. } => {
                println!("Unrecognized token: {:?}", token.1);

                let line = data[..token.0].chars().filter(|&ch| ch == '\n').count() + 1;
                let column = token.0 - data[..token.0].rfind("\n").unwrap_or(0);
                let position = format!("line {}, column {}", line, column);

                println!(
                    "Invalid token \"{}\" at {}",
                    &data[token.0..token.2],
                    position
                )
            }
            ParseError::User { error } => {
                println!("Error: {}", error);
            }
            _ => println!("{:?}", e),
        },
        Ok(mut res) => {

            let mut ident_phandle: BiMap<String, i64> = BiMap::new();

            for entry in &res {
                match entry {
                    FileEntry::Branch(branch) => {
                        first_pass_walker(&mut ident_phandle, "", branch);
                    }
                    _ => continue,
                }
            }

            for entry in &mut res {
                match entry {
                    FileEntry::Branch(branch) => {
                        second_pass_walker(&ident_phandle, branch, search_idents);
                    }
                    _ => continue,
                }
            }

            let out = res.serialize(0);
            std::fs::write(out_filename, out).unwrap();
        }
    }
}
