// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Implement RegExp.prototype[@@replace].
info: bugzilla.mozilla.org/show_bug.cgi?id=887016
esid: pending
---*/

assert.sameValue(RegExp.prototype[Symbol.replace].name, "[Symbol.replace]");
assert.sameValue(RegExp.prototype[Symbol.replace].length, 2);
var desc = Object.getOwnPropertyDescriptor(RegExp.prototype, Symbol.replace);
assert.sameValue(desc.configurable, true);
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.writable, true);

var re = /a/;
var v = re[Symbol.replace]("abcAbcABC", "X");
assert.sameValue(v, "XbcAbcABC");

re = /d/;
v = re[Symbol.replace]("abcAbcABC", "X");
assert.sameValue(v, "abcAbcABC");

re = /a/ig;
v = re[Symbol.replace]("abcAbcABC", "X");
assert.sameValue(v, "XbcXbcXBC");

re = /(a)(b)(cd)/;
v = re[Symbol.replace]("012abcd345", "_$$_$&_$`_$'_$0_$1_$2_$3_$4_$+_$");
assert.sameValue(v, "012_$_abcd_012_345_$0_a_b_cd_$4_$+_$345");

re = /(a)(b)(cd)/;
v = re[Symbol.replace]("012abcd345", "_\u3042_$$_$&_$`_$'_$0_$1_$2_$3_$4_$+_$");
assert.sameValue(v, "012_\u3042_$_abcd_012_345_$0_a_b_cd_$4_$+_$345");
