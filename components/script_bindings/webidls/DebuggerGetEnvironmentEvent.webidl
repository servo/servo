/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// This interface is entirely internal to Servo, and should not be accessible to
// web pages.
[Exposed=DebuggerGlobalScope]
interface DebuggerGetEnvironmentEvent : Event {
    readonly attribute DOMString frameActorId;
};

partial interface DebuggerGlobalScope {
    DOMString? registerEnvironmentActor(
        EnvironmentInfo result,
        DOMString? parent
    );
    undefined getEnvironmentResult(
        DOMString environmentActorId
    );
};

dictionary EnvironmentVariable {
    required PropertyDescriptor property;
    ObjectPreview preview;
};

dictionary EnvironmentInfo {
    DOMString type_;
    DOMString scopeKind;
    DOMString functionDisplayName;
    sequence<EnvironmentVariable> bindingVariables;
};
