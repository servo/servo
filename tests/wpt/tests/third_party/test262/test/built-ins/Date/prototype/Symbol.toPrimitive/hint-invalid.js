// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype-@@toprimitive
description: Behavior when an invalid `hint` argument is specified
info: |
    1. Let O be the this value.
    2. If Type(O) is not Object, throw a TypeError exception.
    3. If hint is the String value "string" or the String value "default", then
       a. Let tryFirst be "string".
    4. Else if hint is the String value "number", then
       a. Let tryFirst be "number".
    5. Else, throw a TypeError exception.
features: [Symbol.toPrimitive]
---*/

var d = new Date(0);

assert.sameValue(typeof d[Symbol.toPrimitive], 'function');

assert.throws(TypeError, function() {
  d[Symbol.toPrimitive]();
});

assert.throws(TypeError, function() {
  d[Symbol.toPrimitive](undefined);
});

assert.throws(TypeError, function() {
  d[Symbol.toPrimitive](null);
});

assert.throws(TypeError, function() {
  d[Symbol.toPrimitive]('');
});

assert.throws(TypeError, function() {
  d[Symbol.toPrimitive]('String');
});

assert.throws(TypeError, function() {
  d[Symbol.toPrimitive]('defaultnumber');
});

assert.throws(TypeError, function() {
  d[Symbol.toPrimitive](new String('number'));
});

assert.throws(TypeError, function() {
  d[Symbol.toPrimitive]({
    toString: function() {
      'number';
    }
  });
});
