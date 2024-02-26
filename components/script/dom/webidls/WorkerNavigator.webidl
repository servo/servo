/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#workernavigator
[Exposed=Worker]
interface WorkerNavigator {};
WorkerNavigator includes NavigatorID;
WorkerNavigator includes NavigatorLanguage;
//WorkerNavigator includes NavigatorOnLine;
WorkerNavigator includes NavigatorConcurrentHardware;

// https://w3c.github.io/permissions/#navigator-and-workernavigator-extension

[Exposed=(Worker)]
partial interface WorkerNavigator {
  [Pref="dom.permissions.enabled"] readonly attribute Permissions permissions;
};

[Exposed=DedicatedWorker]
partial interface WorkerNavigator {
    [SameObject, Pref="dom.webgpu.enabled"] readonly attribute GPU gpu;
};
