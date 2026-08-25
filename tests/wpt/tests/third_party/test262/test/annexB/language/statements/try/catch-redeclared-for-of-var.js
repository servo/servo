// Copyright (C) 2019 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
author: Ross Kirsling
esid: sec-variablestatements-in-catch-blocks
description: Re-declaration of catch parameter (for-of statement)
info: |
    It is a Syntax Error if any element of the BoundNames of CatchParameter
    also occurs in the VarDeclaredNames of Block, unless CatchParameter is
    CatchParameter : BindingIdentifier.
---*/

var before, during, after;

try {
  throw 'exception';
} catch (err) {
  before = err;
  for (var err of [2]) {
    during = err;
  }
  after = err;
}

assert.sameValue(before, 'exception');
assert.sameValue(during, 2, 'during loop body evaluation');
assert.sameValue(after, 2, 'after loop body evaluation');
