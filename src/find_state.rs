use crate::{
    find_highlight::{HighlightHandle, Highlights},
    query_input::QueryInput,
};
use gpui::{Entity, Task};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub(crate) struct FindState {
    pub input: Entity<QueryInput>,
    pub visible: bool,
    pub match_case: bool,
    pub pending: bool,
    pub dirty: bool,
    pub revision: u64,
    pub cancel: Arc<AtomicBool>,
    pub task: Option<Task<()>>,
    pub highlights: HighlightHandle,
}

impl FindState {
    pub fn new(input: Entity<QueryInput>) -> Self {
        Self {
            input,
            visible: false,
            match_case: false,
            pending: false,
            dirty: true,
            revision: 0,
            cancel: Arc::new(AtomicBool::new(false)),
            task: None,
            highlights: Rc::new(RefCell::new(Highlights::default())),
        }
    }

    pub fn cancel(&mut self) {
        self.dirty |= self.pending;
        self.cancel.store(true, Ordering::Relaxed);
        self.cancel = Arc::new(AtomicBool::new(false));
        self.revision = self.revision.wrapping_add(1);
        self.task = None;
        self.pending = false;
    }
}

impl Drop for FindState {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
