// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-statement-runtime-semantics-labelledevaluation
description: Removal of lexical environment for the statement "head"
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
features: [let]
---*/

let x = 'outside';
var run = true;
var probeTest, probeIncr, probeBody;

for (
    let x = 'inside';
    (probeTest = function() { return x; }) && run;
    probeIncr = function() { return x; }
  )
  probeBody = function() { return x; }, run = false;

assert.sameValue(probeBody(), 'inside', 'reference from statement body');
assert.sameValue(probeIncr(), 'inside', 'reference from "increment" position');
assert.sameValue(probeTest(), 'inside', 'reference from "test" position');
assert.sameValue(x, 'outside');
