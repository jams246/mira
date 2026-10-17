use std::{path::PathBuf, rc::Rc};

use gpui::{AppContext, Context, PathPromptOptions, Task, Window};

use super::{OpenRequest, Shell, Tab};
use crate::viewer::{Open, Viewer};

impl Shell {
    pub(crate) fn open_paths(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.enqueue(
            paths
                .into_iter()
                .map(|path| OpenRequest {
                    path,
                    anchor: None,
                    prepared: None,
                })
                .collect(),
            window,
            cx,
        );
    }

    pub(crate) fn open_initial_paths(
        &mut self,
        paths: Vec<PathBuf>,
        prepared: Task<std::io::Result<crate::file::LoadedDocument>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut prepared = Some(prepared);
        self.enqueue(
            paths
                .into_iter()
                .map(|path| OpenRequest {
                    path,
                    anchor: None,
                    prepared: prepared.take(),
                })
                .collect(),
            window,
            cx,
        );
    }

    fn enqueue(&mut self, requests: Vec<OpenRequest>, window: &mut Window, cx: &mut Context<Self>) {
        if requests.is_empty() {
            return;
        }
        self.opening.push_back(requests);
        self.start_opening(window, cx);
    }

    fn start_opening(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.opening_task.is_some() {
            return;
        }
        let Some(requests) = self.opening.pop_front() else {
            return;
        };
        self.message = Some("Opening Markdown...".into());
        let task = cx.background_executor().spawn(async move {
            requests
                .into_iter()
                .map(|request| {
                    let key = request.path.canonicalize().unwrap_or_else(|_| {
                        std::path::absolute(&request.path).unwrap_or_else(|_| request.path.clone())
                    });
                    (request, key)
                })
                .collect::<Vec<_>>()
        });
        self.opening_task = Some(cx.spawn_in(window, async move |shell, cx| {
            let requests = task.await;
            let _ = shell.update_in(cx, |shell, window, cx| {
                if let Some(task) = shell.opening_task.take() {
                    task.detach();
                }
                shell.message = None;
                let mut selected = None;
                for (request, key) in requests {
                    let (id, created) = shell.order.open(key.clone());
                    selected.get_or_insert(id);
                    if created {
                        shell.create_tab(id, key, request, window, cx);
                    } else if let Some(anchor) = request.anchor
                        && let Some(tab) = shell.tabs.iter().find(|tab| tab.id == id)
                    {
                        tab.reader.update(cx, |reader, cx| {
                            reader.navigate_fragment(&anchor, false, cx);
                        });
                    }
                }
                if let Some(id) = selected {
                    shell.select(id, window, cx);
                }
                shell.start_opening(window, cx);
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn create_tab(
        &mut self,
        id: u64,
        path: PathBuf,
        request: OpenRequest,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let reader = cx.new(|cx| Viewer::new(window, cx, self.timing));
        let shell = cx.weak_entity();
        reader.update(cx, |reader, cx| {
            reader.set_open_handler(Rc::new(move |path, anchor, window, cx| {
                let _ = shell.update(cx, |shell, cx| {
                    shell.enqueue(
                        vec![OpenRequest {
                            path,
                            anchor,
                            prepared: None,
                        }],
                        window,
                        cx,
                    );
                });
            }));
            if let Some(prepared) = request.prepared {
                reader.load_with_task(request.path, request.anchor, prepared, window, cx);
            } else {
                reader.load_with_anchor(request.path, request.anchor, window, cx);
            }
            reader.deactivate(window, cx);
        });
        let observation = cx.observe_in(&reader, window, move |shell, reader, window, cx| {
            shell.reader_changed(id, &reader, window, cx);
        });
        self.tabs.push(Tab {
            id,
            path,
            reader,
            _observation: observation,
        });
    }

    pub(super) fn open(&mut self, _: &Open, window: &mut Window, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("Open Markdown".into()),
        });
        cx.spawn_in(window, async move |shell, cx| match paths.await {
            Ok(Ok(Some(paths))) => {
                let _ = shell.update_in(cx, |shell, window, cx| {
                    shell.open_paths(paths, window, cx);
                });
            }
            Ok(Ok(None)) => {}
            result => {
                let _ = shell.update(cx, |shell, cx| {
                    shell.message = Some(format!("Cannot open file dialog: {result:?}"));
                    cx.notify();
                });
            }
        })
        .detach();
    }
}
