/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// This interface is entirely internal to Servo, and should not be accessible to
// web pages.
[Exposed=DebuggerGlobalScope]
interface DebuggerInterruptEvent : Event {};

partial interface DebuggerGlobalScope {
    undefined pauseAndRespond(
        PipelineIdInit pipelineId,
        FrameOffset frameOffset,
        PauseReason pauseReason);

    DOMString? registerFrameActor(
        PipelineIdInit pipelineId,
        FrameInfo result);
};

dictionary PauseReason {
    required DOMString type_;
    boolean onNext;
};

dictionary FrameInfo {
    required DOMString displayName;
    required boolean onStack;
    required boolean oldest;
    required boolean terminated;
    required DOMString type_;
    required DOMString url;
};

dictionary FrameOffset {
    required DOMString frameActorId;
    required unsigned long column;
    required unsigned long line;
};
