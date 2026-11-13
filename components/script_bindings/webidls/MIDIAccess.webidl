/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_WEBMIDI

/*
 * The origin of this IDL file is:
 * https://webaudio.github.io/web-midi-api/#MIDIAccess
 *
 * XXXmsub2 Holding off on adding Transferable,
 * as the spec text is currently underspecified
 * compared to every other API with Transferable interfaces
 */

[Pref="dom_webmidi_enabled", SecureContext, Exposed=(Window,Worker)]/* Transferable */
interface MIDIAccess: EventTarget {
  readonly attribute MIDIInputMap inputs;
  readonly attribute MIDIOutputMap outputs;
  attribute EventHandler onstatechange;
  readonly attribute boolean sysexEnabled;
};