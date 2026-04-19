// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.replaceall
description: >
  [[IsHTMLDDA]] object as @@replace method gets called.
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  [...]
  2. If searchValue is neither undefined nor null, then
    [...]
    c. Let replacer be ? GetMethod(searchValue, @@replace).
    d. If replacer is not undefined, then
      i. Return ? Call(replacer, searchValue, « O, replaceValue »).
features: [Symbol.replace, String.prototype.replaceAll, IsHTMLDDA]
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

assert.sameValue("".replaceAll(searchValue), null);
assert.sameValue(replacerGets, 1);
