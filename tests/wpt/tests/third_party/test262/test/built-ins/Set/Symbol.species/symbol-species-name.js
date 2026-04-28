// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-set-@@species
description: >
  Set[Symbol.species] accessor property get name
info: |
  23.2.2.2 get Set [ @@species ]

  ...
  The value of the name property of this function is "get [Symbol.species]".
features: [Symbol.species]
---*/

var descriptor = Object.getOwnPropertyDescriptor(Set, Symbol.species);

assert.sameValue(
  descriptor.get.name,
  'get [Symbol.species]'
);
