// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function-calls-runtime-semantics-evaluation
description: >
  Direct eval call with empty trailing spread.
info: |
  12.3.4.1 Runtime Semantics: Evaluation
    ...
    3. If Type(ref) is Reference and IsPropertyReference(ref) is false and GetReferencedName(ref) is "eval", then
      a. If SameValue(func, %eval%) is true, then
        i. Let argList be ? ArgumentListEvaluation(Arguments).
        ii. If argList has no elements, return undefined.
        iii. Let evalText be the first element of argList.
        ...

features: [Symbol.iterator]
---*/

var nextCount = 0;
var iter = {};
iter[Symbol.iterator] = function() {
  return {
    next: function() {
      var i = nextCount++;
      return {done: true, value: undefined};
    }
  };
};

var x = "global";

(function() {
  var x = "local";
  eval("x = 0;", ...iter);
  assert.sameValue(x, 0);
})();

assert.sameValue(x, "global");
assert.sameValue(nextCount, 1);
