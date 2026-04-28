// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.split
description: >
  [[IsHTMLDDA]] object as @@split method gets called.
info: |
  String.prototype.split ( separator, limit )

  [...]
  2. If separator is neither undefined nor null, then
    a. Let splitter be ? GetMethod(separator, @@split).
    b. If splitter is not undefined, then
      i. Return ? Call(splitter, separator, « O, limit »).
features: [Symbol.split, IsHTMLDDA]
---*/

var separator = $262.IsHTMLDDA;
var splitterGets = 0;
Object.defineProperty(separator, Symbol.split, {
  get: function() {
    splitterGets += 1;
    return separator;
  },
  configurable: true,
});

assert.sameValue("".split(separator), null);
assert.sameValue(splitterGets, 1);
