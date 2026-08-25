// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
description: >
    Removal of lexical environment for the initial evaluation of the statement
    body
info: |
    IterationStatement : for ( ForDeclaration of AssignmentExpression ) Statement

    [...]
    2. Return ? ForIn/OfBodyEvaluation(ForDeclaration, Statement, keyResult,
       lexicalBinding, labelSet).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    5. Repeat
       [...]
       i. Let result be the result of evaluating stmt.
       j. Set the running execution context's LexicalEnvironment to oldEnv.
       k. If LoopContinues(result, labelSet) is false, return ?
          IteratorClose(iterator, UpdateEmpty(result, V)).
       l. If result.[[Value]] is not empty, let V be result.[[Value]].
features: [let]
---*/

let x = 'outside';
var probeDecl, probeBody;

for (
    let [x, _ = probeDecl = function() { return x; }]
    in
    { i: 0 }
  )
  probeBody = function() { return x; };

assert.sameValue(probeDecl(), 'i', 'reference from ForDeclaration');
assert.sameValue(probeBody(), 'i', 'reference from statement body');
assert.sameValue(x, 'outside');
