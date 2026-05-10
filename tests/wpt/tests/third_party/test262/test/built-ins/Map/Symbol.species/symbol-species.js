// Copyright 2015 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
 Map has a property at `Symbol.species`
esid: sec-get-map-@@species
author: Sam Mikes
description: Map[Symbol.species] exists per spec
includes: [propertyHelper.js]
features: [Symbol.species]
---*/

var desc = Object.getOwnPropertyDescriptor(Map, Symbol.species);

assert.sameValue(desc.set, undefined);
assert.sameValue(typeof desc.get, 'function');

verifyNotWritable(Map, Symbol.species, Symbol.species);
verifyNotEnumerable(Map, Symbol.species);
verifyConfigurable(Map, Symbol.species);
