/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#workerglobalscope
[Abstract, Exposed=Worker]
interface WorkerGlobalScope : GlobalScope {
  [BinaryName="Self_"] readonly attribute WorkerGlobalScope self;
  readonly attribute WorkerLocation location;
  readonly attribute WorkerNavigator navigator;
  [Throws] undefined importScripts((TrustedScriptURL or USVString)... urls);
};

// This is an interface internal to Servo to simplify adding
// event handlers to objects that implement WorkerGlobalScope
interface mixin WorkerGlobalScopeEvents {
  attribute OnErrorEventHandler onerror;
  attribute EventHandler onlanguagechange;
  attribute EventHandler onoffline;
  attribute EventHandler ononline;
  attribute EventHandler onrejectionhandled;
  attribute EventHandler onunhandledrejection;
};

WorkerGlobalScope includes WorkerGlobalScopeEvents;
DedicatedWorkerGlobalScope includes WorkerGlobalScopeEvents;
// ServiceWorkerGlobalScope includes WorkerGlobalScopeEvents;
// SharedWorkerGlobalScope includes WorkerGlobalScopeEvents;
