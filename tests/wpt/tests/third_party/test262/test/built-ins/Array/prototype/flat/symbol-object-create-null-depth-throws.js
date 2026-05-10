// Copyright (C) 2018 Shilpi Jain and Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flat
description: >
    if the argument is a Symbol or Object null, it throws exception
features: [Array.prototype.flat]
---*/

assert.sameValue(typeof Array.prototype.flat, 'function');

assert.throws(TypeError, function() {
  [].flat(Symbol());
}, 'symbol value');

assert.throws(TypeError, function() {
  [].flat(Object.create(null));
}, 'object create null');
