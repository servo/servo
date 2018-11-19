/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#windoworworkerglobalscope

// FIXME(nox): https://github.com/servo/servo/issues/20700
// typedef (DOMString or Function) TimerHandler;

[NoInterfaceObject, Exposed=(Window,Worker)]
interface WindowOrWorkerGlobalScope {
  [Replaceable] readonly attribute USVString origin;

  // base64 utility methods
  [Throws] DOMString btoa(DOMString data);
  [Throws] DOMString atob(DOMString data);

  // timers
  // FIXME(nox): https://github.com/servo/servo/issues/20700
  long setTimeout(Function handler, optional long timeout = 0, any... arguments);
  long setTimeout(DOMString handler, optional long timeout = 0, any... arguments);
  void clearTimeout(optional long handle = 0);
  // FIXME(nox): https://github.com/servo/servo/issues/20700
  long setInterval(Function handler, optional long timeout = 0, any... arguments);
  long setInterval(DOMString handler, optional long timeout = 0, any... arguments);
  void clearInterval(optional long handle = 0);

  // ImageBitmap
  // Promise<ImageBitmap> createImageBitmap(ImageBitmapSource image, optional ImageBitmapOptions options);
  // Promise<ImageBitmap> createImageBitmap(
  //   ImageBitmapSource image, long sx, long sy, long sw, long sh, optional ImageBitmapOptions options);
};

// https://w3c.github.io/hr-time/#the-performance-attribute
partial interface WindowOrWorkerGlobalScope {
    [Replaceable]
    readonly attribute Performance performance;
};

Window implements WindowOrWorkerGlobalScope;
WorkerGlobalScope implements WindowOrWorkerGlobalScope;
