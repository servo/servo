// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function-definitions-runtime-semantics-evaluation
description: Removal of variable environment for the BindingIdentifier
info: |
    [...]
    2. Let scope be the running execution context's LexicalEnvironment.
    3. Let funcEnv be NewDeclarativeEnvironment(scope).
    4. Let envRec be funcEnv's EnvironmentRecord.
    5. Let name be StringValue of BindingIdentifier.
    6. Perform envRec.CreateImmutableBinding(name, false).
    7. Let closure be FunctionCreate(Normal, FormalParameters, FunctionBody,
       funcEnv, strict).
    [...]
---*/

var probe;

var func = function f() {
  probe = function() { return f; };
};
var f = 'outside';

func();

assert.sameValue(f, 'outside');
assert.sameValue(probe(), func);
