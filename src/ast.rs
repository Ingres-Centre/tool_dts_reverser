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
    KeyValue { key: &'a str, value: Value<'a> },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value<'a> {
    IntegerList(Vec<i64>),
    ByteList(Vec<u8>),
    StringList(Vec<&'a str>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    pub version: u32,
}

pub trait Serialize<'a> {
    fn serialize(&'a self, level: usize) -> String;
}

impl<'a> Serialize<'a> for Header {
    fn serialize(&'a self, _: usize) -> String {
        format!("/dts-v{}/;\n\n", self.version)
    }
}

impl<'a> Serialize<'a> for Value<'a> {
    fn serialize(&'a self, _: usize) -> String {
        match &self {
            Value::IntegerList(inner) => {
                let strings: Vec<String> = inner.iter().map(|i| format!("{:#04x}", i)).collect();
                format!("<{}>", strings.join(" "))
            }
            Value::ByteList(inner) => {
                let strings: Vec<String> = inner.iter().map(|i| format!("{:02x}", i)).collect();
                format!("[{}]", strings.join(" "))
            }
            Value::StringList(inner) => {
                let strings: Vec<String> = inner.iter().map(|i| format!("\"{}\"", i)).collect();
                strings.join(", ")
            }
        }
    }
}

impl<'a> Serialize<'a> for BranchEntry<'a> {
    fn serialize(&'a self, level: usize) -> String {
        match &self {
            BranchEntry::Branch(inner) => inner.serialize(level),
            BranchEntry::Key(ident) => {
                format!("{}{};\n", "\t".repeat(level), ident)
            }
            BranchEntry::KeyValue { key, value } => {
                format!(
                    "{}{} = {};\n",
                    "\t".repeat(level),
                    key,
                    value.serialize(level)
                )
            }
        }
    }
}

impl<'a> Serialize<'a> for Branch<'a> {
    fn serialize(&'a self, level: usize) -> String {
        let entries: Vec<String> = self
            .entries
            .iter()
            .map(|v| v.serialize(level + 1))
            .collect();

        format!(
            "{}{} {{\n{}{}}};\n",
            "\t".repeat(level),
            self.ident,
            entries.join(""),
            "\t".repeat(level)
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
