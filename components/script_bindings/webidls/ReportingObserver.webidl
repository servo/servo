/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/reporting/#interface-reporting-observer

dictionary ReportBody {
};

dictionary Report {
  required DOMString type;
  required DOMString url;
  required DOMString destination;
  required double timestamp;
  required long attempts;
  // TODO(37328): Change this to parent class ReportBody
  CSPViolationReportBody body;
};

[Exposed=(Window,Worker)]
interface ReportingObserver {
  constructor(ReportingObserverCallback callback, optional ReportingObserverOptions options = {});
  undefined observe();
  undefined disconnect();
  ReportList takeRecords();
};

callback ReportingObserverCallback = undefined (sequence<Report> reports, ReportingObserver observer);

dictionary ReportingObserverOptions {
  sequence<DOMString> types;
  boolean buffered = false;
};

typedef sequence<Report> ReportList;
