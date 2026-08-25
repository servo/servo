// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-statement-runtime-semantics-labelledevaluation
description: >
    Creation of new lexical environment for the initial evaluation of the
    statement body
info: |
    [...]
    11. Let bodyResult be ForBodyEvaluation(the first Expression, the second
        Expression, Statement, perIterationLets, labelSet).
    [...]

    13.7.4.8 Runtime Semantics: ForBodyEvaluation

    [...]
    2. Perform ? CreatePerIterationEnvironment(perIterationBindings).
    3. Repeat
       [...]
       b. Let result be the result of evaluating stmt.
       [...]
     [...]

    13.7.4.9 Runtime Semantics: CreatePerIterationEnvironment

    1. If perIterationBindings has any elements, then
       [...]
       e. Let thisIterationEnv be NewDeclarativeEnvironment(outer).
       f. Let thisIterationEnvRec be thisIterationEnv's EnvironmentRecord.
features: [let]
---*/

var probeBefore, probeTest, probeIncr, probeBody;
var run = true;

for (
    let x = 'outside', _ = probeBefore = function() { return x; };
    run && (x = 'inside', probeTest = function() { return x; });
    probeIncr = function() { return x; }
  )
  probeBody = function() { return x; }, run = false;

assert.sameValue(probeBefore(), 'outside');
assert.sameValue(probeTest(), 'inside', 'reference from "test" position');
assert.sameValue(probeBody(), 'inside', 'reference from statement body');
assert.sameValue(probeIncr(), 'inside', 'reference from "increment" position');
