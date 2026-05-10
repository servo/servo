// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Call RegExp.prototype[@@replace] from String.prototype.replace.
info: bugzilla.mozilla.org/show_bug.cgi?id=887016
esid: pending
---*/

var called = 0;
var myRegExp = {
  [Symbol.replace](S, R) {
    assert.sameValue(S, "abcAbcABC");
    assert.sameValue(R, "foo");
    called++;
    return 42;
  }
};
assert.sameValue("abcAbcABC".replace(myRegExp, "foo"), 42);
assert.sameValue(called, 1);
