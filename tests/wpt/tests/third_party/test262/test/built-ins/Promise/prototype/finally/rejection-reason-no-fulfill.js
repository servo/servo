// Copyright (C) 2017 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
author: Jordan Harband
description: finally on a rejected promise can not convert to a fulfillment
esid: sec-promise.prototype.finally
features: [Promise.prototype.finally]
flags: [async]
includes: [promiseHelper.js]
---*/
var sequence = [];
var original = {};
var replacement = {};

var p = Promise.reject(original);

p.finally(function() {
  sequence.push(1);
  assert.sameValue(arguments.length, 0, 'onFinally receives zero args');
  return replacement;
}).then(function() {
  throw new Test262Error('promise is rejected pre-finally; onFulfill should not be called');
}).catch(function(reason) {
  sequence.push(2);
  assert.sameValue(reason, original, 'onFinally can not override the rejection value by returning');
}).then(function() {
  assert.sameValue(sequence.length, 2);
  checkSequence(sequence, "All expected callbacks called in correct order");
}).then($DONE, $DONE);
