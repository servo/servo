/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_WEBMIDI

/*
 * The origin of this IDL file is:
 * https://webaudio.github.io/web-midi-api/#MIDIMessageEvent
 */

[Pref="dom_webmidi_enabled", SecureContext, Exposed=(Window,Worker)]
interface MIDIMessageEvent : Event {
  constructor(DOMString type, optional MIDIMessageEventInit eventInitDict = {});
  readonly attribute Uint8Array? data;
};

dictionary MIDIMessageEventInit: EventInit {
  Uint8Array data;
};