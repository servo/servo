/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/screen-wake-lock/

[SecureContext]
partial interface Navigator {
  [SameObject, Pref="dom_wakelock_enabled"] readonly attribute WakeLock wakeLock;
};

enum WakeLockType { "screen" };

[SecureContext, Exposed=(Window), Pref="dom_wakelock_enabled"]
interface WakeLock {
  Promise<WakeLockSentinel> request(optional WakeLockType type = "screen");
};

[SecureContext, Exposed=(Window), Pref="dom_wakelock_enabled"]
interface WakeLockSentinel : EventTarget {
  readonly attribute boolean released;
  readonly attribute WakeLockType type;

  // Promise<void> release();
  // attribute EventHandler onrelease;
};
