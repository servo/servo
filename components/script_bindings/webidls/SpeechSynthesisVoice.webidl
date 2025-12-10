/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-speech-api/#speechsynthesisvoice
 */

[Exposed=Window]
interface SpeechSynthesisVoice {
    readonly attribute DOMString voiceURI;
    readonly attribute DOMString name;
    readonly attribute DOMString lang;
    readonly attribute boolean localService;
    readonly attribute boolean default;
};
