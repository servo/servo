// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.4.4.6
description: Promise `Symbol.species` property
info: |
    Promise[@@species] is an accessor property whose set accessor function is
    undefined.

    ES6 Section 17:

    Every accessor property described in clauses 18 through 26 and in Annex B.2
    has the attributes {[[Enumerable]]: false, [[Configurable]]: true } unless
    otherwise specified.
features: [Symbol.species]
includes: [propertyHelper.js]
---*/

var desc = Object.getOwnPropertyDescriptor(Promise, Symbol.species);

assert.sameValue(desc.set, undefined);
assert.sameValue(typeof desc.get, 'function');

verifyNotEnumerable(Promise, Symbol.species);
verifyConfigurable(Promise, Symbol.species);
