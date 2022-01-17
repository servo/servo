/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/mediacapture-main/#dom-mediastream

[Exposed=Window]
interface MediaStream : EventTarget {
    [Throws] constructor();
    [Throws] constructor(MediaStream stream);
    [Throws] constructor(sequence<MediaStreamTrack> tracks);
    // readonly        attribute DOMString id;
    sequence<MediaStreamTrack> getAudioTracks();
    sequence<MediaStreamTrack> getVideoTracks();
    sequence<MediaStreamTrack> getTracks();
    MediaStreamTrack? getTrackById(DOMString trackId);
    undefined addTrack(MediaStreamTrack track);
    undefined removeTrack(MediaStreamTrack track);
    MediaStream clone();
    // readonly        attribute boolean active;
    //                 attribute EventHandler onaddtrack;
    //                 attribute EventHandler onremovetrack;
};
