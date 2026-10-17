mod find;
mod input;
mod navigation;
mod reload;
pub(crate) mod support;
mod zoom;

use std::{
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex},
    task::Poll,
};

use gpui::{AppContext, Application, Context, IntoElement, Render, Window, div};

use super::super::Viewer;
use crate::timing::Timing;

#[test]
fn hidden_native_reader_preserves_state_and_rejects_stale_async_publications() {
    let outcome = Arc::new(Mutex::new(None));
    let result = Arc::clone(&outcome);
    Application::new().run(move |cx| {
        crate::viewer::bind_keys(cx);
        crate::query_input::bind_keys(cx);
        crate::shell::bind_keys(cx);
        cx.open_window(support::window_options(), |_, cx| cx.new(|_| Guard))
            .expect("native test guard window");
        let window = cx
            .open_window(support::window_options(), |window, cx| {
                cx.new(|cx| Viewer::new(window, cx, Timing::start()))
            })
            .expect("hidden native test window");
        cx.spawn(async move |cx| {
            let mut work = Box::pin(async {
                support::realize(window, cx).await;
                zoom::run(window, cx).await;
                input::run(window, cx);
                navigation::run(window, cx).await;
                reload::run(window, cx).await;
                find::run(window, cx).await;
                crate::shell::native_tests::run(cx).await;
            });
            let completed = std::future::poll_fn(|task| {
                match catch_unwind(AssertUnwindSafe(|| work.as_mut().poll(task))) {
                    Ok(Poll::Ready(())) => Poll::Ready(Ok(())),
                    Ok(Poll::Pending) => Poll::Pending,
                    Err(error) => Poll::Ready(Err(error)),
                }
            })
            .await;
            drop(work);
            *result.lock().expect("native test result") = Some(completed.map_err(|error| {
                error
                    .downcast_ref::<&str>()
                    .map(|text| (*text).to_owned())
                    .or_else(|| error.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "native test panicked".into())
            }));
            cx.update(|cx| cx.quit()).expect("quit native tests");
        })
        .detach();
    });
    assert_eq!(
        outcome.lock().expect("native test result").take(),
        Some(Ok(()))
    );
}

struct Guard;

impl Render for Guard {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
