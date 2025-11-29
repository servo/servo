/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-speech-api/#speechsynthesis
 */

[Exposed=Window]
interface SpeechSynthesis : EventTarget {
    // readonly attribute boolean pending;
    // readonly attribute boolean speaking;
    // readonly attribute boolean paused;

    // attribute EventHandler onvoiceschanged;

    undefined speak(SpeechSynthesisUtterance utterance);
    undefined cancel();
    undefined pause();
    undefined resume();
    sequence<SpeechSynthesisVoice> getVoices();
};

partial interface Window {
    [SameObject] readonly attribute SpeechSynthesis speechSynthesis;
};
