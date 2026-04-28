// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-with-statement-runtime-semantics-evaluation
es6id: 13.11.7
description: Removal of variable environment
info: |
    3. Let oldEnv be the running execution context's LexicalEnvironment.
    4. Let newEnv be NewObjectEnvironment(obj, oldEnv).
    5. Set the withEnvironment flag of newEnv's EnvironmentRecord to true.
    6. Set the running execution context's LexicalEnvironment to newEnv.
    7. Let C be the result of evaluating Statement.
    8. Set the running execution context's LexicalEnvironment to oldEnv.
flags: [noStrict]
---*/

var probeBody;

with ({ x: 0 })
  var x = 1, _ = probeBody = function() { return x; };

var x = 2;

assert.sameValue(probeBody(), 1, 'reference from statement body');
assert.sameValue(x, 2, 'reference following statement');
