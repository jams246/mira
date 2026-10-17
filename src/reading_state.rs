//! Semantic reading locations and independent per-document navigation history.

/// A layout-independent text position with its viewport-relative offset.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Location {
    /// Semantic block index.
    pub block: usize,
    /// Optional table cell identity within the block.
    pub cell: Option<usize>,
    /// Block-global semantic UTF-8 byte offset, including table separators.
    pub byte: usize,
    /// Vertical viewport offset in logical pixels.
    pub offset: f32,
}

/// Back and forward locations for one open document.
#[derive(Default)]
pub struct History {
    back: Vec<Location>,
    forward: Vec<Location>,
}

impl History {
    /// Records a deliberate jump, discarding any superseded forward branch.
    pub fn jump(&mut self, from: Location, to: Location) {
        if from != to {
            self.back.push(from);
            self.forward.clear();
        }
    }

    /// Returns the previous jump location and retains the current departure.
    pub fn back(&mut self, current: Location) -> Option<Location> {
        let target = self.back.pop()?;
        self.forward.push(current);
        Some(target)
    }

    /// Returns the next jump location and retains the current departure.
    pub fn forward(&mut self, current: Location) -> Option<Location> {
        let target = self.forward.pop()?;
        self.back.push(current);
        Some(target)
    }

    /// Whether a previous jump is available.
    pub fn can_back(&self) -> bool {
        !self.back.is_empty()
    }

    /// Whether a forward jump is available.
    pub fn can_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    /// Removes locations invalidated by replacement document content.
    pub fn clear(&mut self) {
        self.back.clear();
        self.forward.clear();
    }
}

#[cfg(test)]
mod tests;
