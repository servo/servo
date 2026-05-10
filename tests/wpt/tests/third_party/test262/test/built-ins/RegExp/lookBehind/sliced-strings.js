// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assertion
description: Sliced strings
info: |
  Rationale from https://github.com/tc39/test262/pull/999#discussion_r113807747

  Since this test originates from V8, this targets V8's sliced strings, which are used for
  substrings above a length of 13 characters. I wrote this test for exactly the reason
  @littledan mentioned. That's why the variable name is called oob_subject. The underlying string
  backing store extends beyond the actual boundary of the sliced string.
features: [regexp-lookbehind]
---*/

var oob_subject = "abcdefghijklmnabcdefghijklmn".slice(14);
assert.sameValue(oob_subject.match(/(?=(abcdefghijklmn))(?<=\1)a/i), null, "");
assert.sameValue(oob_subject.match(/(?=(abcdefghijklmn))(?<=\1)a/), null, "");
assert.sameValue("abcdefgabcdefg".slice(1).match(/(?=(abcdefg))(?<=\1)/), null, "");
