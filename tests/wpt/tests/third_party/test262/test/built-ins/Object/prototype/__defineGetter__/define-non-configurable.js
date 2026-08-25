// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-additional-properties-of-the-object.prototype-object
description: Behavior when property exists and is not configurable
info: |
    [...]
    5. Perform ? DefinePropertyOrThrow(O, key, desc).
features: [__getter__]
---*/

var noop = function() {};
var subject = Object.defineProperty(
  {}, 'attr', { value: 1, configurable: false }
);

assert.sameValue(typeof Object.prototype.__defineGetter__, 'function');

assert.throws(TypeError, function() {
  subject.__defineGetter__('attr', noop);
});
