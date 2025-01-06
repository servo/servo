/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::{HandleObject, MutableHandleValue};
use servo_url::{ImmutableOrigin, ServoUrl};

use super::bindings::codegen::Bindings::NotificationBinding::{
    NotificationAction, NotificationOptions,
};
use super::bindings::reflector::reflect_dom_object_with_proto;
use super::bindings::str::USVString;
use crate::dom::bindings::codegen::Bindings::NotificationBinding::{
    NotificationDirection, NotificationMethods, NotificationPermission,
    NotificationPermissionCallback,
};
use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::{
    PermissionName, PermissionState,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::{Fallible, Rc};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::permissions::get_descriptor_permission_state;
use crate::dom::promise::Promise;
use crate::dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://notifications.spec.whatwg.org/#api>
#[dom_struct]
pub struct Notification {
    eventtarget: EventTarget,
    title: DOMString,
    body: DOMString,
    #[ignore_malloc_size_of = "mozjs"]
    data: Box<Heap<JSVal>>,
    dir: NotificationDirection,
    image: Option<USVString>,
    icon: Option<USVString>,
    badge: Option<USVString>,
    lang: DOMString,
    silent: bool,
    tag: DOMString,
    #[no_trace] // ImmutableOrigin is not traceable
    origin: ImmutableOrigin,
    // TODO: vibrate not implemented yet
    // vibrate: Option<UnionTypes::UnsignedLongOrUnsignedLongSequence>,
    renotify: bool,
    require_interaction: bool,
    #[ignore_malloc_size_of = "NotificationAction"] // malloc not implement for NotificationAction
    actions: Vec<NotificationAction>,
    closed: Cell<bool>,
}

impl Notification {
    pub fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        title: DOMString,
        options: RootedTraceableBox<NotificationOptions>,
        origin: Option<ImmutableOrigin>,
        base_url: Option<ServoUrl>,
    ) -> Fallible<DomRoot<Self>> {
        let notification = Notification::new_inherited(global, title, options, origin, base_url)?;
        Ok(reflect_dom_object_with_proto(
            Box::new(notification),
            global,
            proto,
            can_gc,
        ))
    }

    /// <https://notifications.spec.whatwg.org/#create-a-notification-with-a-settings-object>
    fn new_inherited(
        global: &GlobalScope,
        title: DOMString,
        options: RootedTraceableBox<NotificationOptions>,
        origin: Option<ImmutableOrigin>,
        base_url: Option<ServoUrl>,
    ) -> Fallible<Self> {
        if options.silent.is_some() && options.vibrate.is_some() {
            return Err(Error::Type(
                "silent and vibrate can not be set at the same time".to_string(),
            ));
        }
        if options.renotify && options.tag.is_empty() {
            return Err(Error::Type(
                "tag must be set if renotify is true".to_string(),
            ));
        }
        let data = Heap::boxed(options.data.get());
        let title = title.clone();
        let dir = options.dir;
        let lang = options.lang.clone();
        // FIXME: not sure which Origin should be used
        let origin = origin.unwrap_or(ImmutableOrigin::new_opaque());
        let body = options.body.clone();
        let tag = options.tag.clone();

        let image = options.image.as_ref().and_then(|image_url| {
            ServoUrl::parse_with_base(base_url.as_ref(), image_url)
                .ok()
                .map(|url| USVString::from(url.to_string()))
        });
        let icon = options.icon.as_ref().and_then(|icon_url| {
            ServoUrl::parse_with_base(base_url.as_ref(), icon_url)
                .ok()
                .map(|url| USVString::from(url.to_string()))
        });
        let badge = options.badge.as_ref().and_then(|badge_url| {
            ServoUrl::parse_with_base(base_url.as_ref(), badge_url)
                .ok()
                .map(|url| USVString::from(url.to_string()))
        });

        // TODO: vibrate not implemented yet
        // let vibrate = options.vibrate;

        let renotify = options.renotify;
        let silent = options.silent.unwrap_or(false);

        let require_interaction = options.requireInteraction;

        let mut actions: Vec<NotificationAction> = Vec::new();
        let max_actions = Notification::MaxActions(global);
        for action in &options.actions[0..max_actions as usize] {
            actions.push(NotificationAction {
                action: action.action.clone(), // Spec is using `name`, however in WebIDL it use `action`.
                title: action.title.clone(),
                icon: action.icon.clone(),
            });
        }

        Ok(Self {
            eventtarget: EventTarget::new_inherited(),
            title,
            body,
            data,
            dir,
            image,
            icon,
            badge,
            lang,
            silent,
            origin,
            renotify,
            tag,
            require_interaction,
            actions,
            closed: Cell::new(false),
        })
    }
}

impl NotificationMethods<crate::DomTypeHolder> for Notification {
    /// <https://notifications.spec.whatwg.org/#constructors>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        title: DOMString,
        options: RootedTraceableBox<NotificationOptions>,
    ) -> Fallible<DomRoot<Notification>> {
        // step 1: Check global is a ServiceWorkerGlobalScope
        if global.downcast::<ServiceWorkerGlobalScope>().is_some() {
            return Err(Error::Type("Can not call in a service worker.".to_string()));
        }

        // step 2: Check options.actions must be empty
        if !options.actions.is_empty() {
            return Err(Error::Type("Actions must be empty.".to_string()));
        }

        // step 3: Create a notification with a settings object
        // https://notifications.spec.whatwg.org/#create-a-notification-with-a-settings-object
        let notification = Notification::new(global, proto, can_gc, title, options, None, None)?;

        // step 5.1: Check permission
        let state = get_descriptor_permission_state(PermissionName::Notifications, Some(global));
        if state == PermissionState::Granted {
            // TODO: step 5.2: Run permission show step
            todo!()
        } else {
            notification
                .upcast::<EventTarget>()
                .fire_event(atom!("error"), CanGc::note());
        }

        Ok(notification)
    }

    /// <https://notifications.spec.whatwg.org/#dom-notification-permission>
    fn GetPermission(_global: &GlobalScope) -> Fallible<NotificationPermission> {
        todo!()
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-requestpermission>
    fn RequestPermission(
        _global: &GlobalScope,
        _permission_callback: Option<Rc<NotificationPermissionCallback>>,
    ) -> Rc<Promise> {
        todo!()
    }

    // <https://notifications.spec.whatwg.org/#dom-notification-onclick>
    event_handler!(click, GetOnclick, SetOnclick);
    // <https://notifications.spec.whatwg.org/#dom-notification-onshow>
    event_handler!(show, GetOnshow, SetOnshow);
    // <https://notifications.spec.whatwg.org/#dom-notification-onerror>
    event_handler!(error, GetOnerror, SetOnerror);
    // <https://notifications.spec.whatwg.org/#dom-notification-onclose>
    event_handler!(close, GetOnclose, SetOnclose);

    /// <https://notifications.spec.whatwg.org/#maximum-number-of-actions>
    fn MaxActions(_global: &GlobalScope) -> u32 {
        // TODO: determine the maximum number of actions
        2
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-title>
    fn Title(&self) -> DOMString {
        self.title.clone()
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-dir>
    fn Dir(&self) -> NotificationDirection {
        self.dir
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-lang>
    fn Lang(&self) -> DOMString {
        self.lang.clone()
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-body>
    fn Body(&self) -> DOMString {
        self.body.clone()
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-tag>
    fn Tag(&self) -> DOMString {
        self.tag.clone()
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-image>
    fn Image(&self) -> USVString {
        self.image.clone().unwrap_or_default()
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-icon>
    fn Icon(&self) -> USVString {
        self.icon.clone().unwrap_or_default()
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-badge>
    fn Badge(&self) -> USVString {
        self.badge.clone().unwrap_or_default()
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-renotify>
    fn Renotify(&self) -> bool {
        self.renotify
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-silent>
    fn Silent(&self) -> bool {
        self.silent
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-requireinteraction>
    fn RequireInteraction(&self) -> bool {
        self.require_interaction
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-data>
    #[allow(unsafe_code)]
    fn Data(&self, _cx: SafeJSContext, mut retval: MutableHandleValue) {
        // jason: do we need to use structuredclone here?
        // unsafe {
        //     let global = {
        //         let realm = AlreadyInRealm::assert_for_cx(cx);
        //         GlobalScope::from_context(*cx, InRealm::already(&realm))
        //     };

        //     let handle = HandleValue::from_raw(self.data.handle());
        //     let data = structuredclone::write(cx, handle, None).unwrap();

        //     if structuredclone::read(&global, data, retval).is_err() {
        //         dbg!("StructuredDeserialize failed");
        //     }
        // }
        retval.set(self.data.get());
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-actions>
    fn Actions(&self, cx: SafeJSContext, retval: MutableHandleValue) {
        to_frozen_array(self.actions.as_slice(), cx, retval);
    }
    /// <https://notifications.spec.whatwg.org/#close-steps>
    // TODO: close persistent notification
    fn Close(&self) {
        if self.closed.get() {
            return;
        }

        self.upcast::<EventTarget>()
            .fire_event(atom!("close"), CanGc::note());

        self.closed.set(true);
    }
}
