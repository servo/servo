/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/*
 * The origin of this IDL file is
 * https://www.w3.org/TR/media-source-2/#sourcebufferlist
 *
 */

// https://www.w3.org/TR/media-source-2/#dom-sourcebufferlist
[Pref="dom_media_source_extensions_enabled", Exposed=(Window)]
interface SourceBufferList : EventTarget {
  readonly attribute unsigned long length;

  attribute EventHandler onaddsourcebuffer;
  attribute EventHandler onremovesourcebuffer;

  getter SourceBuffer (unsigned long index);
};
