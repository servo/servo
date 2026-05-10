// Copyright 2015 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
 RegExp has a property at `Symbol.species`
esid: sec-get-regexp-@@species
author: Sam Mikes
description: RegExp[Symbol.species] exists per spec
includes: [propertyHelper.js]
features: [Symbol.species]
---*/

var desc = Object.getOwnPropertyDescriptor(RegExp, Symbol.species);

assert.sameValue(desc.set, undefined);
assert.sameValue(typeof desc.get, 'function');

verifyNotWritable(RegExp, Symbol.species, Symbol.species);
verifyNotEnumerable(RegExp, Symbol.species);
verifyConfigurable(RegExp, Symbol.species);
