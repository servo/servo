/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


// This is a Servo-specific interface, used to represent locations
// that are not similar-origin, so live in another script thread.
// It is based on the interface for Window, but only contains the
// accessors that do not throw security exceptions when called
// cross-origin.
//
// Note that similar-origin locations are kept in the same script
// thread, so this mechanism cannot be relied upon as the only
// way to enforce security policy.

// https://html.spec.whatwg.org/multipage/#location
[Unforgeable, NoInterfaceObject] interface DissimilarOriginLocation {
  [Throws] attribute USVString href;
  [Throws] void assign(USVString url);
  [Throws] void replace(USVString url);
  [Throws] void reload();
  [Throws] stringifier;

  // TODO: finish this interface
};
