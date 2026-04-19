// Copyright (C) 2018 Rick Waldron. All rights reserved.
// Copyright (C) 2018 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol.prototype.description
description: >
  SymbolDescriptiveString(sym) via Symbol.prototype.toString()
info: |
  SymbolDescriptiveString ( sym )

  Assert: Type(sym) is Symbol.
  Let desc be sym's [[Description]] value.
  If desc is undefined, let desc be the empty string.
  Assert: Type(desc) is String.
  Return the string-concatenation of "Symbol(", desc, and ")".

features: [Symbol.prototype.description]
---*/

const symbol = Symbol('foo');

assert.sameValue(
  symbol.description,
  'foo',
  'The value of symbol.description is "foo"'
);
assert.sameValue(
  symbol.toString(),
  `Symbol(${symbol.description})`,
  `symbol.toString() returns "Symbol(${symbol.description})"`
);
