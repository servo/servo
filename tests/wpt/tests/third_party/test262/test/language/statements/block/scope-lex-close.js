// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-block-runtime-semantics-evaluation
description: Removal of lexical environment for BlockStatement
info: |
    1. Let oldEnv be the running execution context's LexicalEnvironment.
    2. Let blockEnv be NewDeclarativeEnvironment(oldEnv).
    3. Perform BlockDeclarationInstantiation(StatementList, blockEnv).
    4. Set the running execution context's LexicalEnvironment to blockEnv.
    5. Let blockValue be the result of evaluating StatementList.
    6. Set the running execution context's LexicalEnvironment to oldEnv.
    7. Return blockValue.
features: [let]
---*/

var probe;

{
  let x = 'inside';
  probe = function() { return x; };
}

let x = 'outside';

assert.sameValue(x, 'outside');
assert.sameValue(probe(), 'inside');
