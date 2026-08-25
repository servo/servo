// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol.keyfor
description: Called with Symbol value that exists in the global symbol registry
info: |
    1. If Type(sym) is not Symbol, throw a TypeError exception.
    2. For each element e of the GlobalSymbolRegistry List (see 19.4.2.1),
       a. If SameValue(e.[[Symbol]], sym) is true, return e.[[Key]].
features: [Symbol]
---*/

var canonical = Symbol.for('s');

assert.sameValue(Symbol.keyFor(canonical), 's');
