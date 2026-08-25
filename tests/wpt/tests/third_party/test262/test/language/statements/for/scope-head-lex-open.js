// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-statement-runtime-semantics-labelledevaluation
description: Creation of new lexical environment for the statement "head"
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
features: [let]
---*/

let x = 'outside';
var probeBefore = function() { return x; };
var probeDecl, probeTest, probeIncr, probeBody;
var run = true;

for (
    let x = 'inside', _ = probeDecl = function() { return x; };
    run && (probeTest = function() { return x; });
    probeIncr = function() { return x; }
  )
  probeBody = function() { return x; }, run = false;

assert.sameValue(probeBefore(), 'outside');
assert.sameValue(probeDecl(), 'inside', 'reference from LexicalDeclaration');
assert.sameValue(probeTest(), 'inside', 'reference from "test" position');
assert.sameValue(probeBody(), 'inside', 'reference from statement body');
assert.sameValue(probeIncr(), 'inside', 'reference from "increment" position');
