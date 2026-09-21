/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use embedder_traits::{
    ClipboardAction, EditingAction, EditingDirection, EditingMotion, EmbedderMsg, InputEventResult,
    ModifySelection,
};
use js::context::{JSContext, NoGC};
use keyboard_types::{
    Key, KeyState, KeyboardEvent as KeyboardTypesEvent, Modifiers, NamedKey, ShortcutMatcher,
};
use script_bindings::codegen::GenericBindings::DocumentBinding::DocumentMethods;
use script_bindings::codegen::GenericBindings::EventBinding::EventMethods;
use script_bindings::codegen::GenericBindings::SelectionBinding::SelectionMethods;
use script_bindings::dom::UnrootedDom;
use script_bindings::inheritance::Castable;
use script_bindings::root::DomRoot;
use script_bindings::str::DOMString;
use servo_base::generic_channel::GenericCallback;

use crate::dom::clipboardevent::ClipboardEventType;
use crate::dom::event::{EventBubbles, EventCancelable};
use crate::dom::execcommand::execcommands::DocumentExecCommandSupport;
use crate::dom::text_control::TextControlElement;
use crate::dom::text_input::{InputEventType, IsComposing};
use crate::dom::types::{
    ClipboardEvent, DataTransfer, Event, EventTarget, HTMLInputElement, HTMLTextAreaElement,
};
use crate::dom::{Document, Node, NodeTraits};
use crate::drag::drag_data_store::{DragDataStore, Kind, Mode};

#[cfg(target_os = "macos")]
pub(crate) const CMD_OR_CONTROL: Modifiers = Modifiers::META;
#[cfg(not(target_os = "macos"))]
pub(crate) const CMD_OR_CONTROL: Modifiers = Modifiers::CONTROL;

#[cfg(target_os = "macos")]
pub(crate) const ALT_OR_CONTROL: Modifiers = Modifiers::ALT;
#[cfg(not(target_os = "macos"))]
pub(crate) const ALT_OR_CONTROL: Modifiers = Modifiers::CONTROL;

impl Document {
    pub(crate) fn editing_context(&self, no_gc: &NoGC, node: &Node) -> EditingContext {
        if let Ok(editing_context) = EditingContext::try_from(node) {
            return editing_context;
        }

        let mut current_node = UnrootedDom::from_ref(node, no_gc);
        while let Some(parent) = current_node.parent_in_flat_tree(no_gc).into_parent() {
            if let Ok(editing_context) = EditingContext::try_from(&**parent) {
                return editing_context;
            }
            current_node = parent;
        }

        EditingContext::Document(DomRoot::from_ref(self))
    }

    /// <https://www.w3.org/TR/clipboard-apis/#clipboard-actions>
    pub(crate) fn handle_clipboard_action(
        &self,
        cx: &mut JSContext,
        editing_context: &EditingContext,
        action: ClipboardAction,
    ) -> InputEventResult {
        let clipboard_event_type = match action {
            ClipboardAction::Copy => ClipboardEventType::Copy,
            ClipboardAction::Cut => ClipboardEventType::Cut,
            ClipboardAction::Paste => ClipboardEventType::Paste,
        };

        // The script_triggered flag is set if the action runs because of a script, e.g. document.execCommand()
        let script_triggered = false;

        // The script_may_access_clipboard flag is set
        // if action is paste and the script thread is allowed to read from clipboard or
        // if action is copy or cut and the script thread is allowed to modify the clipboard
        let script_may_access_clipboard = false;

        // Step 1. If the script-triggered flag is set and the script-may-access-clipboard flag is unset
        if script_triggered && !script_may_access_clipboard {
            return InputEventResult::empty();
        }

        // Step 2. Fire a clipboard event
        let event_target = editing_context.event_target();
        let clipboard_event = self.fire_clipboard_event(cx, &event_target, clipboard_event_type);

        let event = clipboard_event.upcast::<Event>();
        if !event.DefaultPrevented() {
            // Step 3. If the event was not canceled, then
            match clipboard_event.clipboard_event_type() {
                ClipboardEventType::Copy => {
                    // Step 3.1. Copy the selected contents, if any, to the clipboard.
                    // Implementations should create alternate text/html and text/plain
                    // clipboard formats when content in a web page is selected.
                    if let Some(selection) = editing_context.selection_content(cx) {
                        self.send_to_embedder(EmbedderMsg::SetClipboardText(
                            self.webview_id(),
                            selection,
                        ));
                    }
                    // Step 3.2. Fire a clipboard event named clipboardchange
                    self.fire_clipboard_event(cx, &event_target, ClipboardEventType::Change);

                    // This is how `true` is returned from this function.
                    event.mark_as_handled();
                },
                ClipboardEventType::Cut => {
                    if let Some(selection) = editing_context.selection_content(cx) &&
                        editing_context.cutting_and_pasting_enabled()
                    {
                        // Step 3.1. If there is a selection in an editable context where
                        // cutting is enabled, then
                        // Step 3.1.1. Copy the selected contents, if any, to the clipboard.
                        // Implementations should create alternate text/html and text/plain
                        // clipboard formats when content in a web page is selected.
                        self.send_to_embedder(EmbedderMsg::SetClipboardText(
                            self.webview_id(),
                            selection,
                        ));

                        // Step 3.1.2. Remove the contents of the selection from the document
                        // and collapse the selection.
                        editing_context.remove_the_contents_of_the_selection(cx);

                        // Step 3.1.3. Fire a clipboard event named clipboardchange
                        self.fire_clipboard_event(cx, &event_target, ClipboardEventType::Change);

                        // Step 3.1.4. Queue tasks to fire any events that should fire due to
                        // the modification, see §5.3 Integration with other scripts and
                        // events for details.
                        editing_context.fire_cut_events();

                        // This is how `true` is returned from this function.
                        event.mark_as_handled();
                    }
                },
                ClipboardEventType::Paste => {
                    if editing_context.has_selection_or_cursor() &&
                        editing_context.cutting_and_pasting_enabled() &&
                        let Some(text_content) = clipboard_event.text_content()
                    {
                        // Step 3.1. If there is a selection or cursor in an editable context
                        // where pasting is enabled, then
                        // Step 3.1.1. Insert the most suitable content found on the
                        // clipboard, if any, into the context.
                        editing_context.insert_content(cx, &text_content);

                        // Step 3.1.2. Queue tasks to fire any events that should fire due to
                        // the modification, see §5.3 Integration with other scripts and
                        // events for details.
                        editing_context.fire_paste_events(&text_content);

                        // This is how `true` is returned from this function.
                        event.mark_as_handled();
                    }
                },
                _ => (),
            }
        } else {
            // Step 4 If the event was canceled, then
            match clipboard_event.clipboard_event_type() {
                ClipboardEventType::Copy => {
                    // Step 4.1 Call the write content to the clipboard algorithm,
                    // passing on the DataTransferItemList items, a clear-was-called flag and a types-to-clear list.
                    if let Some(clipboard_data) = clipboard_event.clipboard_data() {
                        let drag_data_store =
                            clipboard_data.data_store().expect("This shouldn't fail");
                        self.write_content_to_the_clipboard(&drag_data_store);
                    }
                },
                ClipboardEventType::Cut => {
                    // Step 4.1 Call the write content to the clipboard algorithm,
                    // passing on the DataTransferItemList items, a clear-was-called flag and a types-to-clear list.
                    if let Some(clipboard_data) = clipboard_event.clipboard_data() {
                        let drag_data_store =
                            clipboard_data.data_store().expect("This shouldn't fail");
                        self.write_content_to_the_clipboard(&drag_data_store);
                    }

                    // Step 4.2 Fire a clipboard event named clipboardchange
                    self.fire_clipboard_event(cx, &event_target, ClipboardEventType::Change);
                },
                // Step 4.1 Return false.
                ClipboardEventType::Paste => (),
                _ => (),
            }
        }

        // Step 5: Return true from the action.
        // In this case we are returning the `InputEventResult` instead of true or false.
        event.flags().into()
    }

    /// <https://www.w3.org/TR/clipboard-apis/#fire-a-clipboard-event>
    fn fire_clipboard_event(
        &self,
        cx: &mut JSContext,
        target: &EventTarget,
        clipboard_event_type: ClipboardEventType,
    ) -> DomRoot<ClipboardEvent> {
        let clipboard_event = ClipboardEvent::new(
            cx,
            self.window(),
            None,
            clipboard_event_type,
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
        // Step 6.2 else TODO require Selection see https://github.com/w3c/clipboard-apis/issues/70
        // Step 7
        match clipboard_event.clipboard_event_type() {
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
            ClipboardEventType::Change | ClipboardEventType::Other(..) => (),
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
        event.dispatch(cx, target, false);

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
    ///
    /// This function does not do any checks for whether or not we are actually inside an
    /// editing host, since those checks are performed by exec_command_for_command_id
    /// either way.
    pub(crate) fn perform_editing_action(
        &self,
        cx: &mut JSContext,
        editing_action: &EditingAction,
    ) -> bool {
        if !servo_config::pref!(dom_exec_command_enabled) {
            return false;
        }

        // This function does not do any checks for whether or not we are actually inside an
        // editing host, since those checks are performed by exec_command_for_command_id either way.
        let (command, argument) = match editing_action {
            EditingAction::MoveCursor(..) => return false,
            EditingAction::SelectAll | EditingAction::Clipboard(_) => {
                unreachable!("Should have been handled before this point.")
            },
            EditingAction::InsertNewline => {
                // > When the user instructs the user agent to insert a line break inside an
                // > editing host without breaking out of the current block, such as by
                // > pressing Shift-Enter or Option-Enter while the cursor is in an editable
                // > node, the user agent must call execCommand("insertlinebreak") on the
                // > relevant document.
                (DOMString::from_static("insertlinebreak"), DOMString::new())
            },
            EditingAction::InsertParagraph => {
                // > When the user instructs the user agent to insert a line break inside an
                // > editing host, such as by pressing the Enter key while the cursor is in an
                // > editable node, the user agent must call execCommand("insertparagraph") on
                // > the relevant document.
                (DOMString::from_static("insertparagraph"), DOMString::new())
            },
            // > When the user instructs the user agent to delete the previous character inside an
            // > editing host, such as by pressing the Backspace key while the cursor is in an
            // > editable node, the user agent must call execCommand("delete") on the relevant
            // > document.
            //
            // TODO: Handle other types of motions here.
            EditingAction::Backspace(..) => (DOMString::from_static("delete"), DOMString::new()),
            // > When the user instructs the user agent to delete the next character inside an
            // > editing host, such as by pressing the Delete key while the cursor is in an
            // > editable node, the user agent must call execCommand("forwarddelete") on the
            // > relevant document
            EditingAction::Delete => (DOMString::from_static("forwarddelete"), DOMString::new()),
            // > When the user instructs the user agent to insert text inside an editing host, such
            // > as by typing on the keyboard while the cursor is in an editable node, the user
            // > agent must call execCommand("inserttext", false, value) on the relevant document,
            // > with value equal to the text the user provided. If the user inserts multiple
            // > characters at once or in quick succession, this specification does not define
            // > whether it is treated as one insertion or several consecutive insertions.
            EditingAction::InsertText(text) => (
                DOMString::from_static("inserttext"),
                DOMString::from(text.as_str()),
            ),
        };

        self.exec_command_for_command_id(cx, command, argument)
    }
}

pub(crate) enum TextControlElementEditingContext {
    TextArea(DomRoot<HTMLTextAreaElement>),
    Input(DomRoot<HTMLInputElement>),
}

impl TextControlElementEditingContext {
    fn text_control_element(&self) -> &dyn TextControlElement {
        match self {
            TextControlElementEditingContext::TextArea(text_area) => &**text_area,
            TextControlElementEditingContext::Input(input) => &**input,
        }
    }
}

pub(crate) enum EditingContext {
    TextControl(TextControlElementEditingContext),
    Document(DomRoot<Document>),
}

impl TryFrom<&Node> for EditingContext {
    type Error = ();

    fn try_from(node: &Node) -> Result<Self, Self::Error> {
        if let Some(text_area) = node.downcast::<HTMLTextAreaElement>() {
            return Ok(EditingContext::TextControl(
                TextControlElementEditingContext::TextArea(DomRoot::from_ref(text_area)),
            ));
        }
        if let Some(input) = node.downcast::<HTMLInputElement>() &&
            input.is_textual_or_password()
        {
            return Ok(EditingContext::TextControl(
                TextControlElementEditingContext::Input(DomRoot::from_ref(input)),
            ));
        }
        Err(())
    }
}

impl EditingContext {
    pub(crate) fn event_target(&self) -> DomRoot<EventTarget> {
        match self {
            EditingContext::TextControl(element) => match element {
                TextControlElementEditingContext::TextArea(text_area) => {
                    DomRoot::from_ref(text_area.upcast())
                },
                TextControlElementEditingContext::Input(input) => DomRoot::from_ref(input.upcast()),
            },
            EditingContext::Document(document) => {
                document.event_handler().target_for_events_following_focus()
            },
        }
    }

    pub(crate) fn document(&self) -> DomRoot<Document> {
        match self {
            EditingContext::TextControl(element) => match element {
                TextControlElementEditingContext::TextArea(element) => element.owner_document(),
                TextControlElementEditingContext::Input(element) => element.owner_document(),
            },
            EditingContext::Document(document) => document.clone(),
        }
    }

    pub(crate) fn selection_content(&self, cx: &mut JSContext) -> Option<String> {
        match self {
            EditingContext::TextControl(element) => element
                .text_control_element()
                .text_input()
                .selection_content(),
            EditingContext::Document(document) => document
                .selection()
                .map(|selection| selection.Stringifier(cx).to_string())
                .filter(|selection| !selection.is_empty()),
        }
    }

    pub(crate) fn has_uncollapsed_selection(&self) -> bool {
        match self {
            EditingContext::TextControl(element) => {
                element.text_control_element().has_uncollapsed_selection()
            },
            EditingContext::Document(document) => document
                .selection()
                .is_some_and(|selection| !selection.collapsed()),
        }
    }

    pub(crate) fn has_selectable_text(&self) -> bool {
        match self {
            EditingContext::TextControl(element) => {
                element.text_control_element().has_selectable_text()
            },
            EditingContext::Document(..) => true,
        }
    }

    pub(crate) fn has_selection_or_cursor(&self) -> bool {
        match self {
            EditingContext::TextControl(..) => true,
            EditingContext::Document(document) => document
                .selection()
                .is_some_and(|selection| selection.RangeCount() > 0),
        }
    }

    pub(crate) fn cutting_and_pasting_enabled(&self) -> bool {
        match self {
            EditingContext::TextControl(element) => {
                !element.text_control_element().read_only_or_disabled()
            },
            EditingContext::Document(..) => {
                // TODO(mrobinson): Add support for integration with contenteditable.
                false
            },
        }
    }

    pub(crate) fn remove_the_contents_of_the_selection(&self, cx: &mut JSContext) {
        match self {
            EditingContext::TextControl(element) => {
                element
                    .text_control_element()
                    .remove_the_contents_of_the_selection(cx);
            },
            EditingContext::Document(..) => {
                // TODO(mrobinson): Add support for integration with contenteditable.
            },
        }
    }

    pub(crate) fn fire_cut_events(&self) {
        match self {
            EditingContext::TextControl(element) => {
                element.text_control_element().queue_input_event(
                    None,
                    IsComposing::NotComposing,
                    InputEventType::DeleteByCut,
                );
            },
            EditingContext::Document(..) => {},
        }
    }

    pub(crate) fn insert_content(&self, cx: &mut JSContext, text_content: &str) {
        match self {
            EditingContext::TextControl(element) => {
                element
                    .text_control_element()
                    .insert_content(cx, text_content);
            },
            EditingContext::Document(..) => {
                // TODO(mrobinson): Add support for integration with contenteditable.
            },
        }
    }

    pub(crate) fn fire_paste_events(&self, text_content: &str) {
        match self {
            EditingContext::TextControl(element) => {
                element.text_control_element().queue_input_event(
                    Some(text_content.to_owned()),
                    IsComposing::NotComposing,
                    InputEventType::InsertFromPaste,
                );
            },
            EditingContext::Document(..) => {},
        }
    }

    pub(crate) fn select_all(&self, cx: &mut JSContext) {
        match self {
            EditingContext::TextControl(element) => element.text_control_element().select_all(),
            EditingContext::Document(document) => {
                let Some(selection) = document.GetSelection(cx) else {
                    return;
                };
                if let Some(node) = document
                    .GetBody()
                    .map(DomRoot::upcast::<Node>)
                    .or_else(|| document.GetDocumentElement().map(DomRoot::upcast::<Node>))
                {
                    let _ = selection.SelectAllChildren(cx, &node);
                }
            },
        }
    }

    pub(crate) fn perform_editing_action(&self, cx: &mut JSContext, action: EditingAction) -> bool {
        // A couple editing actions can be performed via the external interface of
        // `EditingContext`. If that's the case we want to do that so that the same
        // code path is used regardless of how the action was executed.
        if let EditingAction::Clipboard(clipboard_action) = action {
            return self
                .document()
                .handle_clipboard_action(cx, self, clipboard_action)
                .contains(InputEventResult::Consumed);
        }

        if let EditingAction::SelectAll = action {
            self.select_all(cx);
            return true;
        }

        match self {
            EditingContext::TextControl(element) => element
                .text_control_element()
                .perform_editing_action(cx, action),
            EditingContext::Document(document) => document.perform_editing_action(cx, &action),
        }
    }
}

pub(crate) fn editing_action_from_keyboard_event(
    event: &KeyboardTypesEvent,
) -> Option<EditingAction> {
    let mut modifiers = event.modifiers;
    if let Some(action) = ShortcutMatcher::new(event.state, event.key.clone(), modifiers)
        .shortcut(CMD_OR_CONTROL, 'A', || Some(EditingAction::SelectAll))
        .shortcut(CMD_OR_CONTROL, 'C', || {
            Some(EditingAction::Clipboard(ClipboardAction::Copy))
        })
        .shortcut(CMD_OR_CONTROL, 'X', || {
            Some(EditingAction::Clipboard(ClipboardAction::Cut))
        })
        .shortcut(CMD_OR_CONTROL, 'V', || {
            Some(EditingAction::Clipboard(ClipboardAction::Paste))
        })
        // TODO: Figure out if the bit about Option+Enter works and whether or not this
        //       ends up providing the correct behavior on Mac. (i.e. whether or not
        //       Shift should be accepted in addition to Option.)
        .shortcut(Modifiers::SHIFT, Key::Named(NamedKey::Enter), || {
            Some(EditingAction::InsertNewline)
        })
        .shortcut(Modifiers::empty(), Key::Named(NamedKey::Enter), || {
            Some(EditingAction::InsertParagraph)
        })
        .otherwise(|| None)
        .flatten()
    {
        return Some(action);
    };

    let update_selection = if modifiers.contains(Modifiers::SHIFT) {
        ModifySelection::Yes
    } else {
        ModifySelection::No
    };
    modifiers.remove(Modifiers::SHIFT);

    ShortcutMatcher::new(KeyState::Down, event.key.clone(), modifiers)
        .shortcut(Modifiers::empty(), Key::Named(NamedKey::ArrowLeft), || {
            Some(EditingAction::MoveCursor(
                EditingDirection::Backward,
                EditingMotion::Grapheme,
                update_selection,
            ))
        })
        .shortcut(Modifiers::empty(), Key::Named(NamedKey::ArrowRight), || {
            Some(EditingAction::MoveCursor(
                EditingDirection::Forward,
                EditingMotion::Grapheme,
                update_selection,
            ))
        })
        .shortcut(Modifiers::empty(), Key::Named(NamedKey::ArrowUp), || {
            Some(EditingAction::MoveCursor(
                EditingDirection::Backward,
                EditingMotion::Line,
                update_selection,
            ))
        })
        .shortcut(Modifiers::empty(), Key::Named(NamedKey::ArrowDown), || {
            Some(EditingAction::MoveCursor(
                EditingDirection::Forward,
                EditingMotion::Line,
                update_selection,
            ))
        })
        .shortcut(Modifiers::CONTROL | Modifiers::ALT, 'B', || {
            Some(EditingAction::MoveCursor(
                EditingDirection::Backward,
                EditingMotion::Word,
                update_selection,
            ))
        })
        .shortcut(ALT_OR_CONTROL, Key::Named(NamedKey::ArrowLeft), || {
            Some(EditingAction::MoveCursor(
                EditingDirection::Backward,
                EditingMotion::Word,
                update_selection,
            ))
        })
        .shortcut(Modifiers::CONTROL | Modifiers::ALT, 'F', || {
            Some(EditingAction::MoveCursor(
                EditingDirection::Forward,
                EditingMotion::Word,
                update_selection,
            ))
        })
        .shortcut(ALT_OR_CONTROL, Key::Named(NamedKey::ArrowRight), || {
            Some(EditingAction::MoveCursor(
                EditingDirection::Forward,
                EditingMotion::Word,
                update_selection,
            ))
        })
        .shortcut(Modifiers::empty(), Key::Named(NamedKey::Home), || {
            Some(EditingAction::MoveCursor(
                EditingDirection::Backward,
                EditingMotion::LineStartOrEnd,
                update_selection,
            ))
        })
        .optional_shortcut(cfg!(target_os = "macos"), Modifiers::CONTROL, 'A', || {
            Some(EditingAction::MoveCursor(
                EditingDirection::Backward,
                EditingMotion::LineStartOrEnd,
                update_selection,
            ))
        })
        .shortcut(Modifiers::empty(), Key::Named(NamedKey::End), || {
            Some(EditingAction::MoveCursor(
                EditingDirection::Forward,
                EditingMotion::LineStartOrEnd,
                update_selection,
            ))
        })
        .optional_shortcut(cfg!(target_os = "macos"), Modifiers::CONTROL, 'E', || {
            Some(EditingAction::MoveCursor(
                EditingDirection::Backward,
                EditingMotion::LineStartOrEnd,
                update_selection,
            ))
        })
        .optional_shortcut(
            cfg!(target_os = "macos"),
            Modifiers::META,
            Key::Named(NamedKey::ArrowLeft),
            || {
                Some(EditingAction::MoveCursor(
                    EditingDirection::Backward,
                    EditingMotion::LineStartOrEnd,
                    update_selection,
                ))
            },
        )
        .optional_shortcut(
            cfg!(target_os = "macos"),
            Modifiers::META,
            Key::Named(NamedKey::ArrowRight),
            || {
                Some(EditingAction::MoveCursor(
                    EditingDirection::Forward,
                    EditingMotion::LineStartOrEnd,
                    update_selection,
                ))
            },
        )
        .shortcut(Modifiers::empty(), Key::Named(NamedKey::PageUp), || {
            Some(EditingAction::MoveCursor(
                EditingDirection::Backward,
                EditingMotion::Page,
                update_selection,
            ))
        })
        .shortcut(Modifiers::empty(), Key::Named(NamedKey::PageDown), || {
            Some(EditingAction::MoveCursor(
                EditingDirection::Forward,
                EditingMotion::Page,
                update_selection,
            ))
        })
        .shortcut(Modifiers::empty(), Key::Named(NamedKey::Delete), || {
            Some(EditingAction::Delete)
        })
        .shortcut(Modifiers::empty(), Key::Named(NamedKey::Backspace), || {
            Some(EditingAction::Backspace(EditingMotion::Grapheme))
        })
        .shortcut(ALT_OR_CONTROL, Key::Named(NamedKey::Backspace), || {
            Some(EditingAction::Backspace(EditingMotion::Word))
        })
        .otherwise(|| match &event.key {
            Key::Character(character) if modifiers.is_empty() => {
                Some(EditingAction::InsertText(character.to_string()))
            },
            _ => None,
        })
        .flatten()
}
