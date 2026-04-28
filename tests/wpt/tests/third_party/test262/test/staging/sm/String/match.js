// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Call RegExp.prototype[@@match] from String.prototype.match.
info: bugzilla.mozilla.org/show_bug.cgi?id=887016
esid: pending
---*/

var called = 0;
var myRegExp = {
  [Symbol.match](S) {
    assert.sameValue(S, "abcAbcABC");
    called++;
    return 42;
  }
};
assert.sameValue("abcAbcABC".match(myRegExp), 42);
assert.sameValue(called, 1);

var origMatch = RegExp.prototype[Symbol.match];

called = 0;
RegExp.prototype[Symbol.match] = function(S) {
  assert.sameValue(S, "abcAbcABC");
  called++;
  return 43;
};
assert.sameValue("abcAbcABC".match("abc"), 43);
assert.sameValue(called, 1);

RegExp.prototype[Symbol.match] = origMatch;
