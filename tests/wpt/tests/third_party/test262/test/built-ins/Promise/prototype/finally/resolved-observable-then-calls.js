// Copyright (C) 2017 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
author: Jordan Harband
description: finally observably calls .then
esid: sec-promise.prototype.finally
features: [Promise.prototype.finally]
flags: [async]
includes: [promiseHelper.js]
---*/
var sequence = [];
var yesValue = {};
var yes = Promise.resolve(yesValue);
yes.then = function() {
  sequence.push(1);
  return Promise.prototype.then.apply(this, arguments);
};

var noReason = {};
var no = Promise.reject(noReason);
no.then = function() {
  sequence.push(4);
  return Promise.prototype.then.apply(this, arguments);
};

yes.then(function(x) {
  sequence.push(2);
  assert.sameValue(x, yesValue);
  return x;
}).finally(function() {
  sequence.push(3);
  return no;
}).catch(function(e) {
  sequence.push(5);
  assert.sameValue(e, noReason);
}).then(function() {
  assert.sameValue(sequence.length, 5);
  checkSequence(sequence, "All expected callbacks called in correct order");
}).then($DONE, $DONE);
