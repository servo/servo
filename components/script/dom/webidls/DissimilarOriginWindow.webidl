/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This is a Servo-specific interface, used to represent windows
// that are not similar-origin, so live in another script thread.
// It is based on the interface for Window, but only contains the
// accessors that do not throw security exceptions when called
// cross-origin.
//
// Note that similar-origin windows are kept in the same script
// thread, so this mechanism cannot be relied upon as the only
// way to enforce security policy.

// https://html.spec.whatwg.org/multipage/#window
[Global, NoInterfaceObject]
interface DissimilarOriginWindow : GlobalScope {
  [Unforgeable] readonly attribute WindowProxy window;
  [BinaryName="Self_", Replaceable] readonly attribute WindowProxy self;
  [Unforgeable] readonly attribute WindowProxy? parent;
  [Unforgeable] readonly attribute WindowProxy? top;
  [Replaceable] readonly attribute WindowProxy frames;
  [Replaceable] readonly attribute unsigned long length;
  [Unforgeable] readonly attribute DissimilarOriginLocation location;

  void close();
  readonly attribute boolean closed;
  [Throws] void postMessage(any message, DOMString targetOrigin);
  attribute any opener;
  void blur();
  void focus();
};
