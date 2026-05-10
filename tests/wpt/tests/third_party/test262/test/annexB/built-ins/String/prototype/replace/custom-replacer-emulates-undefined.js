// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.replace
description: >
  [[IsHTMLDDA]] object as @@replace method gets called.
info: |
  String.prototype.replace ( searchValue, replaceValue )

  [...]
  2. If searchValue is neither undefined nor null, then
    a. Let replacer be ? GetMethod(searchValue, @@replace).
    b. If replacer is not undefined, then
      i. Return ? Call(replacer, searchValue, « O, replaceValue »).
features: [Symbol.replace, IsHTMLDDA]
---*/

var searchValue = $262.IsHTMLDDA;
var replacerGets = 0;
Object.defineProperty(searchValue, Symbol.replace, {
  get: function() {
    replacerGets += 1;
    return searchValue;
  },
  configurable: true,
});

assert.sameValue("".replace(searchValue), null);
assert.sameValue(replacerGets, 1);
