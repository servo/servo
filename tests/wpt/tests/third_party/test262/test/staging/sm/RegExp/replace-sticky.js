// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  String.prototype.replace should use and update lastIndex if sticky flag is set
info: bugzilla.mozilla.org/show_bug.cgi?id=887016
esid: pending
---*/

var input = "abcdeabcdeabcdefghij";
var re = new RegExp("abcde", "y");
re.test(input);
assert.sameValue(re.lastIndex, 5);
var ret = input.replace(re, "ABCDE");
assert.sameValue(ret, "abcdeABCDEabcdefghij");
assert.sameValue(re.lastIndex, 10);
ret = input.replace(re, "ABCDE");
assert.sameValue(ret, "abcdeabcdeABCDEfghij");
assert.sameValue(re.lastIndex, 15);
ret = input.replace(re, "ABCDE");
assert.sameValue(ret, "abcdeabcdeabcdefghij");
assert.sameValue(re.lastIndex, 0);
