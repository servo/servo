// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-dataview-buffer-byteoffset-bytelength
description: >
  The view's offset may equal the underlying buffer's byte length if it is
  modified during retrieval of the NewTarget's prototype.
features: [resizable-arraybuffer]
---*/

// If the host chooses to throw as allowed by the specification, the observed
// behavior will be identical to the case where `ArrayBuffer.prototype.resize`
// has not been implemented. The following assertion prevents this test from
// passing in runtimes which have not implemented the method.
assert.sameValue(typeof ArrayBuffer.prototype.resize, 'function');

var buffer = new ArrayBuffer(3, {maxByteLength: 3});
var expectedByteLength;

var newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, 'prototype', {
  get: function() {
    try {
      buffer.resize(2);
      expectedByteLength = 0;
    } catch (error) {
      expectedByteLength = 1;
    }
  }
});

var result = Reflect.construct(DataView, [buffer, 2], newTarget);

assert.sameValue(result.constructor, DataView);
assert.sameValue(result.byteLength, expectedByteLength);
