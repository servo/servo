/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// This interface is entirely internal to Servo, and should not be accessible to
// web pages.
[Exposed=DebuggerGlobalScope]
interface GetPossibleBreakpointsEvent : Event {
    readonly attribute unsigned long spidermonkeyId;
};

partial interface DebuggerGlobalScope {
    undefined getPossibleBreakpointsResult(
        GetPossibleBreakpointsEvent event,
        sequence<RecommendedBreakpointLocation> result);
};

// <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Script.html#getpossiblebreakpoints-query>
dictionary RecommendedBreakpointLocation {
    required unsigned long offset;
    required unsigned long lineNumber;
    required unsigned long columnNumber;
    required boolean isStepStart;
};
