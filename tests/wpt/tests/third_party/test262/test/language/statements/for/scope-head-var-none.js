// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-statement-runtime-semantics-labelledevaluation
description: No variable environment is created for the statement "head"
info: |
    [...]
    2. Let loopEnv be NewDeclarativeEnvironment(oldEnv).
    3. Let loopEnvRec be loopEnv's EnvironmentRecord.
    4. Let isConst be the result of performing IsConstantDeclaration of
       LexicalDeclaration.
    5. Let boundNames be the BoundNames of LexicalDeclaration.
    6. For each element dn of boundNames do
       a. If isConst is true, then
          i. Perform ! loopEnvRec.CreateImmutableBinding(dn, true).
       b. Else,
          i. Perform ! loopEnvRec.CreateMutableBinding(dn, false).
    7. Set the running execution context's LexicalEnvironment to loopEnv.
    [...]
    12. Set the running execution context's LexicalEnvironment to oldEnv.
    13. Return Completion(bodyResult).
flags: [noStrict]
---*/

var probeBefore = function() { return x; };
var probeTest, probeIncr, probeBody;
var run = true;

for (
    var _ = eval('var x = 1;');
    run && (probeTest = function() { return x; });
    probeIncr = function() { return x; }
  )
  probeBody = function() { return x; }, run = false;

var x = 2;

assert.sameValue(probeBefore(), 2, 'reference preceding statement');
assert.sameValue(probeTest(), 2, 'reference from "test" position');
assert.sameValue(probeBody(), 2, 'reference from statement body');
assert.sameValue(probeIncr(), 2, 'reference from "increment" position');
assert.sameValue(x, 2, 'reference following statement');
