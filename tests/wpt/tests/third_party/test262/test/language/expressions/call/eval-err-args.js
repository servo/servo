// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Error evaluating arguments list for direct eval
esid: sec-function-calls-runtime-semantics-evaluation
info: |
    [...]
    3. If Type(ref) is Reference and IsPropertyReference(ref) is false and
       GetReferencedName(ref) is "eval", then
       a. If SameValue(func, %eval%) is true, then
          i. Let argList be ? ArgumentListEvaluation(Arguments).

     12.3.6.1 Runtime Semantics: ArgumentListEvaluation

     ArgumentList : AssignmentExpression

     1. Let ref be the result of evaluating AssignmentExpression.
     2. Let arg be ? GetValue(ref).
---*/

assert.throws(ReferenceError, function() {
  eval(unresolvable);
});
