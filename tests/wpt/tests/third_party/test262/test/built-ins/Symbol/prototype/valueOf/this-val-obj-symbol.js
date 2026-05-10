// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol.prototype.valueof
description: Called on a Symbol Object value
info: |
  1. Let s be the this value.
  2. If Type(s) is Symbol, return s.
  3. If Type(s) is not Object, throw a TypeError exception.
  4. If s does not have a [[SymbolData]] internal slot, throw a TypeError exception.
  5. Return the value of s's [[SymbolData]] internal slot. 
features: [Symbol]
---*/

var valueOf = Symbol.prototype.valueOf;
var symbol = Symbol('s');
var symbolObject = Object(symbol);

assert.sameValue(valueOf.call(symbolObject), symbol);
