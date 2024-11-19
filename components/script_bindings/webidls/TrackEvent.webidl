/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-trackevent-interface

[Exposed=Window]
interface TrackEvent : Event {
  [Throws] constructor(DOMString type, optional TrackEventInit eventInitDict = {});
  readonly attribute (VideoTrack or AudioTrack or TextTrack)? track;
};

dictionary TrackEventInit : EventInit {
  (VideoTrack or AudioTrack or TextTrack)? track = null;
};
