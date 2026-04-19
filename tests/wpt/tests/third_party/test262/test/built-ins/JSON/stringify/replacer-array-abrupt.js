// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Abrupt completion from Get.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  4. If Type(replacer) is Object, then
    [...]
    2. Let len be ? LengthOfArrayLike(replacer).
    3. Let k be 0.
    4. Repeat, while k < len,
      a. Let v be ? Get(replacer, ! ToString(k)).
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
  JSON.stringify(null, abruptLength);
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
  JSON.stringify([], abruptToLength);
});

var abruptIndex = new Array(1);
Object.defineProperty(abruptIndex, '0', {
  get: function() {
    throw new Test262Error();
  },
});

assert.throws(Test262Error, function() {
  JSON.stringify({}, abruptIndex);
});
