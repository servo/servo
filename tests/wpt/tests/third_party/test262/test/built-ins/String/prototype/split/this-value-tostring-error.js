// Copyright (C) 2020 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.split
description: Abrupt completion from ToString on the "this" value
info: |
  1. Let _O_ be ? RequireObjectCoercible(*this* value).
  1. If _separator_ is neither *undefined* nor *null*, then
    1. Let _splitter_ be ? GetMethod(_separator_, @@split).
    1. If _splitter_ is not *undefined*, then
      1. Return ? Call(_splitter_, _separator_, &laquo; _O_, _limit_ &raquo;).
  1. Let _S_ be ? ToString(_O_).
features: [Symbol, Symbol.split, Symbol.toPrimitive]
---*/

function ExpectedError(message) {
  this.message = message || "";
}
ExpectedError.prototype.toString = function () {
  return "ExpectedError: " + this.message;
};

var split = String.prototype.split;

var nonStringableReceiver = {};
nonStringableReceiver.toString = function() { throw new ExpectedError("receiver.toString"); };

var splitter = {};
splitter[Symbol.split] = function() {};

try {
  split.call(nonStringableReceiver, splitter, Symbol());
} catch (e) {
  assert.sameValue(e, undefined,
      'ToString should not be called on the receiver when the separator has a @@split method.');
}

var nonStringableSeparator = {};
nonStringableSeparator[Symbol.toPrimitive] =
  function() { throw new Test262Error("separator[Symbol.toPrimitive]"); };
nonStringableSeparator.toString = function() { throw new Test262Error("separator.toString"); };
nonStringableSeparator.valueOf = function() { throw new Test262Error("separator.valueOf"); };

assert.throws(ExpectedError, function() {
  split.call(nonStringableReceiver, nonStringableSeparator, Symbol());
}, 'ToString should be called on the receiver before processing the separator or limit.');
