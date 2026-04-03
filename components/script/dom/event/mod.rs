/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) use self::event::*;
pub(crate) mod animationevent;
pub(crate) mod beforeunloadevent;
pub(crate) mod closeevent;
pub(crate) mod commandevent;
pub(crate) mod compositionevent;
pub(crate) mod customevent;
pub(crate) mod errorevent;
#[allow(clippy::module_inception, reason = "The interface name is Event")]
pub(crate) mod event;
pub(crate) mod eventtarget;
pub(crate) mod extendableevent;
pub(crate) mod extendablemessageevent;
pub(crate) mod focusevent;
pub(crate) mod formdataevent;
pub(crate) mod hashchangeevent;
pub(crate) mod inputevent;
pub(crate) mod keyboardevent;
pub(crate) mod messageevent;
pub(crate) mod mouseevent;
pub(crate) mod pagetransitionevent;
#[expect(dead_code)]
pub(crate) mod pointerevent;
pub(crate) mod popstateevent;
pub(crate) mod progressevent;
pub(crate) mod promiserejectionevent;
pub(crate) mod storageevent;
pub(crate) mod submitevent;
pub(crate) mod toggleevent;
pub(crate) mod touchevent;
pub(crate) mod transitionevent;
pub(crate) mod uievent;
pub(crate) mod wheelevent;
