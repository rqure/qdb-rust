
#[derive(Debug, Clone, PartialEq)]
pub enum RawValue {
    Unspecified,
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    EntityReference(String),
    Timestamp(DateTime<Utc>),
    ConnectionState(String),
    GarageDoorState(String),
}

impl RawValue {
    pub fn into_value(self) -> DatabaseValue {
        DatabaseValue::new(self)
    }

    pub fn as_str(&self) -> Result<String> {
        match self {
            RawValue::String(s) => Ok(s.clone()),
            _ => Err(error::Error::from_database_field("Value is not a string")),
        }
    }

    pub fn as_i64(&self) -> Result<i64> {
        match self {
            RawValue::Integer(i) => Ok(*i),
            _ => Err(error::Error::from_database_field("Value is not an integer")),
        }
    }

    pub fn as_f64(&self) -> Result<f64> {
        match self {
            RawValue::Float(f) => Ok(*f),
            _ => Err(error::Error::from_database_field("Value is not a float")),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self {
            RawValue::Boolean(b) => Ok(*b),
            _ => Err(error::Error::from_database_field("Value is not a boolean")),
        }
    }

    pub fn as_entity_reference(&self) -> Result<String> {
        match self {
            RawValue::EntityReference(e) => Ok(e.clone()),
            _ => Err(error::Error::from_database_field(
                "Value is not an entity reference",
            )),
        }
    }

    pub fn as_timestamp(&self) -> Result<DateTime<Utc>> {
        match self {
            RawValue::Timestamp(t) => Ok(*t),
            _ => Err(error::Error::from_database_field("Value is not a timestamp")),
        }
    }

    pub fn as_connection_state(&self) -> Result<String> {
        match self {
            RawValue::ConnectionState(c) => Ok(c.clone()),
            _ => Err(error::Error::from_database_field(
                "Value is not a connection state",
            )),
        }
    }

    pub fn as_garage_door_state(&self) -> Result<String> {
        match self {
            RawValue::GarageDoorState(g) => Ok(g.clone()),
            _ => Err(error::Error::from_database_field(
                "Value is not a garage door state",
            )),
        }
    }

    pub fn update_str(&mut self, value: String) -> Result<()> {
        match self {
            RawValue::String(s) => {
                *s = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field("Value is not a string")),
        }
    }

    pub fn update_i64(&mut self, value: i64) -> Result<()> {
        match self {
            RawValue::Integer(i) => {
                *i = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field("Value is not an integer")),
        }
    }

    pub fn update_f64(&mut self, value: f64) -> Result<()> {
        match self {
            RawValue::Float(f) => {
                *f = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field("Value is not a float")),
        }
    }

    pub fn update_bool(&mut self, value: bool) -> Result<()> {
        match self {
            RawValue::Boolean(b) => {
                *b = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field("Value is not a boolean")),
        }
    }

    pub fn update_entity_reference(&mut self, value: String) -> Result<()> {
        match self {
            RawValue::EntityReference(e) => {
                *e = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field(
                "Value is not an entity reference",
            )),
        }
    }

    pub fn update_timestamp(&mut self, value: DateTime<Utc>) -> Result<()> {
        match self {
            RawValue::Timestamp(t) => {
                *t = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field("Value is not a timestamp")),
        }
    }

    pub fn update_connection_state(&mut self, value: String) -> Result<()> {
        match self {
            RawValue::ConnectionState(c) => {
                *c = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field(
                "Value is not a connection state",
            )),
        }
    }

    pub fn update_garage_door_state(&mut self, value: String) -> Result<()> {
        match self {
            RawValue::GarageDoorState(g) => {
                *g = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field(
                "Value is not a garage door state",
            )),
        }
    }

    pub fn set_str(&mut self, value: String) {
        *self = RawValue::String(value);
    }

    pub fn set_i64(&mut self, value: i64) {
        *self = RawValue::Integer(value);
    }

    pub fn set_f64(&mut self, value: f64) {
        *self = RawValue::Float(value);
    }

    pub fn set_bool(&mut self, value: bool) {
        *self = RawValue::Boolean(value);
    }

    pub fn set_entity_reference(&mut self, value: String) {
        *self = RawValue::EntityReference(value);
    }

    pub fn set_timestamp(&mut self, value: DateTime<Utc>) {
        *self = RawValue::Timestamp(value);
    }

    pub fn set_connection_state(&mut self, value: String) {
        *self = RawValue::ConnectionState(value);
    }

    pub fn set_garage_door_state(&mut self, value: String) {
        *self = RawValue::GarageDoorState(value);
    }

    pub fn set_unspecified(&mut self) {
        *self = RawValue::Unspecified;
    }

    pub fn is_unspecified(&self) -> bool {
        matches!(self, RawValue::Unspecified)
    }

    pub fn is_str(&self) -> bool {
        matches!(self, RawValue::String(_))
    }

    pub fn is_i64(&self) -> bool {
        matches!(self, RawValue::Integer(_))
    }

    pub fn is_f64(&self) -> bool {
        matches!(self, RawValue::Float(_))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, RawValue::Boolean(_))
    }

    pub fn is_entity_reference(&self) -> bool {
        matches!(self, RawValue::EntityReference(_))
    }

    pub fn is_timestamp(&self) -> bool {
        matches!(self, RawValue::Timestamp(_))
    }

    pub fn is_connection_state(&self) -> bool {
        matches!(self, RawValue::ConnectionState(_))
    }

    pub fn is_garage_door_state(&self) -> bool {
        matches!(self, RawValue::GarageDoorState(_))
    }
}

type ValueRef = Rc<RefCell<RawValue>>;

pub struct DatabaseValue(ValueRef);

impl DatabaseValue {
    pub fn new(value: RawValue) -> Self {
        DatabaseValue(Rc::new(RefCell::new(value)))
    }

    pub fn clone(&self) -> Self {
        DatabaseValue(self.0.clone())
    }

    pub fn into_raw(self) -> RawValue {
        self.0.borrow().clone()
    }

    pub fn as_str(&self) -> Result<String> {
        self.0.borrow().as_str()
    }

    pub fn as_i64(&self) -> Result<i64> {
        self.0.borrow().as_i64()
    }

    pub fn as_f64(&self) -> Result<f64> {
        self.0.borrow().as_f64()
    }

    pub fn as_bool(&self) -> Result<bool> {
        self.0.borrow().as_bool()
    }

    pub fn as_entity_reference(&self) -> Result<String> {
        self.0.borrow().as_entity_reference()
    }

    pub fn as_timestamp(&self) -> Result<DateTime<Utc>> {
        self.0.borrow().as_timestamp()
    }

    pub fn as_connection_state(&self) -> Result<String> {
        self.0.borrow().as_connection_state()
    }

    pub fn as_garage_door_state(&self) -> Result<String> {
        self.0.borrow().as_garage_door_state()
    }

    pub fn update_str(&self, value: String) -> Result<()> {
        self.0.borrow_mut().update_str(value)
    }

    pub fn update_i64(&self, value: i64) -> Result<()> {
        self.0.borrow_mut().update_i64(value)
    }

    pub fn update_f64(&self, value: f64) -> Result<()> {
        self.0.borrow_mut().update_f64(value)
    }

    pub fn update_bool(&self, value: bool) -> Result<()> {
        self.0.borrow_mut().update_bool(value)
    }

    pub fn update_entity_reference(&self, value: String) -> Result<()> {
        self.0.borrow_mut().update_entity_reference(value)
    }

    pub fn update_timestamp(&self, value: DateTime<Utc>) -> Result<()> {
        self.0.borrow_mut().update_timestamp(value)
    }

    pub fn update_connection_state(&self, value: String) -> Result<()> {
        self.0.borrow_mut().update_connection_state(value)
    }

    pub fn update_garage_door_state(&self, value: String) -> Result<()> {
        self.0.borrow_mut().update_garage_door_state(value)
    }

    pub fn set_str(&self, value: String) {
        self.0.borrow_mut().set_str(value)
    }

    pub fn set_i64(&self, value: i64) {
        self.0.borrow_mut().set_i64(value)
    }

    pub fn set_f64(&self, value: f64) {
        self.0.borrow_mut().set_f64(value)
    }

    pub fn set_bool(&self, value: bool) {
        self.0.borrow_mut().set_bool(value)
    }

    pub fn set_entity_reference(&self, value: String) {
        self.0.borrow_mut().set_entity_reference(value)
    }

    pub fn set_timestamp(&self, value: DateTime<Utc>) {
        self.0.borrow_mut().set_timestamp(value)
    }

    pub fn set_connection_state(&self, value: String) {
        self.0.borrow_mut().set_connection_state(value)
    }

    pub fn set_garage_door_state(&self, value: String) {
        self.0.borrow_mut().set_garage_door_state(value)
    }

    pub fn set_unspecified(&self) {
        self.0.borrow_mut().set_unspecified()
    }

    pub fn is_unspecified(&self) -> bool {
        self.0.borrow().is_unspecified()
    }

    pub fn is_str(&self) -> bool {
        self.0.borrow().is_str()
    }

    pub fn is_i64(&self) -> bool {
        self.0.borrow().is_i64()
    }

    pub fn is_f64(&self) -> bool {
        self.0.borrow().is_f64()
    }

    pub fn is_bool(&self) -> bool {
        self.0.borrow().is_bool()
    }

    pub fn is_entity_reference(&self) -> bool {
        self.0.borrow().is_entity_reference()
    }

    pub fn is_timestamp(&self) -> bool {
        self.0.borrow().is_timestamp()
    }

    pub fn is_connection_state(&self) -> bool {
        self.0.borrow().is_connection_state()
    }

    pub fn is_garage_door_state(&self) -> bool {
        self.0.borrow().is_garage_door_state()
    }
}