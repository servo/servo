// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replace
description: >
  Non-callable replaceValue is evaluated via toString
info: |
  String.prototype.replace ( searchValue, replaceValue )

  If functionalReplace is false, then
    Set replaceValue to ? ToString(replaceValue).
---*/

let calls = 0;
let replaceValue = /$/;
let oldToString = replaceValue.toString.bind(replaceValue);

replaceValue.toString = () => {
  calls += 1;
  return oldToString();
};

let newString = "".replace("a", replaceValue);
assert.sameValue(newString, "");
assert.sameValue(calls, 1);
assert.sameValue("dollar".replace("dollar", /$/), "/$/");
