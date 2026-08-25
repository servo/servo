// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimend
description: Type error when "this" value is a Symbol
info: |
  Runtime Semantics: TrimString ( string, where )
  2. Let S be ? ToString(str).

  ToString ( argument )
  Argument Type: Symbol
  Result: Throw a TypeError exception
features: [string-trimming, String.prototype.trimEnd]
---*/

assert.sameValue(typeof String.prototype.trimEnd, "function");

var trimEnd = String.prototype.trimEnd;
var symbol = Symbol();

assert.throws(
  TypeError,
  function() {
    trimEnd.call(symbol);
  },
  'String.prototype.trimEnd.call(Symbol())'
);
