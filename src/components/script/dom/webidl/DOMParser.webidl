/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://domparsing.spec.whatwg.org/#the-domparser-interface
 */

/*interface Principal;
interface URI;
interface InputStream;*/

enum SupportedType {
  "text/html",
  "text/xml",
  "application/xml",
  "application/xhtml+xml",
  "image/svg+xml"
};

// the latter is Mozilla-specific
/*[Constructor,
 Constructor(Principal? prin, optional URI? documentURI = null,
 optional URI? baseURI = null)]*/
[Constructor]
interface DOMParser {
  [Creator, Throws]
  Document parseFromString(DOMString str, SupportedType type);

  /*  // Mozilla-specific stuff
  // Throws if the passed-in length is greater than the actual sequence length
  [Creator, Throws, ChromeOnly]
  Document parseFromBuffer(sequence<octet> buf, unsigned long bufLen,
                           SupportedType type);
  // Throws if the passed-in length is greater than the actual typed array length
  [Creator, Throws, ChromeOnly]
  Document parseFromBuffer(Uint8Array buf, unsigned long bufLen,
                           SupportedType type);
  [Creator, Throws, ChromeOnly]
  Document parseFromStream(InputStream stream, DOMString? charset,
                           long contentLength, SupportedType type);
  [Throws, ChromeOnly]
  void init(optional Principal? principal = null,
            optional URI? documentURI = null,
            optional URI? baseURI = null);*/
};

