// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Implement RegExp.prototype[@@search].
info: bugzilla.mozilla.org/show_bug.cgi?id=887016
esid: pending
---*/

assert.sameValue(RegExp.prototype[Symbol.search].name, "[Symbol.search]");
assert.sameValue(RegExp.prototype[Symbol.search].length, 1);
var desc = Object.getOwnPropertyDescriptor(RegExp.prototype, Symbol.search);
assert.sameValue(desc.configurable, true);
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.writable, true);

var re = /B/;
var v = re[Symbol.search]("abcAbcABC");
assert.sameValue(v, 7);

re = /B/i;
v = re[Symbol.search]("abcAbcABCD");
assert.sameValue(v, 1);

re = /d/;
v = re[Symbol.search]("abcAbcABCD");
assert.sameValue(v, -1);
