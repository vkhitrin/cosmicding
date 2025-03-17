use crate::fl;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextPage {
    About,
    AddAccountForm,
    EditAccountForm,
    EditBookmarkForm,
    NewBookmarkForm,
    Settings,
    ViewBookmarkNotes,
}

impl ContextPage {
    pub fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
            Self::Settings => fl!("settings"),
            Self::AddAccountForm => fl!("add-account"),
            Self::EditAccountForm => fl!("edit-account"),
            Self::NewBookmarkForm => fl!("add-bookmark"),
            Self::EditBookmarkForm => fl!("edit-bookmark"),
            Self::ViewBookmarkNotes => fl!("notes"),
        }
    }
}
