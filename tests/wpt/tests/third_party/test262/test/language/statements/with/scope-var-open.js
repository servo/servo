// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-with-statement-runtime-semantics-evaluation
es6id: 13.11.7
description: Creation of new variable environment
info: |
    3. Let oldEnv be the running execution context's LexicalEnvironment.
    4. Let newEnv be NewObjectEnvironment(obj, oldEnv).
    5. Set the withEnvironment flag of newEnv's EnvironmentRecord to true.
    6. Set the running execution context's LexicalEnvironment to newEnv.
    7. Let C be the result of evaluating Statement.
flags: [noStrict]
---*/

var x = 0;
var objectRecord = { x: 2 };
var probeBefore = function() { return x; };
var probeExpr, probeBody;

with (eval('var x = 1;'), probeExpr = function() { return x; }, objectRecord)
  var x = 3, _ = probeBody = function() { return x; };

assert.sameValue(probeBefore(), 1, 'reference preceding statement');
assert.sameValue(probeExpr(), 1, 'reference from expression');
assert.sameValue(probeBody(), 3, 'reference from statement body');
