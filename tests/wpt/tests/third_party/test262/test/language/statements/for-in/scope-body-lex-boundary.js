// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
description: >
    Creation of new lexical environment for each evaluation of the statement
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
var probeFirst, probeSecond;

for (let x in { a: 0, b: 0 })
  if (!probeFirst)
    probeFirst = function() { return x; };
  else
    probeSecond = function() { return x; };


// 13.7.5.15 EnumerateObjectProperties
//
// > [...] The mechanics and order of enumerating the properties is not
// > specified [...]
assert.notSameValue(probeFirst(), probeSecond());
assert(
  probeFirst() === 'a' || probeFirst() === 'b',
  'First binding is either "a" or "b"'
);
assert(
  probeSecond() === 'a' || probeSecond() === 'b',
  'Second binding is either "a" or "b"'
);
