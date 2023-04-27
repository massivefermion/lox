#[derive(Debug, Clone)]
pub(crate) struct EnumOption(String, String);

impl EnumOption {
    pub fn type_of(&self) -> String {
        match self {
            EnumOption(type_of, _) => type_of.clone(),
        }
    }
}

impl From<EnumOption> for String {
    fn from(value: EnumOption) -> Self {
        match value {
            EnumOption(_, value) => value,
        }
    }
}
