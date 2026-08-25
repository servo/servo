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
          ii. If argList has no elements, return undefined.
---*/

assert.sameValue(eval(), undefined);
