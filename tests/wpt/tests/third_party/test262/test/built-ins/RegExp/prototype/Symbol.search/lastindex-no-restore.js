// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.9
description: Behavior when some lastIndex writes should be skipped.
info: |
    [...]
    4. Let previousLastIndex be ? Get(rx, "lastIndex").
    5. If SameValue(previousLastIndex, 0) is false, then
       a. Perform ? Set(rx, "lastIndex", 0, true).
    [...]
    7. Let currentLastIndex be ? Get(rx, "lastIndex").
    8. If SameValue(currentLastIndex, previousLastIndex) is false, then
       a. Perform ? Set(rx, "lastIndex", previousLastIndex, true).
    [...]
features: [Symbol.search]
---*/

var lastIndexValue;
var lastIndexValueAfterExec;
var lastIndexReads;
var lastIndexWrites;
var execCallCount;
var result;

var fakeRe = {
  get lastIndex() {
    lastIndexReads++;
    return lastIndexValue;
  },
  set lastIndex(_) {
    lastIndexWrites++;
    lastIndexValue = _;
  },
  exec: function() {
    execCallCount++;
    lastIndexValue = lastIndexValueAfterExec;
    return null;
  }
};

function reset(value, valueAfterExec) {
  lastIndexValue = value;
  lastIndexValueAfterExec = valueAfterExec;
  lastIndexReads = 0;
  lastIndexWrites = 0;
  execCallCount = 0;
}

reset(0, 0);
result = RegExp.prototype[Symbol.search].call(fakeRe);
assert.sameValue(result, -1);
assert.sameValue(lastIndexValue, 0);
assert.sameValue(lastIndexReads, 2);
assert.sameValue(lastIndexWrites, 0);
assert.sameValue(execCallCount, 1);

reset(420, 420);
result = RegExp.prototype[Symbol.search].call(fakeRe);
assert.sameValue(result, -1);
assert.sameValue(lastIndexValue, 420);
assert.sameValue(lastIndexReads, 2);
assert.sameValue(lastIndexWrites, 1);
assert.sameValue(execCallCount, 1);
