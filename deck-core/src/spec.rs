mod manifest;

pub type Specifier {
    type Id;

    fn matches(&self, id: &Self::Id) -> bool;
}
