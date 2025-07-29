/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// This interface is entirely internal to Servo, and should not be accessible to
// web pages.
[Global=DebuggerGlobalScope, Exposed=DebuggerGlobalScope]
interface DebuggerGlobalScope: GlobalScope {
    undefined notifyNewSource(NotifyNewSource args);
};

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-window-interface
dictionary NotifyNewSource {
    required PipelineId pipelineId;
    required unsigned long spidermonkeyId;
    required DOMString url;
    required DOMString text;

    // FIXME: error[E0599]: the method `trace` exists for reference `&Option<TypedArray<Uint8, *mut JSObject>>`, but
    // its trait bounds were not satisfied
    // Uint8Array binary;

    // TODO: contentType
};

dictionary PipelineId {
    required unsigned long namespaceId;
    required unsigned long index;
};
