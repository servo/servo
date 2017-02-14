/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

 // https://w3c.github.io/permissions/#permissionstatus

dictionary PermissionDescriptor {
  required PermissionName name;
};

enum PermissionState {
  "granted",
  "denied",
  "prompt",
};

enum PermissionName {
  "geolocation",
  "notifications",
  "push",
  "midi",
  "camera",
  "microphone",
  "speaker",
  "device-info",
  "background-sync",
  "bluetooth",
  "persistent-storage",
};

[Pref="dom.permissions.enabled", Exposed=(Window,Worker)]
interface PermissionStatus : EventTarget {
  readonly attribute PermissionState state;
  attribute EventHandler onchange;
};

dictionary PushPermissionDescriptor : PermissionDescriptor {
  boolean userVisibleOnly = false;
};

dictionary MidiPermissionDescriptor : PermissionDescriptor {
  boolean sysex = false;
};

dictionary DevicePermissionDescriptor : PermissionDescriptor {
  DOMString deviceId;
};
