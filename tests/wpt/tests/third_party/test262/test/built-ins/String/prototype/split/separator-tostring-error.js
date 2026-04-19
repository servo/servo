// Copyright (C) 2020 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.split
description: Abrupt completion from ToString on the separator
info: |
  1. Let _R_ be ? ToString(_separator_).
  1. If _lim_ = 0, return _A_.
---*/

function ExpectedError(message) {
  this.message = message || "";
}
ExpectedError.prototype.toString = function () {
  return "ExpectedError: " + this.message;
};

var nonStringableSeparator = {};
nonStringableSeparator.toString = function() { throw new ExpectedError(); };

assert.throws(ExpectedError, function() {
  "foo".split(nonStringableSeparator, 0);
}, 'ToString should be called on the separator before checking if the limit is zero.');
