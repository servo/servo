/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://www.w3.org/TR/trusted-types/#trusted-type-policy-factory
 */

[Exposed=(Window,Worker), Pref="dom_trusted_types_enabled"]
interface TrustedTypePolicyFactory {
    [Throws]
    TrustedTypePolicy createPolicy(
        DOMString policyName, optional TrustedTypePolicyOptions policyOptions = {});
    boolean isHTML(any value);
    boolean isScript(any value);
    boolean isScriptURL(any value);
    readonly attribute TrustedHTML emptyHTML;
    readonly attribute TrustedScript emptyScript;
    DOMString? getAttributeType(
        DOMString tagName,
        DOMString attribute,
        optional DOMString? elementNs = "",
        optional DOMString? attrNs = "");
    DOMString? getPropertyType(
        DOMString tagName,
        DOMString property,
        optional DOMString? elementNs = "");
    readonly attribute TrustedTypePolicy? defaultPolicy;
};

dictionary TrustedTypePolicyOptions {
   CreateHTMLCallback createHTML;
   CreateScriptCallback createScript;
   CreateScriptURLCallback createScriptURL;
};
callback CreateHTMLCallback = DOMString? (DOMString input, any... arguments);
callback CreateScriptCallback = DOMString? (DOMString input, any... arguments);
callback CreateScriptURLCallback = USVString? (DOMString input, any... arguments);
