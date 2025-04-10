/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://www.w3.org/TR/trusted-types/#trusted-html
 */

[Exposed=(Window,Worker), Pref="dom_trusted_types_enabled"]
interface TrustedHTML {
  stringifier;
  DOMString toJSON();
};
