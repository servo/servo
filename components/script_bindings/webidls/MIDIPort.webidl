/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_WEBMIDI

/*
 * The origin of this IDL file is:
 * https://webaudio.github.io/web-midi-api/#dom-midiport
 */

[Pref="dom_webmidi_enabled", SecureContext, Exposed=(Window,Worker)]
interface MIDIPort: EventTarget {
  readonly attribute DOMString id;
  readonly attribute DOMString? manufacturer;
  readonly attribute DOMString? name;
  readonly attribute MIDIPortType type;
  readonly attribute DOMString? version;
  readonly attribute MIDIPortDeviceState state;
  readonly attribute MIDIPortConnectionState connection;
  attribute EventHandler onstatechange;
  Promise<MIDIPort> open();
  Promise<MIDIPort> close();
};

[Pref="dom_webmidi_enabled", SecureContext, Exposed=(Window,Worker)]
interface MIDIInput: MIDIPort {
  attribute EventHandler onmidimessage;
};

[Pref="dom_webmidi_enabled", SecureContext, Exposed=(Window,Worker)]
interface MIDIOutput : MIDIPort {
  [Throws] undefined send(sequence<octet> data, optional DOMHighResTimeStamp timestamp = 0);
  undefined clear();
};

enum MIDIPortType {
  "input",
  "output",
};

enum MIDIPortDeviceState {
  "disconnected",
  "connected",
};

enum MIDIPortConnectionState {
  "open",
  "closed",
  "pending",
};