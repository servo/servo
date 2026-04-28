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
    readonly attribute DOMString? frameActorId;
};

partial interface DebuggerGlobalScope {
    undefined evalResult(DebuggerEvalEvent event, EvalResult result);
};

dictionary EvalResult {
    required DebuggerValue value;
    ObjectPreview preview;
    required DOMString completionType;
    boolean hasException;
};

dictionary PropertyDescriptor {
    required DOMString name;
    required DebuggerValue value;
    required boolean configurable;
    required boolean enumerable;
    required boolean writable;
    required boolean isAccessor;
};

dictionary DebuggerValue {
    required DOMString valueType;
    boolean booleanValue;
    double numberValue;
    DOMString stringValue;
    DOMString objectClass;
};

dictionary ObjectPreview {
    required DOMString kind;
    sequence<PropertyDescriptor> ownProperties;
    unsigned long ownPropertiesLength;
    unsigned long arrayLength;
    sequence<DebuggerValue> items;
    FunctionPreview function;
};

// Function-specific metadata
// <https://searchfox.org/mozilla-central/source/devtools/server/actors/object/previewers.js>
dictionary FunctionPreview {
    DOMString name;
    DOMString displayName;
    required sequence<DOMString> parameterNames;
    boolean isAsync;
    boolean isGenerator;
};
