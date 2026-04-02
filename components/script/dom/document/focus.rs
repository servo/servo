/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use bitflags::bitflags;
use embedder_traits::FocusSequenceNumber;
use script_bindings::inheritance::Castable;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::script_runtime::CanGc;
use servo_constellation_traits::ScriptToConstellationMessage;

use crate::dom::bindings::root::MutNullableDom;
use crate::dom::execcommand::contenteditable::ContentEditableRange;
use crate::dom::focusevent::FocusEventType;
use crate::dom::types::{Element, EventTarget, FocusEvent, HTMLElement, HTMLIFrameElement, Window};
use crate::dom::{Event, EventBubbles, EventCancelable, Node};

pub(crate) enum FocusOperation {
    Focus(FocusableArea),
    Unfocus,
}

/// The kind of focusable area a [`FocusableArea`] is. A [`FocusableArea`] may be click focusable,
/// sequentially focusable, or both.
#[derive(Clone, Copy, Debug, Default, MallocSizeOf)]
pub(crate) struct FocusableAreaKind(u8);

bitflags! {
    impl FocusableAreaKind: u8 {
        /// <https://html.spec.whatwg.org/multipage/#click-focusable>
        ///
        /// > A focusable area is said to be click focusable if the user agent determines that it is
        /// > click focusable. User agents should consider focusable areas with non-null tabindex values
        /// > to be click focusable.
        const Click = 1 << 0;
        /// <https://html.spec.whatwg.org/multipage/#sequentially-focusable>.
        ///
        /// > A focusable area is said to be sequentially focusable if it is included in its
        /// > Document's sequential focus navigation order and the user agent determines that it is
        /// > sequentially focusable.
        const Sequential = 1 << 1;
    }
}

pub(crate) enum FocusableArea {
    Node {
        node: DomRoot<Node>,
        kind: FocusableAreaKind,
    },
    Viewport,
}

impl FocusableArea {
    pub(crate) fn kind(&self) -> FocusableAreaKind {
        match self {
            FocusableArea::Node { kind, .. } => *kind,
            FocusableArea::Viewport => FocusableAreaKind::Click | FocusableAreaKind::Sequential,
        }
    }
}

/// Specifies the initiator of a focus operation.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum FocusInitiator {
    /// The operation is initiated by a focus change in this [`Document`]. This
    /// means the change might trigger focus changes in parent [`Document`]s.
    Local,
    /// The operation is initiated somewhere else, and we are updating our
    /// internal state accordingly.
    Remote,
}

/// The [`DocumentFocusHandler`] is a structure responsible for handling and storing data related to
/// focus for the `Document`. It exists to decrease the size of the `Document`.
/// structure.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DocumentFocusHandler {
    /// The [`Window`] element for this [`DocumentFocusHandler`].
    window: Dom<Window>,
    /// The element that currently has focus in the `Document`.
    focused_element: MutNullableDom<Element>,
    /// The last sequence number sent to the constellation.
    #[no_trace]
    focus_sequence: Cell<FocusSequenceNumber>,
    /// Indicates whether the container is included in the top-level browsing
    /// context's focus chain (not considering system focus). Permanently `true`
    /// for a top-level document.
    has_focus: Cell<bool>,
}

impl DocumentFocusHandler {
    pub(crate) fn new(window: &Window, has_focus: bool) -> Self {
        Self {
            window: Dom::from_ref(window),
            focused_element: Default::default(),
            focus_sequence: Cell::new(FocusSequenceNumber::default()),
            has_focus: Cell::new(has_focus),
        }
    }

    pub(crate) fn has_focus(&self) -> bool {
        self.has_focus.get()
    }

    /// Return the element that currently has focus. If `None` is returned the viewport itself has focus.
    pub(crate) fn focused_element(&self) -> Option<DomRoot<Element>> {
        self.focused_element.get()
    }

    /// Get the last sequence number sent to the constellation.
    ///
    /// Received focus-related messages with sequence numbers less than the one
    /// returned by this method must be discarded.
    pub fn focus_sequence(&self) -> FocusSequenceNumber {
        self.focus_sequence.get()
    }

    /// Generate the next sequence number for focus-related messages.
    fn increment_fetch_focus_sequence(&self) -> FocusSequenceNumber {
        self.focus_sequence.set(FocusSequenceNumber(
            self.focus_sequence
                .get()
                .0
                .checked_add(1)
                .expect("too many focus messages have been sent"),
        ));
        self.focus_sequence.get()
    }

    /// Update the local focus state accordingly after being notified that the
    /// document's container is removed from the top-level browsing context's
    /// focus chain (not considering system focus).
    pub(crate) fn handle_container_unfocus(&self, can_gc: CanGc) {
        if self.window.parent_info().is_none() {
            warn!("Top-level document cannot be unfocused");
            return;
        }
        self.focus(FocusOperation::Unfocus, FocusInitiator::Remote, can_gc);
    }

    /// Reassign the focus context to the element that last requested focus during this
    /// transaction, or the document if no elements requested it.
    pub(crate) fn focus(
        &self,
        focus_operation: FocusOperation,
        focus_initiator: FocusInitiator,
        can_gc: CanGc,
    ) {
        let (mut new_focused, new_focus_state) = match focus_operation {
            FocusOperation::Focus(focusable_area) => (
                match focusable_area {
                    FocusableArea::Node { node, .. } => DomRoot::downcast::<Element>(node),
                    FocusableArea::Viewport => None,
                },
                true,
            ),
            FocusOperation::Unfocus => (
                self.focused_element.get().as_deref().map(DomRoot::from_ref),
                false,
            ),
        };

        if !new_focus_state {
            // In many browsers, a document forgets its focused area when the
            // document is removed from the top-level BC's focus chain
            if new_focused.take().is_some() {
                trace!(
                    "Forgetting the document's focused area because the \
                    document's container was removed from the top-level BC's \
                    focus chain"
                );
            }
        }

        let old_focused = self.focused_element.get();
        let old_focus_state = self.has_focus.get();

        debug!(
            "Committing focus transaction: {:?} → {:?}",
            (&old_focused, old_focus_state),
            (&new_focused, new_focus_state),
        );

        // `*_focused_filtered` indicates the local element (if any) included in
        // the top-level BC's focus chain.
        let old_focused_filtered = old_focused.as_ref().filter(|_| old_focus_state);
        let new_focused_filtered = new_focused.as_ref().filter(|_| new_focus_state);

        let trace_focus_chain = |name, element, doc| {
            trace!(
                "{} local focus chain: {}",
                name,
                match (element, doc) {
                    (Some(e), _) => format!("[{:?}, document]", e),
                    (None, true) => "[document]".to_owned(),
                    (None, false) => "[]".to_owned(),
                }
            );
        };

        trace_focus_chain("Old", old_focused_filtered, old_focus_state);
        trace_focus_chain("New", new_focused_filtered, new_focus_state);

        if old_focused_filtered != new_focused_filtered {
            if let Some(elem) = &old_focused_filtered {
                let node = elem.upcast::<Node>();
                elem.set_focus_state(false);
                // FIXME: pass appropriate relatedTarget
                if node.is_connected() {
                    self.fire_focus_event(FocusEventType::Blur, node.upcast(), None, can_gc);
                }
            }
        }

        if old_focus_state != new_focus_state && !new_focus_state {
            self.fire_focus_event(FocusEventType::Blur, self.window.upcast(), None, can_gc);
        }

        self.focused_element.set(new_focused.as_deref());
        self.has_focus.set(new_focus_state);

        if old_focus_state != new_focus_state && new_focus_state {
            self.fire_focus_event(FocusEventType::Focus, self.window.upcast(), None, can_gc);
        }

        if old_focused_filtered != new_focused_filtered {
            if let Some(elem) = &new_focused_filtered {
                elem.set_focus_state(true);
                let node = elem.upcast::<Node>();
                if let Some(html_element) = elem.downcast::<HTMLElement>() {
                    html_element.handle_focus_state_for_contenteditable(can_gc);
                }
                // FIXME: pass appropriate relatedTarget
                self.fire_focus_event(FocusEventType::Focus, node.upcast(), None, can_gc);
            }
        }

        if focus_initiator == FocusInitiator::Remote {
            return;
        }

        // We are the initiator of the focus operation, so we must broadcast
        // the change we intend to make.
        match (old_focus_state, new_focus_state) {
            (_, true) => {
                // Advertise the change in the focus chain.
                // <https://html.spec.whatwg.org/multipage/#focus-chain>
                // <https://html.spec.whatwg.org/multipage/#focusing-steps>
                //
                // If the top-level BC doesn't have system focus, this won't
                // have an immediate effect, but it will when we gain system
                // focus again. Therefore we still have to send `ScriptMsg::
                // Focus`.
                //
                // When a container with a non-null nested browsing context is
                // focused, its active document becomes the focused area of the
                // top-level browsing context instead. Therefore we need to let
                // the constellation know if such a container is focused.
                //
                // > The focusing steps for an object `new focus target` [...]
                // >
                // >  3. If `new focus target` is a browsing context container
                // >     with non-null nested browsing context, then set
                // >     `new focus target` to the nested browsing context's
                // >     active document.
                let child_browsing_context_id = new_focused
                    .as_ref()
                    .and_then(|elem| elem.downcast::<HTMLIFrameElement>())
                    .and_then(|iframe| iframe.browsing_context_id());

                let sequence = self.increment_fetch_focus_sequence();

                debug!(
                    "Advertising the focus request to the constellation \
                        with sequence number {} and child BC ID {}",
                    sequence,
                    child_browsing_context_id
                        .as_ref()
                        .map(|id| id as &dyn std::fmt::Display)
                        .unwrap_or(&"(none)"),
                );

                self.window
                    .send_to_constellation(ScriptToConstellationMessage::Focus(
                        child_browsing_context_id,
                        sequence,
                    ));
            },
            (false, false) => {
                // Our `Document` doesn't have focus, and we intend to keep it
                // this way.
            },
            (true, false) => {
                unreachable!(
                    "Can't lose the document's focus without specifying \
                    another one to focus"
                );
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#fire-a-focus-event>
    fn fire_focus_event(
        &self,
        focus_event_type: FocusEventType,
        event_target: &EventTarget,
        related_target: Option<&EventTarget>,
        can_gc: CanGc,
    ) {
        let (event_name, does_bubble) = match focus_event_type {
            FocusEventType::Focus => ("focus".into(), EventBubbles::DoesNotBubble),
            FocusEventType::Blur => ("blur".into(), EventBubbles::DoesNotBubble),
        };
        let event = FocusEvent::new(
            &self.window,
            event_name,
            does_bubble,
            EventCancelable::NotCancelable,
            Some(&self.window),
            0i32,
            related_target,
            can_gc,
        );
        let event = event.upcast::<Event>();
        event.set_trusted(true);
        event.fire(event_target, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#focus-fixup-rule>
    /// > For each doc of docs, if the focused area of doc is not a focusable area, then run the
    /// > focusing steps for doc's viewport, and set doc's relevant global object's navigation API's
    /// > focus changed during ongoing navigation to false.
    ///
    /// TODO: Handle the "focus changed during ongoing navigation" flag.
    pub(crate) fn perform_focus_fixup_rule(&self, can_gc: CanGc) {
        if self
            .focused_element
            .get()
            .as_deref()
            .is_none_or(|focused| focused.is_focusable_area())
        {
            return;
        }
        self.focus(
            FocusOperation::Focus(FocusableArea::Viewport),
            FocusInitiator::Local,
            can_gc,
        );
    }
}
