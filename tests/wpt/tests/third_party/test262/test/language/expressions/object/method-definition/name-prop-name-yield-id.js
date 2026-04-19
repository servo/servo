// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    When the `yield` keyword occurs within the PropertyName of a
    non-generator MethodDefinition outside of a generator function, it behaves
    as an Identifier.
es6id: 14.3
flags: [noStrict]
---*/

var yield = 'propName';
var obj = {
  [yield]() {}
};

assert(
  Object.prototype.hasOwnProperty.call(obj, 'propName'),
  "The property name is taken from the 'yield' variable"
);
