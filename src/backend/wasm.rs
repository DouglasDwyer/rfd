use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::HtmlInputElement;

use crate::file_dialog::FileDialog;
use crate::FileHandle;

pub struct WasmDialog {
    input: HtmlInputElement,
}

impl WasmDialog {
    pub fn new(opt: &FileDialog) -> Self {
        let window = web_sys::window().expect("Window not found");
        let document = window.document().expect("Document not found");

        let input_el = document.create_element("input").unwrap();
        let input: HtmlInputElement = wasm_bindgen::JsCast::dyn_into(input_el).unwrap();

        input.set_id("rfd-input");
        input.set_type("file");

        let mut accept: Vec<String> = Vec::new();

        for filter in opt.filters.iter() {
            accept.append(&mut filter.extensions.to_vec());
        }

        accept.iter_mut().for_each(|ext| ext.insert_str(0, "."));

        input.set_accept(&accept.join(","));

        Self {
            input
        }
    }

    async fn show(&self) {
        let window = web_sys::window().expect("Window not found");
        let body = window.document().and_then(|x| x.body()).expect("Document body not found");
        self.input.focus().unwrap();
        self.input.click();
        let mut cloned_closure = Closure::wrap(Box::new(|| ()) as Box<dyn FnMut()>);
        let promise = js_sys::Promise::new(&mut |res, _rej| {
            let body_clone = body.clone();
            let cl = self.input.clone();
            let closure = Closure::once(Box::new(move || {
                let cloned_closure2 = std::sync::Arc::new(std::cell::Cell::<Option<Closure<dyn FnMut()>>>::new(None));
                let cloned_closure22 = cloned_closure2.clone();
                let body_clone2 = body_clone.clone();
                let closure2 = Closure::once(Box::new(move || {
                    res.call0(&JsValue::undefined()).unwrap();
                    drop(body_clone2.remove_event_listener_with_callback("mousemove", cloned_closure22.replace(None).unwrap().as_ref().unchecked_ref()));
                }) as Box<dyn FnOnce()>);
                drop(cl.set_oninput(Some(closure2.as_ref().unchecked_ref())));
                drop(body_clone.add_event_listener_with_callback("mousemove", closure2.as_ref().unchecked_ref()));
                cloned_closure2.set(Some(closure2));
            }) as Box<dyn FnOnce()>);

            drop(window.add_event_listener_with_callback("focus", closure.as_ref().unchecked_ref()));
            cloned_closure = closure;
        });
        let future = wasm_bindgen_futures::JsFuture::from(promise);
        future.await.unwrap();
        drop(window.remove_event_listener_with_callback("focus", cloned_closure.as_ref().unchecked_ref()));
    }

    fn get_results(&self) -> Option<Vec<FileHandle>> {
        if let Some(files) = self.input.files() {
            let len = files.length();
            if len > 0 {
                let mut file_handles = Vec::new();
                for id in 0..len {
                    let file = files.get(id).unwrap();
                    file_handles.push(FileHandle::wrap(file));
                }
                Some(file_handles)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_result(&self) -> Option<FileHandle> {
        let files = self.get_results();
        files.and_then(|mut f| f.pop())
    }

    async fn pick_files(self) -> Option<Vec<FileHandle>> {
        self.input.set_multiple(true);

        self.show().await;

        self.get_results()
    }

    async fn pick_file(self) -> Option<FileHandle> {
        self.input.set_multiple(false);

        self.show().await;

        self.get_result()
    }
}

impl Drop for WasmDialog {
    fn drop(&mut self) {
        self.input.remove();
    }
}

use super::{AsyncFilePickerDialogImpl, DialogFutureType};

impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let dialog = WasmDialog::new(&self);
        Box::pin(dialog.pick_file())
    }
    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let dialog = WasmDialog::new(&self);
        Box::pin(dialog.pick_files())
    }
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    fn confirm(s: &str) -> bool;
}

use crate::backend::MessageDialogImpl;
use crate::message_dialog::{MessageButtons, MessageDialog};

impl MessageDialogImpl for MessageDialog {
    fn show(self) -> bool {
        let text = format!("{}\n{}", self.title, self.description);
        match self.buttons {
            MessageButtons::Ok | MessageButtons::OkCustom(_) => {
                alert(&text);
                true
            }
            MessageButtons::OkCancel
            | MessageButtons::YesNo
            | MessageButtons::OkCancelCustom(_, _) => confirm(&text),
        }
    }
}

use crate::backend::AsyncMessageDialogImpl;

impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<bool> {
        let val = MessageDialogImpl::show(self);
        Box::pin(std::future::ready(val))
    }
}
