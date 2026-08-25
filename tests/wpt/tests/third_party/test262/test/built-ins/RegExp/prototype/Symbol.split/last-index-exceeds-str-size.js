// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@split
description: The `lastIndex` property is clamped to the string size.
info: |
  RegExp.prototype [ @@split ] ( string, limit )

  ...
  19. Repeat, while q < size
    ...
    d. Else z is not null,
      i. Let e be ? ToLength(Get(splitter, "lastIndex")).
      ii. Let e be min(e, size).
  ...
features: [Symbol.split]
---*/

var regExp = /a/;
var string = "foo";

RegExp.prototype.exec = function() {
  this.lastIndex = 100;
  return {length: 0, index: 0};
};

var result = regExp[Symbol.split](string);

assert.sameValue(result.length, 2, "result.length");
assert.sameValue(result[0], "", "result[0]");
assert.sameValue(result[1], "", "result[1]");
