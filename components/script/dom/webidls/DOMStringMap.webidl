/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-domstringmap-interface
[OverrideBuiltins]
interface DOMStringMap {
  getter DOMString (DOMString name);
  [Throws]
  setter void (DOMString name, DOMString value);
  deleter void (DOMString name);
};
