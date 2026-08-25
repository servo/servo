// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.find
description: >
  Predicate is only called if this.length is > 0.
info: |
  22.1.3.8 Array.prototype.find ( predicate[ , thisArg ] )

  ...
  7. Let k be 0.
  8. Repeat, while k < len
    ...
    d. Let testResult be ToBoolean(Call(predicate, T, «kValue, k, O»)).
  ...
  9. Return undefined.
---*/

var called = false;

var predicate = function() {
  called = true;
  return true;
};

var result = [].find(predicate);

assert.sameValue(called, false, '[].find(predicate) does not call predicate');
assert.sameValue(result, undefined, '[].find(predicate) returned undefined');
