use std::{
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

use gpui::{
    AppContext, AsyncApp, BackgroundExecutor, Bounds, Task, WindowBounds, WindowHandle,
    WindowOptions, point, px, size,
};

use super::Viewer;

pub(crate) fn window_options() -> WindowOptions {
    WindowOptions {
        show: false,
        focus: false,
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
            point(px(50.), px(50.)),
            size(px(900.), px(700.)),
        ))),
        ..Default::default()
    }
}

pub(crate) async fn realize<V: 'static>(window: WindowHandle<V>, cx: &mut AsyncApp) {
    // Windows defers initial placement until a hidden window is shown.
    cx.update_window(window.into(), |_, window, _| {
        window.resize(size(px(900.), px(700.)))
    })
    .expect("size hidden native window");
    until(
        cx,
        |cx| {
            cx.update_window(window.into(), |_, window, _| {
                window.viewport_size().width > px(0.) && window.viewport_size().height > px(0.)
            })
            .unwrap()
        },
        "hidden native viewport initialization",
    )
    .await;
}

pub(crate) fn fixture(name: &str, source: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".verification/reading-features/native-integration");
    std::fs::create_dir_all(&directory).expect("native fixture directory");
    let path = directory.join(name);
    std::fs::write(&path, source).expect("native fixture");
    path
}

pub(crate) async fn until(
    cx: &mut AsyncApp,
    mut ready: impl FnMut(&mut AsyncApp) -> bool,
    operation: &str,
) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !ready(cx) {
        assert!(Instant::now() < deadline, "Timed out: {operation}");
        cx.background_executor()
            .timer(Duration::from_millis(5))
            .await;
    }
}

pub(crate) fn settled(reader: &Viewer) -> bool {
    !reader.loading
}

pub(crate) fn failed(reader: &Viewer) -> bool {
    !reader.loading && !reader.is_loaded() && reader.message.starts_with("Cannot open ")
}

pub(crate) fn track_caches(reader: &Viewer) -> impl Fn() -> [bool; 4] + use<> {
    let text = Rc::downgrade(reader.texts.as_ref().expect("rendered text cache"));
    let code = Rc::downgrade(reader.codes.as_ref().expect("rendered code cache"));
    let flow = Rc::downgrade(reader.flows.as_ref().expect("rendered flow cache"));
    let image = Rc::downgrade(reader.images.as_ref().expect("rendered image cache"));
    move || {
        [
            text.upgrade().is_some(),
            code.upgrade().is_some(),
            flow.upgrade().is_some(),
            image.upgrade().is_some(),
        ]
    }
}

pub(crate) struct Gate {
    release: Option<mpsc::Sender<()>>,
    pub ready: Arc<AtomicBool>,
    pub done: Arc<AtomicBool>,
}

impl Drop for Gate {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
    }
}

pub(crate) fn gated<T: Send + 'static>(
    executor: &BackgroundExecutor,
    work: impl FnOnce() -> T + Send + 'static,
) -> (Task<T>, Gate) {
    let (release, wait) = mpsc::channel();
    let ready = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicBool::new(false));
    let worker_ready = Arc::clone(&ready);
    let worker_done = Arc::clone(&done);
    let task = executor.spawn(async move {
        let result = work();
        worker_ready.store(true, Ordering::Release);
        let _ = wait.recv();
        worker_done.store(true, Ordering::Release);
        result
    });
    (
        task,
        Gate {
            release: Some(release),
            ready,
            done,
        },
    )
}

pub(super) async fn load(window: WindowHandle<Viewer>, path: PathBuf, cx: &mut AsyncApp) {
    let expected = path.canonicalize().expect("fixture canonical path");
    window
        .update(cx, |reader, window, cx| reader.load(path, window, cx))
        .expect("start native document load");
    until(
        cx,
        |cx| {
            window
                .update(cx, |reader, _, _| {
                    !reader.loading && reader.loaded_path() == Some(expected.as_path())
                })
                .expect("read native load")
        },
        "native document load",
    )
    .await;
}
