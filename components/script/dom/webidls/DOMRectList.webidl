/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://dev.w3.org/fxtf/geometry/#DOMRectList
[NoInterfaceObject, Exposed=(Window,Worker)]
//[ArrayClass]
interface DOMRectList {
  readonly attribute unsigned long length;
  getter DOMRect? item(unsigned long index);
};
