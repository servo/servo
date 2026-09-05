/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
[Global=DissimilarOriginWindow, Exposed=(Window,DissimilarOriginWindow), LegacyNoInterfaceObject]
interface DissimilarOriginWindow : GlobalScope {
  [LegacyUnforgeable, CrossOriginReadable] readonly attribute WindowProxy window;
  [BinaryName="Self_", Replaceable, CrossOriginReadable] readonly attribute WindowProxy self;
  [Replaceable, CrossOriginReadable] readonly attribute WindowProxy? parent;
  [LegacyUnforgeable, CrossOriginReadable] readonly attribute WindowProxy? top;
  [Replaceable, CrossOriginReadable] readonly attribute WindowProxy frames;
  [Replaceable, CrossOriginReadable] readonly attribute unsigned long length;
  [PutForwards=href, LegacyUnforgeable, CrossOriginReadable, CrossOriginWritable]
    readonly attribute DissimilarOriginLocation location;

  [CrossOriginCallable] undefined close();
  [CrossOriginReadable] readonly attribute boolean closed;
  [Throws, CrossOriginCallable]
  undefined postMessage(any message, USVString targetOrigin, optional sequence<object> transfer = []);
  [Throws, CrossOriginCallable]
  undefined postMessage(any message, optional WindowPostMessageOptions options = {});
  [CrossOriginReadable] attribute any opener;
  [CrossOriginCallable] undefined focus();
  [CrossOriginCallable] undefined blur();
};
