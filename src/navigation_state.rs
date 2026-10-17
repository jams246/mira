use gpui::ScrollHandle;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub(crate) type CodeScrolls = Rc<RefCell<HashMap<usize, ScrollHandle>>>;
