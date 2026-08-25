// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-ecmascript-function-objects-call-thisargument-argumentslist
description: >
    Creation of new variable environment for the function parameters and body
    (as distinct from that for the function's BindingIdentifier)
info: |
    [...]
    3. Let callerContext be the running execution context.
    4. Let calleeContext be PrepareForOrdinaryCall(F, undefined).
    [...]

    9.2.1.1 PrepareForOrdinaryCall

    [...]
    8. Let localEnv be NewFunctionEnvironment(F, newTarget).
    9. Set the LexicalEnvironment of calleeContext to localEnv.
    10. Set the VariableEnvironment of calleeContext to localEnv.
    [...]
features: [let]
---*/

var n = 'outside';
var probeBefore = function() { return n; };
var probeInside;

// This test intentionally elides parameter expressions because their presence
// triggers the creation of an additional LexicalEnvironment dedicated to the
// function body (see sec-functiondeclarationinstantiation)
var func = function n() {
  let n = 'inside';
  probeInside = function() { return n; };
};

func();

assert.sameValue(probeBefore(), 'outside');
assert.sameValue(probeInside(), 'inside');
