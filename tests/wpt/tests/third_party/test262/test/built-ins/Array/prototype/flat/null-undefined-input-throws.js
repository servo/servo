// Copyright (C) 2018 Shilpi Jain and Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flat
description: >
    null or undefined should throw TypeError Exception
features: [Array.prototype.flat]
---*/

assert.sameValue(typeof Array.prototype.flat, 'function');

assert.throws(TypeError, function() {
  [].flat.call(null);
}, 'null value');

assert.throws(TypeError, function() {
  [].flat.call();
}, 'missing');

assert.throws(TypeError, function() {
  [].flat.call(void 0);
}, 'undefined');
