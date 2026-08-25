// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.4.4.6
description: >
  Promise[Symbol.species] accessor property get name
info: |
  25.4.4.6 get Promise [ @@species ]

  ...
  The value of the name property of this function is "get [Symbol.species]".
features: [Symbol.species]
---*/

var descriptor = Object.getOwnPropertyDescriptor(Promise, Symbol.species);

assert.sameValue(
  descriptor.get.name,
  'get [Symbol.species]'
);
