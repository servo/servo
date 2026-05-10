// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Abrupt completions resulting from value retrieval are forwarded to the
    runtime.
esid: sec-moduleevaluation
info: |
    [...]
    16. Let result be the result of evaluating module.[[ECMAScriptCode]].
    [...]

    15.2.3.11 Runtime Semantics: Evaluation

    ExportDeclaration : export default AssignmentExpression;

    [...]
    1. Let rhs be the result of evaluating AssignmentExpression.
    2. Let value be ? GetValue(rhs).

    6.2.3.1 GetValue (V)

    1. ReturnIfAbrupt(V).
    2. If Type(V) is not Reference, return V.
    3. Let base be GetBase(V).
    4. If IsUnresolvableReference(V) is true, throw a ReferenceError exception.
negative:
  phase: runtime
  type: ReferenceError
flags: [module]
---*/

export default unresolvable;
