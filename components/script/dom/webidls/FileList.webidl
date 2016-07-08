/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/FileAPI/#dfn-filelist

[Exposed=(Window,Worker)]
interface FileList {
  getter File? item(unsigned long index);
  readonly attribute unsigned long length;
};
