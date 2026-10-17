use std::sync::atomic::{AtomicUsize, Ordering};

static IN_FLIGHT: AtomicUsize = AtomicUsize::new(0);
const MAX_IN_FLIGHT: usize = 1;

pub(super) struct Permit;

impl Permit {
    pub(super) fn acquire() -> Option<Self> {
        Self::acquire_from(&IN_FLIGHT).map(|()| Self)
    }

    fn acquire_from(counter: &AtomicUsize) -> Option<()> {
        counter
            .try_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < MAX_IN_FLIGHT).then_some(count + 1)
            })
            .ok()
            .map(|_| ())
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        IN_FLIGHT.fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_slots_are_bounded_without_polling_or_a_waiting_thread() {
        let counter = AtomicUsize::new(0);
        assert_eq!(Permit::acquire_from(&counter), Some(()));
        assert_eq!(Permit::acquire_from(&counter), None);
        counter.fetch_sub(1, Ordering::AcqRel);
        assert_eq!(Permit::acquire_from(&counter), Some(()));
    }
}
