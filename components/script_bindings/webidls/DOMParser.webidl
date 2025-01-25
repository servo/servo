/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/DOM-Parsing/#the-domparser-interface
 */

enum SupportedType {
  "text/html",
  "text/xml",
  "application/xml",
  "application/xhtml+xml",
  "image/svg+xml"
};

[Exposed=Window]
interface DOMParser {
  [Throws] constructor();
  [Throws]
  Document parseFromString(DOMString str, SupportedType type);
};
