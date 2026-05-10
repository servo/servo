// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Evaluated code honors the strictness of the calling context
esid: sec-function-calls-runtime-semantics-evaluation
info: |
    [...]
    3. If Type(ref) is Reference and IsPropertyReference(ref) is false and
       GetReferencedName(ref) is "eval", then
       a. If SameValue(func, %eval%) is true, then
          [...]
          iv. If the source code matching this CallExpression is strict code,
              let strictCaller be true. Otherwise let strictCaller be false.
          [...]
flags: [onlyStrict]
---*/

assert.throws(SyntaxError, function() {
  eval('var static;');
});

assert.throws(SyntaxError, function() {
  eval('with ({}) {}');
});

assert.throws(ReferenceError, function() {
  eval('unresolvable = null;');
});
