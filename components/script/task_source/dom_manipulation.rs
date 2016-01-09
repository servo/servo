/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentBinding::DocumentReadyState;
use dom::bindings::global::{GlobalRef, global_root_from_reflector};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::RootedReference;
use dom::bindings::refcounted::Trusted;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::storage::Storage;
use dom::storageevent::StorageEvent;
use dom::urlhelper::UrlHelper;
use dom::window::ReflowReason;
use layout_interface::{ReflowGoal, ReflowQueryType};
use msg::constellation_msg::{ConstellationChan, MozBrowserEvent, PipelineId};
use page::IterablePage;
use script_task::{ScriptTask, MainThreadScriptMsg, get_page};
use script_traits::ScriptMsg as ConstellationMsg;
use std::result::Result;
use std::sync::mpsc::Sender;
use string_cache::Atom;
use task_source::TaskSource;
use util::str::DOMString;

#[derive(JSTraceable)]
pub struct DOMManipulationTaskSource(pub Sender<MainThreadScriptMsg>);

impl TaskSource<DOMManipulationTaskMsg> for DOMManipulationTaskSource {
    fn queue(&self, msg: DOMManipulationTaskMsg) -> Result<(), ()> {
        let DOMManipulationTaskSource(ref chan) = *self;
        chan.send(MainThreadScriptMsg::DOMManipulation(msg)).map_err(|_| ())
    }

    fn clone(&self) -> Box<TaskSource<DOMManipulationTaskMsg> + Send> {
        let DOMManipulationTaskSource(ref chan) = *self;
        box DOMManipulationTaskSource((*chan).clone())
    }
}

pub enum DOMManipulationTaskMsg {
    // https://html.spec.whatwg.org/multipage/#the-end step 7
    DocumentLoadsComplete(PipelineId),
    // https://dom.spec.whatwg.org/#concept-event-fire
    FireEvent(Atom, Trusted<EventTarget>, EventBubbles, EventCancelable),
    // https://html.spec.whatwg.org/multipage/#fire-a-simple-event
    FireSimpleEvent(Atom, Trusted<EventTarget>),
    // https://html.spec.whatwg.org/multipage/#send-a-storage-notification
    SendStorageNotification(Trusted<Storage>, Option<String>, Option<String>, Option<String>)
}

impl DOMManipulationTaskMsg {
    pub fn handle_msg(self, script_task: &ScriptTask, constellation_chan: ConstellationChan<ConstellationMsg>) {
        use self::DOMManipulationTaskMsg::{DocumentLoadsComplete, FireEvent};
        use self::DOMManipulationTaskMsg::{FireSimpleEvent, SendStorageNotification};

        match self {
            DocumentLoadsComplete(pipeline) => {
                let page = get_page(&script_task.root_page(), pipeline);
                let doc = page.document();
                let doc = doc.r();
                if doc.loader().is_blocked() {
                    return;
                }

                doc.mut_loader().inhibit_events();

                let window = doc.window();
                if window.is_alive() {
                    // https://html.spec.whatwg.org/multipage/#the-end step 7.1
                    doc.set_ready_state(DocumentReadyState::Complete);
                    // https://html.spec.whatwg.org/multipage/#the-end step 7.2
                    let event = Event::new(GlobalRef::Window(window),
                                           atom!("load"),
                                           EventBubbles::DoesNotBubble,
                                           EventCancelable::NotCancelable);
                    let wintarget = window.upcast::<EventTarget>();
                    event.set_trusted(true);
                    let _ = wintarget.dispatch_event_with_target(doc.upcast(), &event);

                    doc.notify_constellation_load();

                    // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserloadend
                    doc.trigger_mozbrowser_event(MozBrowserEvent::LoadEnd);

                    window.reflow(ReflowGoal::ForDisplay,
                                  ReflowQueryType::NoQuery,
                                  ReflowReason::DocumentLoaded);
                }

                let ConstellationChan(ref chan) = constellation_chan;
                chan.send(ConstellationMsg::LoadComplete(pipeline)).unwrap();
            }
            FireSimpleEvent(name, element) => {
                let target = element.root();
                let target = target.r();
                let global_root = global_root_from_reflector(target);
                let global_ref = global_root.r();
                target.fire_simple_event(&*name, global_ref);
            }
            FireEvent(name, element, bubbles, cancelable) => {
                let target = element.root();
                let target = target.r();
                let global_root = global_root_from_reflector(target);
                let global_ref = global_root.r();
                target.fire_event(&*name, bubbles, cancelable, global_ref);
            }
            SendStorageNotification(element, key, old_value, new_value) => {
                let storage = element.root();
                let storage = storage.r();
                let global_root = global_root_from_reflector(storage);
                let global_ref = global_root.r();
                let window = global_ref.as_window();
                let url = global_ref.get_url();

                let storage_event = StorageEvent::new(
                    global_ref,
                    atom!("storage"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    key.map(DOMString::from),
                    old_value.map(DOMString::from),
                    new_value.map(DOMString::from),
                    DOMString::from(url.to_string()),
                    Some(storage)
                );

                let root_page = script_task.root_page();
                for it_page in root_page.iter() {
                    let it_window_root = it_page.window();
                    let it_window = it_window_root.r();
                    assert!(UrlHelper::SameOrigin(&url, &it_window.get_url()));
                    // TODO: Such a Document object is not necessarily fully active, but events fired on such
                    // objects are ignored by the event loop until the Document becomes fully active again.
                    if window.pipeline() != it_window.pipeline() {
                        storage_event.upcast::<Event>().fire(it_window.upcast());
                    }
                }
            }
        }
    }
}
