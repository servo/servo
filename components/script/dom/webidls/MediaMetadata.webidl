/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/mediasession/#mediametadata
 */

dictionary MediaImage {
  required USVString src;
  DOMString sizes = "";
  DOMString type = "";
};

[Exposed=Window]
interface MediaMetadata {
  [Throws] constructor(optional MediaMetadataInit init = {});
  attribute DOMString title;
  attribute DOMString artist;
  attribute DOMString album;
  // TODO: https://github.com/servo/servo/issues/10072
  // attribute FrozenArray<MediaImage> artwork;
};

dictionary MediaMetadataInit {
  DOMString title = "";
  DOMString artist = "";
  DOMString album = "";
  sequence<MediaImage> artwork = [];
};
