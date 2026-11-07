/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_WEBMIDI

// https://webaudio.github.io/web-midi-api/#extensions-to-the-navigator-interface
partial interface Navigator {
  [Pref="dom_webmidi_enabled", SecureContext]
  Promise<MIDIAccess> requestMIDIAccess(optional MIDIOptions options = {});
};

dictionary MIDIOptions {
  boolean sysex;
  boolean software;
};