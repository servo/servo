// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-variablestatements-in-catch-blocks
es6id: B.3.5
description: Re-declaration of catch parameter (for-in statement)
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
  for (var err in { propertyName: null }) {
    during = err;
  }
  after = err;
}

assert.sameValue(before, 'exception');
assert.sameValue(during, 'propertyName', 'during loop body evaluation');
assert.sameValue(after, 'propertyName', 'after loop body evaluation');
