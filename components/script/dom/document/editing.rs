/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use embedder_traits::{EditingActionEvent, EmbedderMsg, InputEventResult};
use js::context::JSContext;
use keyboard_types::{Key, Modifiers, NamedKey};
use script_bindings::codegen::GenericBindings::EventBinding::EventMethods;
use script_bindings::inheritance::Castable;
use script_bindings::match_domstring_ascii;
use script_bindings::root::DomRoot;
use script_bindings::str::DOMString;
use servo_base::generic_channel::GenericCallback;

use crate::dom::Document;
use crate::dom::clipboardevent::ClipboardEventType;
use crate::dom::event::{EventBubbles, EventCancelable};
use crate::dom::execcommand::execcommands::DocumentExecCommandSupport;
use crate::dom::types::{ClipboardEvent, DataTransfer, Element, Event, KeyboardEvent};
use crate::drag::drag_data_store::{DragDataStore, Kind, Mode};

impl Document {
    /// <https://www.w3.org/TR/clipboard-apis/#clipboard-actions>
    pub(crate) fn handle_editing_action(
        &self,
        cx: &mut JSContext,
        element: Option<DomRoot<Element>>,
        action: EditingActionEvent,
    ) -> InputEventResult {
        let clipboard_event_type = match action {
            EditingActionEvent::Copy => ClipboardEventType::Copy,
            EditingActionEvent::Cut => ClipboardEventType::Cut,
            EditingActionEvent::Paste => ClipboardEventType::Paste,
        };

        // The script_triggered flag is set if the action runs because of a script, e.g. document.execCommand()
        let script_triggered = false;

        // The script_may_access_clipboard flag is set
        // if action is paste and the script thread is allowed to read from clipboard or
        // if action is copy or cut and the script thread is allowed to modify the clipboard
        let script_may_access_clipboard = false;

        // Step 1 If the script-triggered flag is set and the script-may-access-clipboard flag is unset
        if script_triggered && !script_may_access_clipboard {
            return InputEventResult::empty();
        }

        // Step 2 Fire a clipboard event
        let clipboard_event = self.fire_clipboard_event(cx, element.clone(), clipboard_event_type);

        // Step 3 If a script doesn't call preventDefault()
        // the event will be handled inside target's VirtualMethods::handle_event
        let event = clipboard_event.upcast::<Event>();
        if !event.IsTrusted() {
            return event.flags().into();
        }

        // Step 4 If the event was canceled, then
        if event.DefaultPrevented() {
            let event_type = event.Type();
            match_domstring_ascii!(event_type,

                "copy" => {
                    // Step 4.1 Call the write content to the clipboard algorithm,
                    // passing on the DataTransferItemList items, a clear-was-called flag and a types-to-clear list.
                    if let Some(clipboard_data) = clipboard_event.get_clipboard_data() {
                        let drag_data_store =
                            clipboard_data.data_store().expect("This shouldn't fail");
                        self.write_content_to_the_clipboard(&drag_data_store);
                    }
                },
                "cut" => {
                    // Step 4.1 Call the write content to the clipboard algorithm,
                    // passing on the DataTransferItemList items, a clear-was-called flag and a types-to-clear list.
                    if let Some(clipboard_data) = clipboard_event.get_clipboard_data() {
                        let drag_data_store =
                            clipboard_data.data_store().expect("This shouldn't fail");
                        self.write_content_to_the_clipboard(&drag_data_store);
                    }

                    // Step 4.2 Fire a clipboard event named clipboardchange
                    self.fire_clipboard_event(cx, element, ClipboardEventType::Change);
                },
                // Step 4.1 Return false.
                // Note: This function deviates from the specification a bit by returning
                // the `InputEventResult` below.
                "paste" => (),
                _ => (),
            )
        }

        // Step 5: Return true from the action.
        // In this case we are returning the `InputEventResult` instead of true or false.
        event.flags().into()
    }

    /// <https://www.w3.org/TR/clipboard-apis/#fire-a-clipboard-event>
    pub(crate) fn fire_clipboard_event(
        &self,
        cx: &mut JSContext,
        target: Option<DomRoot<Element>>,
        clipboard_event_type: ClipboardEventType,
    ) -> DomRoot<ClipboardEvent> {
        let clipboard_event = ClipboardEvent::new(
            cx,
            self.window(),
            None,
            clipboard_event_type.as_str().into(),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            None,
        );

        // Step 1 Let clear_was_called be false
        // Step 2 Let types_to_clear an empty list
        let mut drag_data_store = DragDataStore::new();

        // Step 4 let clipboard-entry be the sequence number of clipboard content, null if the OS doesn't support it.

        // Step 5 let trusted be true if the event is generated by the user agent, false otherwise
        let trusted = true;

        // Step 6 if the context is editable:
        let target = target
            .map(DomRoot::upcast)
            .unwrap_or_else(|| self.event_handler().target_for_events_following_focus());

        // Step 6.2 else TODO require Selection see https://github.com/w3c/clipboard-apis/issues/70
        // Step 7
        match clipboard_event_type {
            ClipboardEventType::Copy | ClipboardEventType::Cut => {
                // Step 7.2.1
                drag_data_store.set_mode(Mode::ReadWrite);
            },
            ClipboardEventType::Paste => {
                let (callback, receiver) =
                    GenericCallback::new_blocking().expect("Could not create callback");
                self.send_to_embedder(EmbedderMsg::GetClipboardText(self.webview_id(), callback));
                let text_contents = receiver
                    .recv()
                    .map(Result::unwrap_or_default)
                    .unwrap_or_default();

                // Step 7.1.1
                drag_data_store.set_mode(Mode::ReadOnly);
                // Step 7.1.2 If trusted or the implementation gives script-generated events access to the clipboard
                if trusted {
                    // Step 7.1.2.1 For each clipboard-part on the OS clipboard:

                    // Step 7.1.2.1.1 If clipboard-part contains plain text, then
                    let data = DOMString::from(text_contents);
                    let type_ = DOMString::from_static("text/plain");
                    let _ = drag_data_store.add(Kind::Text { data, type_ });

                    // Step 7.1.2.1.2 TODO If clipboard-part represents file references, then for each file reference
                    // Step 7.1.2.1.3 TODO If clipboard-part contains HTML- or XHTML-formatted text then

                    // Step 7.1.3 Update clipboard-event-data’s files to match clipboard-event-data’s items
                    // Step 7.1.4 Update clipboard-event-data’s types to match clipboard-event-data’s items
                }
            },
            ClipboardEventType::Change => (),
        }

        // Step 3
        let clipboard_event_data = DataTransfer::new(
            cx,
            self.window(),
            Rc::new(RefCell::new(Some(drag_data_store))),
        );

        // Step 8
        clipboard_event.set_clipboard_data(Some(&clipboard_event_data));

        // Step 9
        let event = clipboard_event.upcast::<Event>();
        event.set_trusted(trusted);

        // Step 10 Set event’s composed to true.
        event.set_composed(true);

        // Step 11
        event.dispatch(cx, &target, false);

        DomRoot::from(clipboard_event)
    }

    /// <https://www.w3.org/TR/clipboard-apis/#write-content-to-the-clipboard>
    fn write_content_to_the_clipboard(&self, drag_data_store: &DragDataStore) {
        // Step 1
        if drag_data_store.list_len() > 0 {
            // Step 1.1 Clear the clipboard.
            self.send_to_embedder(EmbedderMsg::ClearClipboard(self.webview_id()));
            // Step 1.2
            for item in drag_data_store.iter_item_list() {
                match item {
                    Kind::Text { data, .. } => {
                        // Step 1.2.1.1 Ensure encoding is correct per OS and locale conventions
                        // Step 1.2.1.2 Normalize line endings according to platform conventions
                        // Step 1.2.1.3
                        self.send_to_embedder(EmbedderMsg::SetClipboardText(
                            self.webview_id(),
                            data.to_string(),
                        ));
                    },
                    Kind::File { .. } => {
                        // Step 1.2.2 If data is of a type listed in the mandatory data types list, then
                        // Step 1.2.2.1 Place part on clipboard with the appropriate OS clipboard format description
                        // Step 1.2.3 Else this is left to the implementation
                    },
                }
            }
        } else {
            // Step 2.1
            if drag_data_store.clear_was_called {
                // Step 2.1.1 If types-to-clear list is empty, clear the clipboard
                self.send_to_embedder(EmbedderMsg::ClearClipboard(self.webview_id()));
                // Step 2.1.2 Else remove the types in the list from the clipboard
                // As of now this can't be done with Arboard, and it's possible that will be removed from the spec
            }
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#additional-requirements>
    pub(crate) fn maybe_perform_editing_command(
        &self,
        cx: &mut JSContext,
        event: &KeyboardEvent,
    ) -> bool {
        if !servo_config::pref!(dom_exec_command_enabled) {
            return false;
        }
        // This function does not do any checks for whether or not we are actually inside an
        // editing host, since those checks are performed by exec_command_for_command_id either way.
        match event.key() {
            Key::Named(NamedKey::Enter) => {
                // TODO: Figure out if the bit about Option+Enter works and whether or not this
                //       ends up providing the correct behavior on Mac. (i.e. whether or not
                //       Shift should be accepted in addition to Option.)
                if event.modifiers().contains(Modifiers::SHIFT) {
                    // > When the user instructs the user agent to insert a line break inside an
                    // > editing host without breaking out of the current block, such as by
                    // > pressing Shift-Enter or Option-Enter while the cursor is in an editable
                    // > node, the user agent must call execCommand("insertlinebreak") on the
                    // > relevant document.
                    self.exec_command_for_command_id(
                        cx,
                        DOMString::from_static("insertlinebreak"),
                        DOMString::new(),
                    )
                } else {
                    // > When the user instructs the user agent to insert a line break inside an
                    // > editing host, such as by pressing the Enter key while the cursor is in an
                    // > editable node, the user agent must call execCommand("insertparagraph") on
                    // > the relevant document.
                    self.exec_command_for_command_id(
                        cx,
                        DOMString::from_static("insertparagraph"),
                        DOMString::new(),
                    )
                }
            },
            // > When the user instructs the user agent to delete the previous character inside an
            // > editing host, such as by pressing the Backspace key while the cursor is in an
            // > editable node, the user agent must call execCommand("delete") on the relevant
            // > document.
            // TODO: Gecko, Chromium and WebKit seem to delete up to the next word boundary on
            //       Ctrl+Backspace and Ctrl+Delete. We probably want that as well.
            Key::Named(NamedKey::Backspace) => self.exec_command_for_command_id(
                cx,
                DOMString::from_static("delete"),
                DOMString::new(),
            ),
            // > When the user instructs the user agent to delete the next character inside an
            // > editing host, such as by pressing the Delete key while the cursor is in an
            // > editable node, the user agent must call execCommand("forwarddelete") on the
            // > relevant document.
            Key::Named(NamedKey::Delete) => self.exec_command_for_command_id(
                cx,
                DOMString::from_static("forwarddelete"),
                DOMString::new(),
            ),
            // > When the user instructs the user agent to insert text inside an editing host, such
            // > as by typing on the keyboard while the cursor is in an editable node, the user
            // > agent must call execCommand("inserttext", false, value) on the relevant document,
            // > with value equal to the text the user provided. If the user inserts multiple
            // > characters at once or in quick succession, this specification does not define
            // > whether it is treated as one insertion or several consecutive insertions.
            Key::Character(string) => self.exec_command_for_command_id(
                cx,
                DOMString::from_static("inserttext"),
                DOMString::from(string),
            ),
            _ => false,
        }
    }
}
