// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replace
description: >
  Non-callable replaceValue is evaluated before searching the string.
info: |
  String.prototype.replace ( searchValue, replaceValue )

  ...
  6. If functionalReplace is false, then
    a. Set replaceValue to ? ToString(replaceValue).
  7. Search string for the first occurrence of searchString and let pos be the
  index within string of the first code unit of the matched substring and let
  matched be searchString. If no occurrences of searchString were found, return string.
---*/

var calls = 0;
var replaceValue = {
  toString: function() {
    calls += 1;
    return "b";
  },
};

var newString = "".replace("a", replaceValue);
assert.sameValue(newString, "");
assert.sameValue(calls, 1);
