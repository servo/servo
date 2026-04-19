// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-statement-runtime-semantics-labelledevaluation
description: >
    No variable environment is created for each evaluation of the statement
    body
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
       e. Perform ? CreatePerIterationEnvironment(perIterationBindings).
       [...]

    13.7.4.9 Runtime Semantics: CreatePerIterationEnvironment

    1. If perIterationBindings has any elements, then
       [...]
       e. Let thisIterationEnv be NewDeclarativeEnvironment(outer).
       f. Let thisIterationEnvRec be thisIterationEnv's EnvironmentRecord.
flags: [noStrict]
---*/

var probeBefore = function() { return [x, y, z]; };
var probeTest, probeIncr, probeBody;
var run = true;

for (
    ;
    run && (eval('var x = 1;'), probeTest = function() { return [x, y, z]; });
    eval('var y = 1;'), probeIncr = function() { return [x, y, z]; }
  )
  var z = 1, _ = (probeBody = function() { return [x, y, z]; }), run = false;

var x = 2;
var y = 2;
var z = 2;

assert.sameValue(
  probeBefore()[0],
  2,
  'reference preceding statement (redeclared in "test" position)'
);
assert.sameValue(
  probeBefore()[1],
  2,
  'reference preceding statement (redeclared in statement body)'
);
assert.sameValue(
  probeBefore()[2],
  2,
  'reference preceding statement (redeclared in "increment" position)'
);

assert.sameValue(
  probeTest()[0],
  2,
  'reference from "test" position (redeclared in "test" position)'
);
assert.sameValue(
  probeTest()[1],
  2,
  'reference from "test" position (redeclared in statement body)'
);
assert.sameValue(
  probeTest()[2],
  2,
  'reference from "test" position (redeclared in "increment" position)'
);

assert.sameValue(
  probeBody()[0],
  2,
  'reference from statement body (redeclared in "test" position)'
);
assert.sameValue(
  probeBody()[1],
  2,
  'reference from statement body (redeclared in statement body)'
);
assert.sameValue(
  probeBody()[2],
  2,
  'reference from statement body (redeclared in "increment" position)'
);

assert.sameValue(
  probeIncr()[0],
  2,
  'reference from "increment" position (redeclared in "test" position)'
);
assert.sameValue(
  probeIncr()[1],
  2,
  'reference from "increment" position (redeclared in statement body)'
);
assert.sameValue(
  probeIncr()[2],
  2,
  'reference from "increment" position (redeclared in "increment" position)'
);
