/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/permissions/#permissions-interface

[Pref="dom.permissions.enabled", Exposed=(Window,Worker)]
interface Permissions {
  Promise<PermissionStatus> query(object permissionDesc);

  Promise<PermissionStatus> request(object permissionDesc);

  Promise<PermissionStatus> revoke(object permissionDesc);
};
