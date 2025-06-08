/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webappsec-csp/#cspviolationreportbody

[Exposed=Window]
interface CSPViolationReportBody : ReportBody {
  [Default] object toJSON();
  readonly attribute USVString documentURL;
  readonly attribute USVString? referrer;
  readonly attribute USVString? blockedURL;
  readonly attribute DOMString effectiveDirective;
  readonly attribute DOMString originalPolicy;
  readonly attribute USVString? sourceFile;
  readonly attribute DOMString? sample;
  readonly attribute SecurityPolicyViolationEventDisposition disposition;
  readonly attribute unsigned short statusCode;
  readonly attribute unsigned long? lineNumber;
  readonly attribute unsigned long? columnNumber;
};
