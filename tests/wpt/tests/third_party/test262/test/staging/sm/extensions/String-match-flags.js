// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  String.prototype.match with non-string non-standard flags argument.
info: bugzilla.mozilla.org/show_bug.cgi?id=1263139
esid: pending
---*/

var called;
var flags = {
  toString() {
    called = true;
    return "";
  }
};

called = false;
"a".match("a", flags);
assert.sameValue(called, false);

called = false;
"a".search("a", flags);
assert.sameValue(called, false);

called = false;
"a".replace("a", "b", flags);
assert.sameValue(called, false);
