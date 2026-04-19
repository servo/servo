// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonarray
description: >
  Abrupt completion from Get.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  10. If Type(value) is Object and IsCallable(value) is false, then
    a. Let isArray be ? IsArray(value).
    b. If isArray is true, return ? SerializeJSONArray(value).

  SerializeJSONArray ( value )

  [...]
  6. Let len be ? LengthOfArrayLike(value).
features: [Proxy]
---*/

var abruptLength = new Proxy([], {
  get: function(_target, key) {
    if (key === 'length') {
      throw new Test262Error();
    }
  },
});

assert.throws(Test262Error, function() {
  JSON.stringify(abruptLength);
});

var abruptToPrimitive = {
  valueOf: function() {
    throw new Test262Error();
  },
};

var abruptToLength = new Proxy([], {
  get: function(_target, key) {
    if (key === 'length') {
      return abruptToPrimitive;
    }
  },
});

assert.throws(Test262Error, function() {
  JSON.stringify([abruptToLength]);
});

var abruptIndex = new Array(1);
Object.defineProperty(abruptIndex, '0', {
  get: function() {
    throw new Test262Error();
  },
});

assert.throws(Test262Error, function() {
  JSON.stringify({key: abruptIndex});
});
