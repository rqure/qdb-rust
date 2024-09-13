

pub type FieldRef = Rc<RefCell<RawField>>;

pub struct RawField {
    pub entity_id: String,
    pub name: String,
    pub value: DatabaseValue,
    pub write_time: DateTime<Utc>,
    pub writer_id: String,
}

impl RawField {
    pub fn entity_id(&self) -> String {
        self.entity_id.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn value(&self) -> DatabaseValue {
        self.value.clone()
    }

    pub fn write_time(&self) -> DateTime<Utc> {
        self.write_time
    }

    pub fn writer_id(&self) -> String {
        self.writer_id.clone()
    }

    pub fn update_entity_id(&mut self, entity_id: &str) {
        self.entity_id = entity_id.into();
    }

    pub fn update_value(&mut self, value: DatabaseValue) {
        self.value = value;
    }

    pub fn update_write_time(&mut self, write_time: DateTime<Utc>) {
        self.write_time = write_time;
    }

    pub fn update_writer_id(&mut self, writer_id: &str) {
        self.writer_id = writer_id.into();
    }

    pub fn update_name(&mut self, name: &str) {
        self.name = name.into();
    }

    pub fn new(entity_id: impl Into<String>, field: impl Into<String>) -> Self {
        RawField {
            entity_id: entity_id.into(),
            name: field.into(),
            value: DatabaseValue::new(RawValue::Unspecified),
            write_time: Utc::now(),
            writer_id: "".to_string(),
        }
    }

    pub fn new_with_value(
        entity_id: impl Into<String>,
        field: impl Into<String>,
        value: RawValue,
    ) -> Self {
        RawField {
            entity_id: entity_id.into(),
            name: field.into(),
            value: DatabaseValue::new(value),
            write_time: Utc::now(),
            writer_id: "".to_string(),
        }
    }

    pub fn into_field(self) -> DatabaseField {
        DatabaseField::new(self)
    }
}

pub struct DatabaseField(FieldRef);

impl Clone for DatabaseField {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl DatabaseField {
    pub fn new(field: RawField) -> Self {
        Self(Rc::new(RefCell::new(field)))
    }

    pub fn into_raw(self) -> RawField {
        let field = self.0.borrow();
        RawField {
            entity_id: field.entity_id(),
            name: field.name(),
            value: field.value(),
            write_time: field.write_time(),
            writer_id: field.writer_id(),
        }
    }

    pub fn entity_id(&self) -> String {
        self.0.borrow().entity_id()
    }

    pub fn name(&self) -> String {
        self.0.borrow().name()
    }

    pub fn value(&self) -> DatabaseValue {
        self.0.borrow().value()
    }

    pub fn write_time(&self) -> DateTime<Utc> {
        self.0.borrow().write_time()
    }

    pub fn writer_id(&self) -> String {
        self.0.borrow().writer_id()
    }

    pub fn update_entity_id(&self, entity_id: &str) {
        self.0.borrow_mut().update_entity_id(entity_id);
    }

    pub fn update_value(&self, value: DatabaseValue) {
        self.0.borrow_mut().update_value(value);
    }

    pub fn update_write_time(&self, write_time: DateTime<Utc>) {
        self.0.borrow_mut().update_write_time(write_time);
    }

    pub fn update_writer_id(&self, writer_id: &str) {
        self.0.borrow_mut().update_writer_id(writer_id);
    }

    pub fn update_name(&self, name: &str) {
        self.0.borrow_mut().update_name(name);
    }

    pub fn set_str_value(&self, value: String) -> &Self {
        self.0.borrow_mut().update_value(DatabaseValue::new(RawValue::String(value)));
        self
    }

    pub fn set_i64_value(&self, value: i64) -> &Self {
        self.0.borrow_mut().update_value(DatabaseValue::new(RawValue::Integer(value)));
        self
    }

    pub fn set_f64_value(&self, value: f64) -> &Self {
        self.0.borrow_mut().update_value(DatabaseValue::new(RawValue::Float(value)));
        self
    }

    pub fn set_bool_value(&self, value: bool) -> &Self {
        self.0.borrow_mut().update_value(DatabaseValue::new(RawValue::Boolean(value)));
        self
    }

    pub fn set_entity_reference_value(&self, value: String) -> &Self {
        self.0
            .borrow_mut()
            .update_value(DatabaseValue::new(RawValue::EntityReference(value)));
        self
    }

    pub fn set_timestamp_value(&self, value: DateTime<Utc>) -> &Self {
        self.0
            .borrow_mut()
            .update_value(DatabaseValue::new(RawValue::Timestamp(value)));
        self
    }

    pub fn set_connection_state_value(&self, value: String) -> &Self {
        self.0
            .borrow_mut()
            .update_value(DatabaseValue::new(RawValue::ConnectionState(value)));
        self
    }

    pub fn set_garage_door_state_value(&self, value: String) -> &Self {
        self.0
            .borrow_mut()
            .update_value(DatabaseValue::new(RawValue::GarageDoorState(value)));
        self
    }

    pub fn set_unspecified_value(&self) -> &Self {
        self.0.borrow_mut().update_value(DatabaseValue::new(RawValue::Unspecified));
        self
    }

}