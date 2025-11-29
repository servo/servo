/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-speech-api/#speechsynthesisutterance
 */

[Exposed=Window]
interface SpeechSynthesisUtterance : EventTarget {
    constructor(optional DOMString text);

    attribute DOMString text;
    attribute DOMString lang;
    attribute SpeechSynthesisVoice? voice;
    attribute float volume;
    attribute float rate;
    attribute float pitch;

    attribute EventHandler onstart;
    attribute EventHandler onend;
    attribute EventHandler onerror;
    attribute EventHandler onpause;
    attribute EventHandler onresume;
    attribute EventHandler onmark;
    attribute EventHandler onboundary;
};
