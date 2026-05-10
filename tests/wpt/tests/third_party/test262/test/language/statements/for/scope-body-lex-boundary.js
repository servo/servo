// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-statement-runtime-semantics-labelledevaluation
description: >
    Creation of new lexical environment for each evaluation of the statement
    body
info: |
    [...]
    11. Let bodyResult be ForBodyEvaluation(the first Expression, the second
        Expression, Statement, perIterationLets, labelSet).
    [...]

    13.7.4.8 Runtime Semantics: ForBodyEvaluation

    [...]
    3. Repeat
       [...]
       b. Let result be the result of evaluating stmt.
       [...]
       e. Perform ? CreatePerIterationEnvironment(perIterationBindings).
       [...]

    13.7.4.9 Runtime Semantics: CreatePerIterationEnvironment

    1. If perIterationBindings has any elements, then
       [...]
       e. Let thisIterationEnv be NewDeclarativeEnvironment(outer).
       f. Let thisIterationEnvRec be thisIterationEnv's EnvironmentRecord.
features: [let]
---*/

var probeFirst;
var probeSecond = null;

for (let x = 'first'; probeSecond === null; x = 'second')
  if (!probeFirst)
    probeFirst = function() { return x; };
  else
    probeSecond = function() { return x; };

assert.sameValue(probeFirst(), 'first');
assert.sameValue(probeSecond(), 'second');
