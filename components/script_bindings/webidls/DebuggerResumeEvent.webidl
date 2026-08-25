/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// This interface is entirely internal to Servo, and should not be accessible to
// web pages.
[Exposed=DebuggerGlobalScope]
interface DebuggerResumeEvent : Event {
    readonly attribute DOMString? resumeLimitType;
    readonly attribute DOMString? frameActorID;
};
