/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dom.spec.whatwg.org/#interface-domimplementation
 *
 * Copyright:
 * To the extent possible under law, the editors have waived all copyright and
 * related or neighboring rights to this work.
 */

[Exposed=Window]
interface DOMImplementation {
  [NewObject, Throws]
  DocumentType createDocumentType(DOMString qualifiedName, DOMString publicId,
                                  DOMString systemId);
  [NewObject, Throws]
  XMLDocument createDocument(DOMString? namespace,
                             [LegacyNullToEmptyString] DOMString qualifiedName,
                             optional DocumentType? doctype = null);
  [NewObject]
  Document createHTMLDocument(optional DOMString title);

  [Pure]
  boolean hasFeature(); // useless, always return true
};
