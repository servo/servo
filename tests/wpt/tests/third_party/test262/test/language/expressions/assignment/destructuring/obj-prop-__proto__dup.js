// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-destructuring-assignment
es6id: 12.14.5
description: Duplicate __proto__ property names
info: |
    Annex B defines an early error for duplicate PropertyName of `__proto__`,
    in object initializers, but this does not apply to Object Assignment
    patterns
---*/

// Explicitly define an "own" property to avoid Annex B "__proto__ Property
// Names in Object Initializers" semantics (in environments which implement
// that extension)
var value = Object.defineProperty({}, '__proto__', { value: 123 });
var result, x, y;

result = { __proto__: x, __proto__: y } = value;

assert.sameValue(result, value);
assert.sameValue(x, 123, 'first AssignmentProperty');
assert.sameValue(y, 123, 'second AssignmentProperty');

result = x = y = null;

// CoverParenthesizedExpressionAndArrowParameterList
result = ({ __proto__: x, __proto__: y } = value);

assert.sameValue(result, value);
assert.sameValue(x, 123, 'first AssignmentProperty (CPEAAPL)');
assert.sameValue(y, 123, 'second AssignmentProperty (CPEAAPL)');
