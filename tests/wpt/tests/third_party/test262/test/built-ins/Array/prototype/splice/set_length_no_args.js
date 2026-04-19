// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Array.prototype.splice sets length when called with no arguments
info: |
  22.1.3.25 Array.prototype.splice (start, deleteCount , ...items )

  ...
  24. Let setStatus be Set(O, "length", len – actualDeleteCount + itemCount, true).
  25. ReturnIfAbrupt(setStatus).
esid: sec-array.prototype.splice
---*/

var getCallCount = 0,
  setCallCount = 0;
var lengthValue;

var obj = {
  get length() {
    getCallCount += 1;
    return "0";
  },
  set length(v) {
    setCallCount += 1;
    lengthValue = v;
  }
};

Array.prototype.splice.call(obj);

assert.sameValue(getCallCount, 1, "Get('length') called exactly once");
assert.sameValue(setCallCount, 1, "Set('length') called exactly once");
assert.sameValue(lengthValue, 0, "Set('length') called with ToLength('0')");
