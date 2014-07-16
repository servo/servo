/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://www.whatwg.org/html/#workerglobalscope
//[Exposed=Worker]
interface WorkerGlobalScope : EventTarget {
  readonly attribute WorkerGlobalScope self;
  //readonly attribute WorkerLocation location;

  //void close();
  //         attribute OnErrorEventHandler onerror;
  //         attribute EventHandler onlanguagechange;
  //         attribute EventHandler onoffline;
  //         attribute EventHandler ononline;

  // also has obsolete members
};

// http://www.whatwg.org/html/#WorkerGlobalScope-partial
//[Exposed=Worker]
partial interface WorkerGlobalScope {
  //void importScripts(DOMString... urls);
  //readonly attribute WorkerNavigator navigator;
};
//WorkerGlobalScope implements WindowTimers;
//WorkerGlobalScope implements WindowBase64;

// Proprietary
partial interface WorkerGlobalScope {
  [Replaceable]
  readonly attribute Console console;
};
