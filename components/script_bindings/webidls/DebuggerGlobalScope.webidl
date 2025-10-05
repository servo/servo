/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// This interface is entirely internal to Servo, and should not be accessible to
// web pages.
partial interface DebuggerGlobalScope {
    undefined notifyNewSource(NotifyNewSource args);
};

dictionary NotifyNewSource {
    required PipelineIdInit pipelineId;
    required DOMString? workerId;
    required unsigned long spidermonkeyId;
    required DOMString url;
    required DOMString? urlOverride;
    required DOMString text;
    required DOMString? introductionType;
};

dictionary PipelineIdInit {
    required unsigned long namespaceId;
    required unsigned long index;
};
