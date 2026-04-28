// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-block-runtime-semantics-evaluation
description: Retainment of existing variable environment for BlockStatement
info: |
    1. Let oldEnv be the running execution context's LexicalEnvironment.
    2. Let blockEnv be NewDeclarativeEnvironment(oldEnv).
    3. Perform BlockDeclarationInstantiation(StatementList, blockEnv).
    4. Set the running execution context's LexicalEnvironment to blockEnv.
    5. Let blockValue be the result of evaluating StatementList.
    6. Set the running execution context's LexicalEnvironment to oldEnv.
    7. Return blockValue.
---*/

var x = 'outside';
var probeBefore = function() { return x; };
var probeInside;

{
  var x = 'inside';
  probeInside = function() { return x; };
}

assert.sameValue(probeBefore(), 'inside', 'reference preceding statement');
assert.sameValue(probeInside(), 'inside', 'reference within statement');
assert.sameValue(x, 'inside', 'reference following statement');
