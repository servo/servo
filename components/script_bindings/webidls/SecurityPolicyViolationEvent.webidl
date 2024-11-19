/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webappsec-csp/#securitypolicyviolationevent

enum SecurityPolicyViolationEventDisposition {
  "enforce", "report"
};

[Exposed=(Window,Worker)]
interface SecurityPolicyViolationEvent : Event {
    constructor(DOMString type, optional SecurityPolicyViolationEventInit eventInitDict = {});
    readonly    attribute USVString      documentURI;
    readonly    attribute USVString      referrer;
    readonly    attribute USVString      blockedURI;
    readonly    attribute DOMString      effectiveDirective;
    readonly    attribute DOMString      violatedDirective; // historical alias of effectiveDirective
    readonly    attribute DOMString      originalPolicy;
    readonly    attribute USVString      sourceFile;
    readonly    attribute DOMString      sample;
    readonly    attribute SecurityPolicyViolationEventDisposition      disposition;
    readonly    attribute unsigned short statusCode;
    readonly    attribute unsigned long  lineNumber;
    readonly    attribute unsigned long  columnNumber;
};

dictionary SecurityPolicyViolationEventInit : EventInit {
    USVString      documentURI = "";
    USVString      referrer = "";
    USVString      blockedURI = "";
    DOMString      violatedDirective = "";
    DOMString      effectiveDirective = "";
    DOMString      originalPolicy = "";
    USVString      sourceFile = "";
    DOMString      sample = "";
    SecurityPolicyViolationEventDisposition disposition = "enforce";
    unsigned short statusCode = 0;
    unsigned long  lineNumber = 0;
    unsigned long  columnNumber = 0;
};
