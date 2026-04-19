// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.4
description: >
    Symbol ToObject auto-boxing
flags: [noStrict]
features: [Symbol]
---*/

var sym = Symbol('66');

sym.a = 0;
assert.sameValue(sym.a, undefined, "The value of `sym.a` is `undefined`, after executing `sym.a = 0;`");

sym['a' + 'b'] = 0;
assert.sameValue(sym['a' + 'b'], undefined, "The value of `sym['a' + 'b']` is `undefined`, after executing `sym['a' + 'b'] = 0;`");

sym[62] = 0;
assert.sameValue(sym[62], undefined, "The value of `sym[62]` is `undefined`, after executing `sym[62] = 0;`");
