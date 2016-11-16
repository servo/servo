/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * http://dev.w3.org/csswg/cssom/#the-css-interface
 */

[Abstract, Exposed=Window]
interface CSS {
  [Throws]
  static DOMString escape(DOMString ident);
};
