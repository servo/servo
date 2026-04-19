// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
description: No variable environment is created for the statement body
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
var probeExpr, probeDecl, probeBody;
var x = 1;

for (
    let [_ = probeDecl = function() { return x; }]
    in
    { '': probeExpr = function() { return x; }}
  )
  var x = 2, __ = probeBody = function() { return x; };


assert.sameValue(probeBefore(), 2, 'reference preceding statement');
assert.sameValue(probeExpr(), 2, 'reference from AssignmentExpression');
assert.sameValue(probeDecl(), 2, 'reference from ForDeclaration');
assert.sameValue(probeBody(), 2, 'reference from statement body');
assert.sameValue(x, 2, 'reference following statement');
