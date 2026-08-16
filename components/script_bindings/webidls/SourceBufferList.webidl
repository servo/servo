/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/media-source/#idl-def-sourcebufferlist

[Exposed=Window, Pref="dom_media_source_enabled"]
interface SourceBufferList : EventTarget {
  readonly attribute unsigned long length;

  attribute EventHandler onaddsourcebuffer;
  attribute EventHandler onremovesourcebuffer;

  getter SourceBuffer (unsigned long index);
};
