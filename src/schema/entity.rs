use crate::schema::field::{DatabaseField, RawField};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatabaseEntity {
    pub id: String,
    pub type_name: String,
    pub name: String,
}

impl DatabaseEntity {
    pub fn new(id: &str, type_name: &str, name: &str) -> Self {
        DatabaseEntity {
            id: id.into(),
            type_name: type_name.into(),
            name: name.into(),
        }
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn type_name(&self) -> String {
        self.type_name.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn update_id(&mut self, id: &str) {
        self.id = id.into();
    }

    pub fn update_type_name(&mut self, type_name: &str) {
        self.type_name = type_name.into();
    }

    pub fn update_name(&mut self, name: &str) {
        self.name = name.into();
    }

    pub fn field(&self, name: &str) -> DatabaseField {
        DatabaseField::new(RawField::new(self.id(), name))
    }
}