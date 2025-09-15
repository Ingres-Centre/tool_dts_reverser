use crate::ast::{Branch, BranchEntry, FileEntry, Serialize, Value};
use bimap::{BiHashMap, BiMap};
use lalrpop_util::{ParseError, lalrpop_mod};

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

fn build_child_path(parent: &str, ident: &str) -> String {
    if parent.is_empty() {
        ident.to_string()
    } else if parent == "/" {
        format!("{}{}", parent, ident)
    } else {
        format!("{}/{}", parent, ident)
    }
}

fn should_transform_key(key: &str, args: &[String]) -> bool {
    args.iter().any(|arg| key.contains(arg.as_str()))
}

fn integer_preview(values: &[i64], limit: usize) -> String {
    let joined = values
        .iter()
        .map(|value| format!("0x{:x}", value))
        .collect::<Vec<_>>()
        .join(", ");

    if joined.len() <= limit {
        joined
    } else {
        joined[..limit].to_string()
    }
}

pub fn first_pass_walker(map: &mut BiMap<String, i64>, parent: &str, branch: &Branch) {
    let current_path = build_child_path(parent, branch.ident);

    for entry in &branch.entries {
        match &entry {
            BranchEntry::Branch(child) => {
                first_pass_walker(map, &current_path, child);
            }
            BranchEntry::KeyValue { key, value } => {
                if *key != "phandle" {
                    continue;
                }

                match &value {
                    Value::IntegerList(integers) => {
                        map.insert(current_path.clone(), *integers.first().unwrap());
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
    for entry in branch.entries.iter_mut() {
        match entry {
            BranchEntry::Branch(child) => second_pass_walker(map, child, args),
            BranchEntry::KeyValue { key, value } => match value {
                Value::IntegerList(integers) => {
                    if *key == "phandle" {
                        integers.iter_mut().for_each(|value| *value = 0);
                        continue;
                    }

                    if !should_transform_key(key, args) {
                        continue;
                    }

                    println!("{:25}: {}", key, integer_preview(integers, 25));

                    let replacements = integers
                        .iter()
                        .map(|number| {
                            map.get_by_right(number)
                                .cloned()
                                .unwrap_or(format!("{:#x}", number))
                        })
                        .collect();

                    *value = Value::StringList(replacements);
                }
                _ => continue,
            },
            _ => continue,
        }
    }
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
