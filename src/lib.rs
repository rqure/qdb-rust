pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

use chrono::{DateTime, Utc};
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;

mod error;
