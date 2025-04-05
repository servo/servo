/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://www.w3.org/TR/trusted-types/#trusted-type-policy
 */

[Exposed=(Window,Worker), Pref="dom_trusted_types_enabled"]
interface TrustedTypePolicy {
  readonly attribute DOMString name;
  TrustedHTML createHTML(DOMString input, any... arguments);
  TrustedScript createScript(DOMString input, any... arguments);
  TrustedScriptURL createScriptURL(DOMString input, any... arguments);
};
