pub type File<'a> = Vec<FileEntry<'a>>;

#[derive(Debug, PartialEq, Clone)]
pub enum FileEntry<'a> {
    Header(Header),
    Branch(Branch<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Branch<'a> {
    pub ident: &'a str,
    pub entries: Vec<BranchEntry<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BranchEntry<'a> {
    Branch(Branch<'a>),
    Key(&'a str),
    KeyValue { key: &'a str, value: Value },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    IntegerList(Vec<i64>),
    ByteList(Vec<u8>),
    StringList(Vec<String>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    pub version: u32,
}

pub trait Serialize<'a> {
    fn serialize(&'a self, level: usize) -> String;
}

fn indent(level: usize) -> String {
    "\t".repeat(level)
}

fn join_map<I, F>(iter: I, f: F, separator: &str) -> String
where
    I: IntoIterator,
    F: FnMut(I::Item) -> String,
{
    iter.into_iter().map(f).collect::<Vec<_>>().join(separator)
}

impl<'a> Serialize<'a> for Header {
    fn serialize(&'a self, _: usize) -> String {
        format!("/dts-v{}/;\n\n", self.version)
    }
}

impl<'a> Serialize<'a> for Value {
    fn serialize(&'a self, _: usize) -> String {
        match self {
            Value::IntegerList(inner) => {
                let contents = join_map(inner.iter(), |value| format!("{:#04x}", value), " ");
                format!("<{}>", contents)
            }
            Value::ByteList(inner) => {
                let contents = join_map(inner.iter(), |value| format!("{:02x}", value), " ");
                format!("[{}]", contents)
            }
            Value::StringList(inner) => {
                join_map(inner.iter(), |value| format!("\"{}\"", value), ", ")
            }
        }
    }
}

impl<'a> Serialize<'a> for BranchEntry<'a> {
    fn serialize(&'a self, level: usize) -> String {
        match &self {
            BranchEntry::Branch(inner) => inner.serialize(level),
            BranchEntry::Key(ident) => {
                format!("{}{};\n", indent(level), ident)
            }
            BranchEntry::KeyValue { key, value } => {
                format!("{}{} = {};\n", indent(level), key, value.serialize(level))
            }
        }
    }
}

impl<'a> Serialize<'a> for Branch<'a> {
    fn serialize(&'a self, level: usize) -> String {
        let entries = join_map(self.entries.iter(), |entry| entry.serialize(level + 1), "");
        let indent = indent(level);

        format!(
            "{}{} {{\n{}{}}};\n",
            indent.clone(),
            self.ident,
            entries,
            indent
        )
    }
}

impl<'a> Serialize<'a> for FileEntry<'a> {
    fn serialize(&'a self, level: usize) -> String {
        match &self {
            FileEntry::Header(inner) => inner.serialize(level),
            FileEntry::Branch(inner) => inner.serialize(level),
        }
    }
}

impl<'a> Serialize<'a> for File<'a> {
    fn serialize(&'a self, level: usize) -> String {
        let entries: Vec<String> = self.iter().map(|v| v.serialize(level)).collect();

        entries.join("")
    }
}
