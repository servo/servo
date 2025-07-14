/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#windoworworkerglobalscope

typedef (DOMString or Function) TimerHandler;

[Exposed=(Window,Worker)]
interface mixin WindowOrWorkerGlobalScope {
  [Replaceable] readonly attribute USVString origin;

  // base64 utility methods
  [Throws] DOMString btoa(DOMString data);
  [Throws] DOMString atob(DOMString data);

  // timers
  long setTimeout(TimerHandler handler, optional long timeout = 0, any... arguments);
  undefined clearTimeout(optional long handle = 0);
  long setInterval(TimerHandler handler, optional long timeout = 0, any... arguments);
  undefined clearInterval(optional long handle = 0);

  // microtask queuing
  undefined queueMicrotask(VoidFunction callback);

  // ImageBitmap
  Promise<ImageBitmap> createImageBitmap(ImageBitmapSource image, optional ImageBitmapOptions options = {});
  Promise<ImageBitmap> createImageBitmap(ImageBitmapSource image, long sx, long sy, long sw, long sh,
                                         optional ImageBitmapOptions options = {});

  // structured cloning
  [Throws]
  any structuredClone(any value, optional StructuredSerializeOptions options = {});
};

// https://w3c.github.io/hr-time/#the-performance-attribute
partial interface mixin WindowOrWorkerGlobalScope {
    [Replaceable]
    readonly attribute Performance performance;
};

// https://w3c.github.io/webappsec-secure-contexts/#monkey-patching-global-object
partial interface mixin WindowOrWorkerGlobalScope {
  readonly attribute boolean isSecureContext;
};

// https://www.w3.org/TR/trusted-types/#extensions-to-the-windoworworkerglobalscope-interface
partial interface mixin WindowOrWorkerGlobalScope {
  [Pref="dom_trusted_types_enabled"]
  readonly attribute TrustedTypePolicyFactory trustedTypes;
};

Window includes WindowOrWorkerGlobalScope;
WorkerGlobalScope includes WindowOrWorkerGlobalScope;
