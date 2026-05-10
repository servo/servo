// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.search
description: >
  [[IsHTMLDDA]] object as @@search method gets called.
info: |
  String.prototype.search ( regexp )

  [...]
  2. If regexp is neither undefined nor null, then
    a. Let searcher be ? GetMethod(regexp, @@search).
    b. If searcher is not undefined, then
      i. Return ? Call(searcher, regexp, « O »).
features: [Symbol.search, IsHTMLDDA]
---*/

var regexp = $262.IsHTMLDDA;
var searcherGets = 0;
Object.defineProperty(regexp, Symbol.search, {
  get: function() {
    searcherGets += 1;
    return regexp;
  },
  configurable: true,
});

assert.sameValue("".search(regexp), null);
assert.sameValue(searcherGets, 1);
