// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
description: >
    Creation of new lexical environment for the initial evaluation of the
    statement body
info: |
    IterationStatement : for ( ForDeclaration of AssignmentExpression ) Statement

    [...]
    2. Return ? ForIn/OfBodyEvaluation(ForDeclaration, Statement, keyResult,
       lexicalBinding, labelSet).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    5. Repeat
       [...]
       d. If lhsKind is either assignment or varBinding, then
          [...]
       e. Else,
          i. Assert: lhsKind is lexicalBinding.
          ii. Assert: lhs is a ForDeclaration.
          iii. Let iterationEnv be NewDeclarativeEnvironment(oldEnv).
          iv. Perform BindingInstantiation for lhs passing iterationEnv as the
              argument.
          v. Set the running execution context's LexicalEnvironment to
             iterationEnv.
          [...]
features: [let]
---*/

var probeBefore = function() { return x; };
let x = 'outside';
var probeExpr, probeDecl, probeBody;

for (
    let [x, _ = probeDecl = function() { return x; }]
    in
    { i: probeExpr = function() { typeof x; }}
  )
  probeBody = function() { return x; };

assert.sameValue(probeBefore(), 'outside');
assert.throws(ReferenceError, probeExpr);
assert.sameValue(probeDecl(), 'i', 'reference from ForDeclaration');
assert.sameValue(probeBody(), 'i', 'reference from statement body');
