// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function-definitions-runtime-semantics-evaluation
description: >
    Creation of new variable environment for the BindingIdentifier
    parameter
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
flags: [noStrict]
---*/

var f = 'outside';
var probeBefore = function() { return f; };
var setBefore = function() { f = null; };
var probeParams, setParams, probeBody, setBody;

var func = function f(
  _ = (
    probeParams = function() { return f; },
    setParams = function() { f = null; }
    )
  ) {
  probeBody = function() { return f; };
  setBody = function() { f = null; };
};

func();

assert.sameValue(probeBefore(), 'outside');
setBefore();
assert.sameValue(probeBefore(), null);

assert.sameValue(probeParams(), func, 'inner binding value (from parameters)');
setParams();
assert.sameValue(
  probeParams(), func, 'inner binding is immutable (from parameters)'
);

assert.sameValue(probeBody(), func, 'inner binding value (from body)');
setBody();
assert.sameValue(probeBody(), func, 'inner binding is immutable (from body)');
