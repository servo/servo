// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.2.4
description: >
  Returns abrupt completion from ToObject(template).
info: |
  21.1.2.4 String.raw ( template , ...substitutions )

  1. Let substitutions be a List consisting of all of the arguments passed to
  this function, starting with the second argument. If fewer than two arguments
  were passed, the List is empty.
  2. Let numberOfSubstitutions be the number of elements in substitutions.
  3. Let cooked be ToObject(template).
  4. ReturnIfAbrupt(cooked).
---*/

assert.throws(TypeError, function() {
  String.raw(null);
});

assert.throws(TypeError, function() {
  String.raw(undefined);
});
