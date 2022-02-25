#[derive(PartialEq)]
pub(crate) struct HelmetState {
    pub tags: Vec<HelmetTag>,
}

#[derive(PartialEq)]
pub(crate) enum HelmetTag {
    Title(String),
}
