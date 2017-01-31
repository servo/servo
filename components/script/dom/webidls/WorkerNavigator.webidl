/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#workernavigator
[Exposed=Worker]
interface WorkerNavigator {};
WorkerNavigator implements NavigatorID;
WorkerNavigator implements NavigatorLanguage;
//WorkerNavigator implements NavigatorOnLine;

// https://w3c.github.io/permissions/#navigator-and-workernavigator-extension

[Exposed=(Worker)]
partial interface WorkerNavigator {
  [Pref="dom.permissions.enabled"] readonly attribute Permissions permissions;
};
