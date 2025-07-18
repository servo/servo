/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webappsec-csp/#dictdef-cspviolationreportbody

dictionary CSPViolationReportBody : ReportBody {
  required USVString documentURL;
  USVString referrer;
  USVString blockedURL;
  required DOMString effectiveDirective;
  required DOMString originalPolicy;
  USVString sourceFile;
  DOMString sample;
  required SecurityPolicyViolationEventDisposition disposition;
  required unsigned short statusCode;
  unsigned long lineNumber;
  unsigned long columnNumber;
};
