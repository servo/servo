/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

[Exposed=(Window,Worker,Worklet,DissimilarOriginWindow,DebuggerGlobalScope)]
interface EventTarget {
  [Throws] constructor();
};

[Exposed=(Window,Worker,Worklet,DissimilarOriginWindow,DebuggerGlobalScope),
 Inline]
interface GlobalScope : EventTarget {};

[Abstract, Exposed=Worker]
interface WorkerGlobalScope : GlobalScope {};

[Global=Window, Exposed=Window]
interface Window : GlobalScope {
};

[Global=DissimilarOriginWindow, Exposed=(Window,DissimilarOriginWindow), LegacyNoInterfaceObject]
interface DissimilarOriginWindow : GlobalScope {};

[Global=DebuggerGlobalScope, Exposed=DebuggerGlobalScope]
interface DebuggerGlobalScope: GlobalScope {};

[Global=(Worker,DedicatedWorker), Exposed=DedicatedWorker]
interface DedicatedWorkerGlobalScope : WorkerGlobalScope {};

[Global=(Worklet,PaintWorklet), Pref="dom_worklet_enabled", Exposed=PaintWorklet]
interface PaintWorkletGlobalScope : WorkletGlobalScope {};

[Pref="dom_worklet_enabled", Exposed=Worklet]
interface WorkletGlobalScope: GlobalScope {
};
