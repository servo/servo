// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
description: >
    No variable environment is created for the statement "head"
info: |
    IterationStatement : for ( ForDeclaration of AssignmentExpression ) Statement

    1. Let keyResult be the result of performing ?
       ForIn/OfHeadEvaluation(BoundNames of ForDeclaration,
       AssignmentExpression, iterate).
    [...]

    13.7.5.12 Runtime Semantics: ForIn/OfHeadEvaluation

    [...]
    2. If TDZnames is not an empty List, then
       a. Assert: TDZnames has no duplicate entries.
       b. Let TDZ be NewDeclarativeEnvironment(oldEnv).
       c. Let TDZEnvRec be TDZ's EnvironmentRecord.
       d. For each string name in TDZnames, do
          i. Perform ! TDZEnvRec.CreateMutableBinding(name, false).
       e. Set the running execution context's LexicalEnvironment to TDZ.
    3. Let exprRef be the result of evaluating expr.
    [...]
flags: [noStrict]
---*/

var probeBefore = function() { return x; };
var x = 1;
var probeDecl, probeExpr, probeBody;

for (
    let [_ = probeDecl = function() { return x; }]
    of
    [[eval('var x = 2;'), probeExpr = function() { return x; }]]
  )
  probeBody = function() { return x; };

assert.sameValue(probeBefore(), 2, 'reference preceding statement');
assert.sameValue(probeDecl(), 2, 'reference from ForDeclaration');
assert.sameValue(probeExpr(), 2, 'reference from AssignmentExpression');
assert.sameValue(probeBody(), 2, 'reference from statement body');
assert.sameValue(x, 2, 'reference following statement');
