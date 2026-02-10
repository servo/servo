/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// This interface is entirely internal to Servo, and should not be accessible to
// web pages.
[Exposed=DebuggerGlobalScope]
interface DebuggerEvalEvent : Event {
    readonly attribute DOMString code;
    readonly attribute PipelineId pipelineId;
    readonly attribute DOMString? workerId;
};

partial interface DebuggerGlobalScope {
    undefined evalResult(DebuggerEvalEvent event, EvalResultValue result);
};

// Result from Debugger.Object.executeInGlobal() completion value.
// <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Object.html#executeinglobal-code-options>
//
// Completion values are described at:
// <https://firefox-source-docs.mozilla.org/js/Debugger/Conventions.html#completion-values>
// - { return: value } for normal completion
// - { throw: value, stack } for thrown exception
// - null for termination
//
// The `value` inside is a debuggee value:
// <https://firefox-source-docs.mozilla.org/js/Debugger/Conventions.html#debuggee-values>
dictionary EvalResultValue {
    required DOMString completionType;
    required DOMString valueType;
    boolean? booleanValue;
    double? numberValue;
    DOMString? stringValue;

    // A string naming the ECMAScript [[Class]] of the referent.
    // <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Object.html#accessor-properties-of-the-debugger-object-prototype>
    DOMString? objectClass;
};
