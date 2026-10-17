use std::{collections::HashMap, path::PathBuf};

#[derive(Default)]
pub(super) struct Tabs {
    pub ids: Vec<u64>,
    pub active: Option<u64>,
    next: u64,
    paths: HashMap<PathBuf, u64>,
}

impl Tabs {
    pub fn open(&mut self, path: PathBuf) -> (u64, bool) {
        if let Some(&id) = self.paths.get(&path) {
            return (id, false);
        }
        self.next += 1;
        let id = self.next;
        self.ids.push(id);
        self.paths.insert(path, id);
        (id, true)
    }

    pub fn close(&mut self, id: u64) {
        let Some(index) = self.ids.iter().position(|&other| other == id) else {
            return;
        };
        self.ids.remove(index);
        self.paths.retain(|_, other| *other != id);
        if self.active == Some(id) {
            self.active = self.ids.get(index).or_else(|| self.ids.last()).copied();
        }
    }

    pub fn rekey(&mut self, id: u64, path: PathBuf) -> Option<u64> {
        if let Some(&existing) = self.paths.get(&path) {
            return (existing != id).then_some(existing);
        }
        if self.ids.contains(&id) {
            self.paths.retain(|_, existing| *existing != id);
            self.paths.insert(path, id);
        }
        None
    }

    pub fn neighbor(&self, backwards: bool) -> Option<u64> {
        let index = self.ids.iter().position(|id| Some(*id) == self.active)?;
        let next = if backwards {
            (index + self.ids.len() - 1) % self.ids.len()
        } else {
            (index + 1) % self.ids.len()
        };
        self.ids.get(next).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_requests_reuse_stable_id_until_closed() {
        let mut tabs = Tabs::default();
        let path = PathBuf::from("a.md");
        let (id, _) = tabs.open(path.clone());
        assert_eq!(tabs.open(path.clone()), (id, false));
        tabs.close(id);
        assert_eq!(tabs.open(path), (id + 1, true));
    }

    #[test]
    fn active_close_selects_right_then_left_then_empty() {
        let mut tabs = Tabs::default();
        let first = tabs.open("a.md".into()).0;
        let second = tabs.open("b.md".into()).0;
        let third = tabs.open("c.md".into()).0;
        tabs.active = Some(second);
        tabs.close(second);
        assert_eq!(tabs.active, Some(third));
        tabs.close(third);
        assert_eq!(tabs.active, Some(first));
        tabs.close(first);
        assert_eq!(tabs.active, None);
    }

    #[test]
    fn cycling_wraps_in_visual_order() {
        let mut tabs = Tabs::default();
        let first = tabs.open("a.md".into()).0;
        let second = tabs.open("b.md".into()).0;
        tabs.active = Some(first);
        assert_eq!(tabs.neighbor(true), Some(second));
        tabs.active = Some(second);
        assert_eq!(tabs.neighbor(false), Some(first));
    }

    #[test]
    fn closing_inactive_tab_keeps_selection() {
        let mut tabs = Tabs::default();
        let first = tabs.open("a.md".into()).0;
        let second = tabs.open("b.md".into()).0;
        tabs.active = Some(second);
        tabs.close(first);
        assert_eq!(tabs.active, Some(second));
    }

    #[test]
    fn successful_retry_updates_missing_path_identity() {
        let mut tabs = Tabs::default();
        let id = tabs.open("missing.md".into()).0;
        let canonical = PathBuf::from("resolved/missing.md");
        assert_eq!(tabs.rekey(id, canonical.clone()), None);
        assert_eq!(tabs.open(canonical), (id, false));
    }

    #[test]
    fn retry_collision_preserves_existing_destination() {
        let mut tabs = Tabs::default();
        let existing = tabs.open("resolved.md".into()).0;
        let retry = tabs.open("missing-alias.md".into()).0;
        assert_eq!(tabs.rekey(retry, "resolved.md".into()), Some(existing));
        tabs.close(retry);
        assert_eq!(tabs.open("resolved.md".into()), (existing, false));
    }

    #[test]
    fn reverse_completion_preserves_supplied_tab_order() {
        let mut tabs = Tabs::default();
        let first = tabs.open("first.md".into()).0;
        let failed = tabs.open("unsupported.txt".into()).0;
        let last = tabs.open("last.md".into()).0;
        tabs.rekey(last, "canonical/last.md".into());
        tabs.rekey(first, "canonical/first.md".into());
        assert_eq!(tabs.ids, [first, failed, last]);
    }

    #[test]
    fn closed_loading_tab_rejects_late_identity_publication() {
        let mut tabs = Tabs::default();
        let id = tabs.open("pending.md".into()).0;
        tabs.close(id);
        tabs.rekey(id, "canonical/pending.md".into());
        assert_eq!(tabs.open("canonical/pending.md".into()), (id + 1, true));
    }
}
