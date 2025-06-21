/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use dom_struct::dom_struct;
use embedder_traits::{
    EmbedderMsg, Notification as EmbedderNotification,
    NotificationAction as EmbedderNotificationAction,
};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::{HandleObject, MutableHandleValue};
use net_traits::http_status::HttpStatus;
use net_traits::image_cache::{
    ImageCache, ImageCacheResponseMessage, ImageCacheResult, ImageLoadListener,
    ImageOrMetadataAvailable, ImageResponse, PendingImageId, UsePlaceholder,
};
use net_traits::request::{Destination, RequestBuilder, RequestId};
use net_traits::{
    FetchMetadata, FetchResponseListener, FetchResponseMsg, NetworkError, ResourceFetchTiming,
    ResourceTimingType,
};
use pixels::RasterImage;
use servo_url::{ImmutableOrigin, ServoUrl};
use uuid::Uuid;

use super::bindings::cell::DomRefCell;
use super::bindings::refcounted::{Trusted, TrustedPromise};
use super::bindings::reflector::DomGlobal;
use super::performanceresourcetiming::InitiatorType;
use super::permissionstatus::PermissionStatus;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::NotificationBinding::{
    NotificationAction, NotificationDirection, NotificationMethods, NotificationOptions,
    NotificationPermission, NotificationPermissionCallback,
};
use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::PermissionStatus_Binding::PermissionStatusMethods;
use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::{
    PermissionDescriptor, PermissionName, PermissionState,
};
use crate::dom::bindings::codegen::UnionTypes::UnsignedLongOrUnsignedLongSequence;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::csp::{Violation, report_csp_violations};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::permissions::{PermissionAlgorithm, Permissions, descriptor_permission_state};
use crate::dom::promise::Promise;
use crate::dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use crate::dom::serviceworkerregistration::ServiceWorkerRegistration;
use crate::fetch::create_a_potential_cors_request;
use crate::network_listener::{self, PreInvoke, ResourceTimingListener};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

// TODO: Service Worker API (persistent notification)
// https://notifications.spec.whatwg.org/#service-worker-api

/// <https://notifications.spec.whatwg.org/#notifications>
#[dom_struct]
pub(crate) struct Notification {
    eventtarget: EventTarget,
    /// <https://notifications.spec.whatwg.org/#service-worker-registration>
    serviceworker_registration: Option<Dom<ServiceWorkerRegistration>>,
    /// <https://notifications.spec.whatwg.org/#concept-title>
    title: DOMString,
    /// <https://notifications.spec.whatwg.org/#body>
    body: DOMString,
    /// <https://notifications.spec.whatwg.org/#data>
    #[ignore_malloc_size_of = "mozjs"]
    data: Heap<JSVal>,
    /// <https://notifications.spec.whatwg.org/#concept-direction>
    dir: NotificationDirection,
    /// <https://notifications.spec.whatwg.org/#image-url>
    image: Option<USVString>,
    /// <https://notifications.spec.whatwg.org/#icon-url>
    icon: Option<USVString>,
    /// <https://notifications.spec.whatwg.org/#badge-url>
    badge: Option<USVString>,
    /// <https://notifications.spec.whatwg.org/#concept-language>
    lang: DOMString,
    /// <https://notifications.spec.whatwg.org/#silent-preference-flag>
    silent: Option<bool>,
    /// <https://notifications.spec.whatwg.org/#tag>
    tag: DOMString,
    /// <https://notifications.spec.whatwg.org/#concept-origin>
    #[no_trace] // ImmutableOrigin is not traceable
    origin: ImmutableOrigin,
    /// <https://notifications.spec.whatwg.org/#vibration-pattern>
    vibration_pattern: Vec<u32>,
    /// <https://notifications.spec.whatwg.org/#timestamp>
    timestamp: u64,
    /// <https://notifications.spec.whatwg.org/#renotify-preference-flag>
    renotify: bool,
    /// <https://notifications.spec.whatwg.org/#require-interaction-preference-flag>
    require_interaction: bool,
    /// <https://notifications.spec.whatwg.org/#actions>
    actions: Vec<Action>,
    /// Pending image, icon, badge, action icon resource request's id
    #[no_trace] // RequestId is not traceable
    pending_request_ids: DomRefCell<HashSet<RequestId>>,
    /// <https://notifications.spec.whatwg.org/#image-resource>
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    image_resource: DomRefCell<Option<Arc<RasterImage>>>,
    /// <https://notifications.spec.whatwg.org/#icon-resource>
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    icon_resource: DomRefCell<Option<Arc<RasterImage>>>,
    /// <https://notifications.spec.whatwg.org/#badge-resource>
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    badge_resource: DomRefCell<Option<Arc<RasterImage>>>,
}

impl Notification {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        global: &GlobalScope,
        title: DOMString,
        options: RootedTraceableBox<NotificationOptions>,
        origin: ImmutableOrigin,
        base_url: ServoUrl,
        fallback_timestamp: u64,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let notification = reflect_dom_object_with_proto(
            Box::new(Notification::new_inherited(
                global,
                title,
                &options,
                origin,
                base_url,
                fallback_timestamp,
            )),
            global,
            proto,
            can_gc,
        );

        notification.data.set(options.data.get());

        notification
    }

    /// partial implementation of <https://notifications.spec.whatwg.org/#create-a-notification>
    fn new_inherited(
        global: &GlobalScope,
        title: DOMString,
        options: &RootedTraceableBox<NotificationOptions>,
        origin: ImmutableOrigin,
        base_url: ServoUrl,
        fallback_timestamp: u64,
    ) -> Self {
        // TODO: missing call to https://html.spec.whatwg.org/multipage/#structuredserializeforstorage
        // may be find in `dom/bindings/structuredclone.rs`
        let data = Heap::default();

        let title = title.clone();
        let dir = options.dir;
        let lang = options.lang.clone();
        let body = options.body.clone();
        let tag = options.tag.clone();

        // If options["image"] exists, then parse it using baseURL, and if that does not return failure,
        // set notification’s image URL to the return value. (Otherwise notification’s image URL is not set.)
        let image = options.image.as_ref().and_then(|image_url| {
            ServoUrl::parse_with_base(Some(&base_url), image_url.as_ref())
                .map(|url| USVString::from(url.to_string()))
                .ok()
        });
        // If options["icon"] exists, then parse it using baseURL, and if that does not return failure,
        // set notification’s icon URL to the return value. (Otherwise notification’s icon URL is not set.)
        let icon = options.icon.as_ref().and_then(|icon_url| {
            ServoUrl::parse_with_base(Some(&base_url), icon_url.as_ref())
                .map(|url| USVString::from(url.to_string()))
                .ok()
        });
        // If options["badge"] exists, then parse it using baseURL, and if that does not return failure,
        // set notification’s badge URL to the return value. (Otherwise notification’s badge URL is not set.)
        let badge = options.badge.as_ref().and_then(|badge_url| {
            ServoUrl::parse_with_base(Some(&base_url), badge_url.as_ref())
                .map(|url| USVString::from(url.to_string()))
                .ok()
        });
        // If options["vibrate"] exists, then validate and normalize it and
        // set notification’s vibration pattern to the return value.
        let vibration_pattern = match &options.vibrate {
            Some(pattern) => validate_and_normalize_vibration_pattern(pattern),
            None => Vec::new(),
        };
        // If options["timestamp"] exists, then set notification’s timestamp to the value.
        // Otherwise, set notification’s timestamp to fallbackTimestamp.
        let timestamp = options.timestamp.unwrap_or(fallback_timestamp);
        let renotify = options.renotify;
        let silent = options.silent;
        let require_interaction = options.requireInteraction;

        // For each entry in options["actions"]
        // up to the maximum number of actions supported (skip any excess entries):
        let mut actions: Vec<Action> = Vec::new();
        let max_actions = Notification::MaxActions(global);
        for action in options.actions.iter().take(max_actions as usize) {
            actions.push(Action {
                id: Uuid::new_v4().simple().to_string(),
                name: action.action.clone(),
                title: action.title.clone(),
                // If entry["icon"] exists, then parse it using baseURL, and if that does not return failure
                // set action’s icon URL to the return value. (Otherwise action’s icon URL remains null.)
                icon_url: action.icon.as_ref().and_then(|icon_url| {
                    ServoUrl::parse_with_base(Some(&base_url), icon_url.as_ref())
                        .map(|url| USVString::from(url.to_string()))
                        .ok()
                }),
                icon_resource: DomRefCell::new(None),
            });
        }

        Self {
            eventtarget: EventTarget::new_inherited(),
            // A non-persistent notification is a notification whose service worker registration is null.
            serviceworker_registration: None,
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
            vibration_pattern,
            timestamp,
            renotify,
            tag,
            require_interaction,
            actions,
            pending_request_ids: DomRefCell::new(HashSet::new()),
            image_resource: DomRefCell::new(None),
            icon_resource: DomRefCell::new(None),
            badge_resource: DomRefCell::new(None),
        }
    }

    /// <https://notifications.spec.whatwg.org/#notification-show-steps>
    fn show(&self) {
        // step 3: set shown to false
        let shown = false;

        // TODO: step 4: Let oldNotification be the notification in the list of notifications
        //               whose tag is not the empty string and is notification’s tag,
        //               and whose origin is same origin with notification’s origin,
        //               if any, and null otherwise.

        // TODO: step 5: If oldNotification is non-null, then:
        // TODO:   step 5.1: Handle close events with oldNotification.
        // TODO:   step 5.2: If the notification platform supports replacement, then:
        // TODO:     step 5.2.1: Replace oldNotification with notification, in the list of notifications.
        // TODO:     step 5.2.2: Set shown to true.
        // TODO:   step 5.3: Otherwise, remove oldNotification from the list of notifications.

        // step 6: If shown is false, then:
        if !shown {
            // TODO: step 6.1: Append notification to the list of notifications.
            // step 6.2: Display notification on the device
            self.global()
                .send_to_embedder(EmbedderMsg::ShowNotification(
                    self.global().webview_id(),
                    self.to_embedder_notification(),
                ));
        }

        // TODO: step 7: If shown is false or oldNotification is non-null,
        //               and notification’s renotify preference is true,
        //               then run the alert steps for notification.

        // step 8: If notification is a non-persistent notification,
        //         then queue a task to fire an event named show on
        //         the Notification object representing notification.
        if self.serviceworker_registration.is_none() {
            self.global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue_simple_event(self.upcast(), atom!("show"));
        }
    }

    /// Create an [`embedder_traits::Notification`].
    fn to_embedder_notification(&self) -> EmbedderNotification {
        EmbedderNotification {
            title: self.title.to_string(),
            body: self.body.to_string(),
            tag: self.tag.to_string(),
            language: self.lang.to_string(),
            require_interaction: self.require_interaction,
            silent: self.silent,
            icon_url: self
                .icon
                .as_ref()
                .and_then(|icon| ServoUrl::parse(icon).ok()),
            badge_url: self
                .badge
                .as_ref()
                .and_then(|badge| ServoUrl::parse(badge).ok()),
            image_url: self
                .image
                .as_ref()
                .and_then(|image| ServoUrl::parse(image).ok()),
            actions: self
                .actions
                .iter()
                .map(|action| EmbedderNotificationAction {
                    name: action.name.to_string(),
                    title: action.title.to_string(),
                    icon_url: action
                        .icon_url
                        .as_ref()
                        .and_then(|icon| ServoUrl::parse(icon).ok()),
                    icon_resource: action.icon_resource.borrow().clone(),
                })
                .collect(),
            icon_resource: self.icon_resource.borrow().clone(),
            badge_resource: self.badge_resource.borrow().clone(),
            image_resource: self.image_resource.borrow().clone(),
        }
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
        if global.is::<ServiceWorkerGlobalScope>() {
            return Err(Error::Type(
                "Notification constructor cannot be used in service worker.".to_string(),
            ));
        }

        // step 2: Check options.actions must be empty
        if !options.actions.is_empty() {
            return Err(Error::Type(
                "Actions are only supported for persistent notifications.".to_string(),
            ));
        }

        // step 3: Create a notification with a settings object
        let notification =
            create_notification_with_settings_object(global, title, options, proto, can_gc)?;

        // TODO: Run step 5.1, 5.2 in parallel
        // step 5.1: If the result of getting the notifications permission state is not "granted",
        //           then queue a task to fire an event named error on this, and abort these steps.
        let permission_state = get_notifications_permission_state(global);
        if permission_state != NotificationPermission::Granted {
            global
                .task_manager()
                .dom_manipulation_task_source()
                .queue_simple_event(notification.upcast(), atom!("error"));
            // TODO: abort steps
        } else {
            // step 5.2: Run the notification show steps for notification
            // <https://notifications.spec.whatwg.org/#notification-show-steps>
            // step 1: Run the fetch steps for notification.
            notification.fetch_resources_and_show_when_ready();
        }

        Ok(notification)
    }

    /// <https://notifications.spec.whatwg.org/#dom-notification-permission>
    fn GetPermission(global: &GlobalScope) -> Fallible<NotificationPermission> {
        Ok(get_notifications_permission_state(global))
    }

    /// <https://notifications.spec.whatwg.org/#dom-notification-requestpermission>
    fn RequestPermission(
        global: &GlobalScope,
        permission_callback: Option<Rc<NotificationPermissionCallback>>,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 2: Let promise be a new promise in this’s relevant Realm.
        let promise = Promise::new(global, can_gc);

        // TODO: Step 3: Run these steps in parallel:
        // Step 3.1: Let permissionState be the result of requesting permission to use "notifications".
        let notification_permission = request_notification_permission(global, can_gc);

        // Step 3.2: Queue a global task on the DOM manipulation task source given global to run these steps:
        let trusted_promise = TrustedPromise::new(promise.clone());
        let uuid = Uuid::new_v4().simple().to_string();
        let uuid_ = uuid.clone();

        if let Some(callback) = permission_callback {
            global.add_notification_permission_request_callback(uuid.clone(), callback.clone());
        }

        global.task_manager().dom_manipulation_task_source().queue(
            task!(request_permission: move || {
                let promise = trusted_promise.root();
                let global = promise.global();

                // Step 3.2.1: If deprecatedCallback is given,
                //             then invoke deprecatedCallback with « permissionState » and "report".
                if let Some(callback) = global.remove_notification_permission_request_callback(uuid_) {
                    let _ = callback.Call__(notification_permission, ExceptionHandling::Report, CanGc::note());
                }

                // Step 3.2.2: Resolve promise with permissionState.
                promise.resolve_native(&notification_permission, CanGc::note());
            }),
        );

        promise
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
        // step 1: If there is no this’s notification’s image URL, then return the empty string.
        // step 2: Return this’s notification’s image URL, serialized.
        self.image.clone().unwrap_or_default()
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-icon>
    fn Icon(&self) -> USVString {
        // step 1: If there is no this’s notification’s icon URL, then return the empty string.
        // step 2: Return this’s notification’s icon URL, serialized.
        self.icon.clone().unwrap_or_default()
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-badge>
    fn Badge(&self) -> USVString {
        // step 1: If there is no this’s notification’s badge URL, then return the empty string.
        // step 2: Return this’s notification’s badge URL, serialized.
        self.badge.clone().unwrap_or_default()
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-renotify>
    fn Renotify(&self) -> bool {
        self.renotify
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-silent>
    fn GetSilent(&self) -> Option<bool> {
        self.silent
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-requireinteraction>
    fn RequireInteraction(&self) -> bool {
        self.require_interaction
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-data>
    fn Data(&self, _cx: SafeJSContext, mut retval: MutableHandleValue) {
        retval.set(self.data.get());
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-actions>
    fn Actions(&self, cx: SafeJSContext, can_gc: CanGc, retval: MutableHandleValue) {
        // step 1: Let frozenActions be an empty list of type NotificationAction.
        let mut frozen_actions: Vec<NotificationAction> = Vec::new();

        // step 2: For each entry of this’s notification’s actions
        for action in self.actions.iter() {
            let action = NotificationAction {
                action: action.name.clone(),
                title: action.title.clone(),
                // If entry’s icon URL is non-null,
                // then set action["icon"] to entry’s icon URL, icon_url, serialized.
                icon: action.icon_url.clone(),
            };

            // TODO: step 2.5: Call Object.freeze on action, to prevent accidental mutation by scripts.
            // step 2.6: Append action to frozenActions.
            frozen_actions.push(action);
        }

        // step 3: Return the result of create a frozen array from frozenActions.
        to_frozen_array(frozen_actions.as_slice(), cx, retval, can_gc);
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-vibrate>
    fn Vibrate(&self, cx: SafeJSContext, can_gc: CanGc, retval: MutableHandleValue) {
        to_frozen_array(self.vibration_pattern.as_slice(), cx, retval, can_gc);
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-timestamp>
    fn Timestamp(&self) -> u64 {
        self.timestamp
    }
    /// <https://notifications.spec.whatwg.org/#dom-notification-close>
    fn Close(&self) {
        // TODO: If notification is a persistent notification and notification was closed by the end user
        // then fire a service worker notification event named "notificationclose" given notification.

        // If notification is a non-persistent notification
        // then queue a task to fire an event named close on the Notification object representing notification.
        if self.serviceworker_registration.is_none() {
            self.global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue_simple_event(self.upcast(), atom!("close"));
        }
    }
}

/// <https://notifications.spec.whatwg.org/#actions>
#[derive(JSTraceable, MallocSizeOf)]
struct Action {
    id: String,
    /// <https://notifications.spec.whatwg.org/#action-name>
    name: DOMString,
    /// <https://notifications.spec.whatwg.org/#action-title>
    title: DOMString,
    /// <https://notifications.spec.whatwg.org/#action-icon-url>
    icon_url: Option<USVString>,
    /// <https://notifications.spec.whatwg.org/#action-icon-resource>
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    icon_resource: DomRefCell<Option<Arc<RasterImage>>>,
}

/// <https://notifications.spec.whatwg.org/#create-a-notification-with-a-settings-object>
fn create_notification_with_settings_object(
    global: &GlobalScope,
    title: DOMString,
    options: RootedTraceableBox<NotificationOptions>,
    proto: Option<HandleObject>,
    can_gc: CanGc,
) -> Fallible<DomRoot<Notification>> {
    // step 1: Let origin be settings’s origin.
    let origin = global.origin().immutable().clone();
    // step 2: Let baseURL be settings’s API base URL.
    let base_url = global.api_base_url();
    // step 3: Let fallbackTimestamp be the number of milliseconds from
    //         the Unix epoch to settings’s current wall time, rounded to the nearest integer.
    let fallback_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    // step 4: Return the result of creating a notification given title, options, origin,
    //         baseURL, and fallbackTimestamp.
    create_notification(
        global,
        title,
        options,
        origin,
        base_url,
        fallback_timestamp,
        proto,
        can_gc,
    )
}

/// <https://notifications.spec.whatwg.org/#create-a-notification
#[allow(clippy::too_many_arguments)]
fn create_notification(
    global: &GlobalScope,
    title: DOMString,
    options: RootedTraceableBox<NotificationOptions>,
    origin: ImmutableOrigin,
    base_url: ServoUrl,
    fallback_timestamp: u64,
    proto: Option<HandleObject>,
    can_gc: CanGc,
) -> Fallible<DomRoot<Notification>> {
    // If options["silent"] is true and options["vibrate"] exists, then throw a TypeError.
    if options.silent.is_some() && options.vibrate.is_some() {
        return Err(Error::Type(
            "Can't specify vibration patterns when setting notification to silent.".to_string(),
        ));
    }
    // If options["renotify"] is true and options["tag"] is the empty string, then throw a TypeError.
    if options.renotify && options.tag.is_empty() {
        return Err(Error::Type(
            "tag must be set to renotify as an existing notification.".to_string(),
        ));
    }

    Ok(Notification::new(
        global,
        title,
        options,
        origin,
        base_url,
        fallback_timestamp,
        proto,
        can_gc,
    ))
}

/// <https://w3c.github.io/vibration/#dfn-validate-and-normalize>
fn validate_and_normalize_vibration_pattern(
    pattern: &UnsignedLongOrUnsignedLongSequence,
) -> Vec<u32> {
    // Step 1: If pattern is a list, proceed to the next step. Otherwise run the following substeps:
    let mut pattern: Vec<u32> = match pattern {
        UnsignedLongOrUnsignedLongSequence::UnsignedLong(value) => {
            // Step 1.1: Let list be an initially empty list, and add pattern to list.
            // Step 1.2: Set pattern to list.
            vec![*value]
        },
        UnsignedLongOrUnsignedLongSequence::UnsignedLongSequence(values) => values.clone(),
    };

    // Step 2: Let max length have the value 10.
    // Step 3: If the length of pattern is greater than max length, truncate pattern,
    //         leaving only the first max length entries.
    pattern.truncate(10);

    // If the length of the pattern is even and not zero then the last entry in the pattern will
    // have no effect so an implementation can remove it from the pattern at this point.
    if pattern.len() % 2 == 0 && !pattern.is_empty() {
        pattern.pop();
    }

    // Step 4: Let max duration have the value 10000.
    // Step 5: For each entry in pattern whose value is greater than max duration,
    //         set the entry's value to max duration.
    pattern.iter_mut().for_each(|entry| {
        *entry = 10000.min(*entry);
    });

    // Step 6: Return pattern.
    pattern
}

/// <https://notifications.spec.whatwg.org/#get-the-notifications-permission-state>
fn get_notifications_permission_state(global: &GlobalScope) -> NotificationPermission {
    let permission_state = descriptor_permission_state(PermissionName::Notifications, Some(global));
    match permission_state {
        PermissionState::Granted => NotificationPermission::Granted,
        PermissionState::Denied => NotificationPermission::Denied,
        PermissionState::Prompt => NotificationPermission::Default,
    }
}

fn request_notification_permission(global: &GlobalScope, can_gc: CanGc) -> NotificationPermission {
    let cx = GlobalScope::get_cx();
    let promise = &Promise::new(global, can_gc);
    let descriptor = PermissionDescriptor {
        name: PermissionName::Notifications,
    };
    let status = PermissionStatus::new(global, &descriptor, can_gc);

    // The implementation of `request_notification_permission` seemed to be synchronous
    Permissions::permission_request(cx, promise, &descriptor, &status);

    match status.State() {
        PermissionState::Granted => NotificationPermission::Granted,
        PermissionState::Denied => NotificationPermission::Denied,
        // Should only receive "Granted" or "Denied" from the permission request
        PermissionState::Prompt => NotificationPermission::Default,
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum ResourceType {
    Image,
    Icon,
    Badge,
    ActionIcon(String), // action id
}

struct ResourceFetchListener {
    /// The ID of the pending image cache for this request.
    pending_image_id: PendingImageId,
    /// A reference to the global image cache.
    image_cache: Arc<dyn ImageCache>,
    /// The notification instance which makes this request.
    notification: Trusted<Notification>,
    /// Request status that indicates whether this request failed, and the reason.
    status: Result<(), NetworkError>,
    /// Resource URL of this request.
    url: ServoUrl,
    /// Timing data for this resource.
    resource_timing: ResourceFetchTiming,
}

impl FetchResponseListener for ResourceFetchListener {
    fn process_request_body(&mut self, _: RequestId) {}
    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(
        &mut self,
        request_id: RequestId,
        metadata: Result<FetchMetadata, NetworkError>,
    ) {
        self.image_cache.notify_pending_response(
            self.pending_image_id,
            FetchResponseMsg::ProcessResponse(request_id, metadata.clone()),
        );

        let metadata = metadata.ok().map(|meta| match meta {
            FetchMetadata::Unfiltered(m) => m,
            FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
        });

        let status = metadata
            .as_ref()
            .map(|m| m.status.clone())
            .unwrap_or_else(HttpStatus::new_error);

        self.status = {
            if status.is_success() {
                Ok(())
            } else if status.is_error() {
                Err(NetworkError::Internal(
                    "No http status code received".to_owned(),
                ))
            } else {
                Err(NetworkError::Internal(format!(
                    "HTTP error code {}",
                    status.code()
                )))
            }
        };
    }

    fn process_response_chunk(&mut self, request_id: RequestId, payload: Vec<u8>) {
        if self.status.is_ok() {
            self.image_cache.notify_pending_response(
                self.pending_image_id,
                FetchResponseMsg::ProcessResponseChunk(request_id, payload),
            );
        }
    }

    fn process_response_eof(
        &mut self,
        request_id: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        self.image_cache.notify_pending_response(
            self.pending_image_id,
            FetchResponseMsg::ProcessResponseEOF(request_id, response),
        );
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        network_listener::submit_timing(self, CanGc::note())
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        report_csp_violations(global, violations, None);
    }
}

impl ResourceTimingListener for ResourceFetchListener {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (InitiatorType::Other, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.notification.root().global()
    }
}

impl PreInvoke for ResourceFetchListener {
    fn should_invoke(&self) -> bool {
        true
    }
}

impl Notification {
    fn build_resource_request(&self, url: &ServoUrl) -> RequestBuilder {
        let global = &self.global();
        create_a_potential_cors_request(
            None,
            url.clone(),
            Destination::Image,
            None, // TODO: check which CORS should be used
            None,
            global.get_referrer(),
            global.insecure_requests_policy(),
            global.has_trustworthy_ancestor_or_current_origin(),
            global.policy_container(),
        )
        .origin(global.origin().immutable().clone())
        .pipeline_id(Some(global.pipeline_id()))
    }

    /// <https://notifications.spec.whatwg.org/#fetch-steps>
    fn fetch_resources_and_show_when_ready(&self) {
        let mut pending_requests: Vec<(RequestBuilder, ResourceType)> = vec![];
        if let Some(image_url) = &self.image {
            if let Ok(url) = ServoUrl::parse(image_url) {
                let request = self.build_resource_request(&url);
                self.pending_request_ids.borrow_mut().insert(request.id);
                pending_requests.push((request, ResourceType::Image));
            }
        }
        if let Some(icon_url) = &self.icon {
            if let Ok(url) = ServoUrl::parse(icon_url) {
                let request = self.build_resource_request(&url);
                self.pending_request_ids.borrow_mut().insert(request.id);
                pending_requests.push((request, ResourceType::Icon));
            }
        }
        if let Some(badge_url) = &self.badge {
            if let Ok(url) = ServoUrl::parse(badge_url) {
                let request = self.build_resource_request(&url);
                self.pending_request_ids.borrow_mut().insert(request.id);
                pending_requests.push((request, ResourceType::Badge));
            }
        }
        for action in self.actions.iter() {
            if let Some(icon_url) = &action.icon_url {
                if let Ok(url) = ServoUrl::parse(icon_url) {
                    let request = self.build_resource_request(&url);
                    self.pending_request_ids.borrow_mut().insert(request.id);
                    pending_requests.push((request, ResourceType::ActionIcon(action.id.clone())));
                }
            }
        }

        for (request, resource_type) in pending_requests {
            self.fetch_and_show_when_ready(request, resource_type);
        }
    }

    fn fetch_and_show_when_ready(&self, request: RequestBuilder, resource_type: ResourceType) {
        let global: &GlobalScope = &self.global();
        let request_id = request.id;

        let cache_result = global.image_cache().get_cached_image_status(
            request.url.clone(),
            global.origin().immutable().clone(),
            None, // TODO: check which CORS should be used
            UsePlaceholder::No,
        );
        match cache_result {
            ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable {
                image, ..
            }) => {
                let image = image.as_raster_image();
                if image.is_none() {
                    warn!("Vector images are not supported in notifications yet");
                };
                self.set_resource_and_show_when_ready(request_id, &resource_type, image);
            },
            ImageCacheResult::Available(ImageOrMetadataAvailable::MetadataAvailable(
                _,
                pending_image_id,
            )) => {
                self.register_image_cache_callback(
                    request_id,
                    pending_image_id,
                    resource_type.clone(),
                );
            },
            ImageCacheResult::Pending(pending_image_id) => {
                self.register_image_cache_callback(
                    request_id,
                    pending_image_id,
                    resource_type.clone(),
                );
            },
            ImageCacheResult::ReadyForRequest(pending_image_id) => {
                self.register_image_cache_callback(
                    request_id,
                    pending_image_id,
                    resource_type.clone(),
                );
                self.fetch(pending_image_id, request, global);
            },
            ImageCacheResult::LoadError => {
                self.set_resource_and_show_when_ready(request_id, &resource_type, None);
            },
        };
    }

    fn register_image_cache_callback(
        &self,
        request_id: RequestId,
        pending_image_id: PendingImageId,
        resource_type: ResourceType,
    ) {
        let (sender, receiver) = ipc::channel().expect("ipc channel failure");

        let global: &GlobalScope = &self.global();

        let trusted_this = Trusted::new(self);
        let resource_type = resource_type.clone();
        let task_source = global.task_manager().networking_task_source().to_sendable();

        ROUTER.add_typed_route(
            receiver,
            Box::new(move |response| {
                let trusted_this = trusted_this.clone();
                let resource_type = resource_type.clone();
                task_source.queue(task!(handle_response: move || {
                    let this = trusted_this.root();
                    if let Ok(response) = response {
                        let ImageCacheResponseMessage::NotifyPendingImageLoadStatus(status) = response else {
                            warn!("Received unexpected message from image cache: {response:?}");
                            return;
                        };
                        this.handle_image_cache_response(request_id, status.response, resource_type);
                    } else {
                        this.handle_image_cache_response(request_id, ImageResponse::None, resource_type);
                    }
                }));
            }),
        );

        global.image_cache().add_listener(ImageLoadListener::new(
            sender,
            global.pipeline_id(),
            pending_image_id,
        ));
    }

    fn handle_image_cache_response(
        &self,
        request_id: RequestId,
        response: ImageResponse,
        resource_type: ResourceType,
    ) {
        match response {
            ImageResponse::Loaded(image, _) => {
                let image = image.as_raster_image();
                if image.is_none() {
                    warn!("Vector images are not yet supported in notification attribute");
                };
                self.set_resource_and_show_when_ready(request_id, &resource_type, image);
            },
            ImageResponse::PlaceholderLoaded(image, _) => {
                self.set_resource_and_show_when_ready(request_id, &resource_type, Some(image));
            },
            ImageResponse::None => {
                self.set_resource_and_show_when_ready(request_id, &resource_type, None);
            },
            _ => (),
        };
    }

    fn set_resource_and_show_when_ready(
        &self,
        request_id: RequestId,
        resource_type: &ResourceType,
        image: Option<Arc<RasterImage>>,
    ) {
        match resource_type {
            ResourceType::Image => {
                *self.image_resource.borrow_mut() = image;
            },
            ResourceType::Icon => {
                *self.icon_resource.borrow_mut() = image;
            },
            ResourceType::Badge => {
                *self.badge_resource.borrow_mut() = image;
            },
            ResourceType::ActionIcon(id) => {
                if let Some(action) = self.actions.iter().find(|&action| *action.id == *id) {
                    *action.icon_resource.borrow_mut() = image;
                }
            },
        }

        let mut pending_requests_id = self.pending_request_ids.borrow_mut();
        pending_requests_id.remove(&request_id);

        // <https://notifications.spec.whatwg.org/#notification-show-steps>
        // step 2: Wait for any fetches to complete and notification’s resources to be set
        if pending_requests_id.is_empty() {
            self.show();
        }
    }

    fn fetch(
        &self,
        pending_image_id: PendingImageId,
        request: RequestBuilder,
        global: &GlobalScope,
    ) {
        let context = Arc::new(Mutex::new(ResourceFetchListener {
            pending_image_id,
            image_cache: global.image_cache(),
            notification: Trusted::new(self),
            url: request.url.clone(),
            status: Ok(()),
            resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
        }));

        global.fetch(
            request,
            context,
            global.task_manager().networking_task_source().into(),
        );
    }
}
