// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.9
description: >
    Behavior when error thrown while initially setting `lastIndex` property
info: |
    [...]
    7. Let status be Set(rx, "lastIndex", 0, true).
    8. ReturnIfAbrupt(status).
features: [Symbol.search]
---*/

var callCount;
var poisonedLastIndex = {
  get lastIndex() {
    callCount += 1;
  },
  set lastIndex(_) {
    throw new Test262Error();
  }
};
var nonWritableLastIndex = {
  get lastIndex() {
    callCount += 1;
  },
  // This method defined to avoid false positives from unrelated TypeErrors
  exec: function() {
    return null;
  }
};

callCount = 0;
assert.throws(Test262Error, function() {
  RegExp.prototype[Symbol.search].call(poisonedLastIndex);
});
assert.sameValue(
  callCount,
  1,
  'Property value was accessed before being set ("poisoned" lastIndex)'
);

callCount = 0;
assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.search].call(nonWritableLastIndex);
});
assert.sameValue(
  callCount,
  1,
  'Property value was accessed before being set (non-writable lastIndex)'
);
