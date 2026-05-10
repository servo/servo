// Copyright (C) 2018 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Jeff Walden <jwalden+code@mit.edu>
esid: sec-dataview-buffer-byteoffset-bytelength
description: >
  The `DataView` constructor shouldn't be able to return a `DataView` instance
  backed by a detached `ArrayBuffer` when `OrdinaryCreateFromConstructor`
  returns an instance so backed.
info: |
  `OrdinaryCreateFromConstructor` has the potential to invoke user-defined code
  that may detach the `ArrayBuffer` intended to underlie the fresh instance.
  Verify that a final is-detached check is performed before the new instance is
  returned.
includes: [detachArrayBuffer.js]
features: [Reflect.construct]
---*/

var buffer = new ArrayBuffer(8);

var called = false;
var byteOffset = { valueOf() { called = true; return 0; } };

var newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, "prototype", {
  get() {
    $DETACHBUFFER(buffer);
    return DataView.prototype;
  }
});

assert.throws(TypeError, function() {
  Reflect.construct(DataView, [buffer, byteOffset], newTarget);
});
assert.sameValue(called, true);
