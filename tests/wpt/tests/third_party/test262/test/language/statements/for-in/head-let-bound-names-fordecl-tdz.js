// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.12_S2
description: >
    ForIn/Of: Bound names of ForDeclaration are in TDZ (for-of)
---*/

assert.throws(ReferenceError, function() {
  let x = 1;
  for (let x in { x }) {}
});
