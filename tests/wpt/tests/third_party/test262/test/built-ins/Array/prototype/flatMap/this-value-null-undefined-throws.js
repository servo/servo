// Copyright (C) 2018 Shilpi Jain and Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flatmap
description: >
  Throw a TypeError if this value is null or undefined
info: |
  Array.prototype.flatMap ( mapperFunction [ , thisArg ] )

  1. Let O be ? ToObject(this value).
  ...
features: [Array.prototype.flatMap]
---*/

assert.sameValue(typeof Array.prototype.flatMap, 'function');

var mapperFn = function() {};

assert.throws(TypeError, function() {
  [].flatMap.call(null, mapperFn);
}, 'null value');

assert.throws(TypeError, function() {
  [].flatMap.call(undefined, mapperFn);
}, 'undefined');
