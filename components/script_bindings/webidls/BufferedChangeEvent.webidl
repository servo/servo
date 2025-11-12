/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/*
 * The origin of this IDL file is
 * https://www.w3.org/TR/media-source-2/#bufferedchangeevent-interface
 *
 */

// https://www.w3.org/TR/media-source-2/#dom-bufferedchangeevent
[Pref="dom_media_source_extensions_enabled", Exposed=(Window)]
interface BufferedChangeEvent : Event {
  constructor(DOMString type, optional BufferedChangeEventInit eventInitDict = {});

  [SameObject] readonly attribute TimeRanges addedRanges;
  [SameObject] readonly attribute TimeRanges removedRanges;
};

// https://www.w3.org/TR/media-source-2/#dom-bufferedchangeeventinit
dictionary BufferedChangeEventInit : EventInit {
  TimeRanges addedRanges;
  TimeRanges removedRanges;
};
