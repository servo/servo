// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asyncgeneratorfunction
description: Definition of instance `length` property
info: |
    AsyncGeneratorFunction ( p1, p2, … , pn, body )
    ...
    3. Return CreateDynamicFunction(C, NewTarget, "async generator", args).

    Runtime Semantics: CreateDynamicFunction
    ...
    29. If kind is "async" or "async generator", then
        a. If parameters Contains AwaitExpression is true, throw a SyntaxError
           exception.
features: [async-iteration]
flags: [async]
---*/

var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;

// AwaitExpression is permitted in function body.
AsyncGeneratorFunction('x = await 42');

assert.throws(SyntaxError, function() {
  AsyncGeneratorFunction('x = await 42', '');
}, 'AwaitExpression not permitted in parameters');

var withinAsyncGenerator = async function*() {
  AsyncGeneratorFunction('x = await 42', '');
};

withinAsyncGenerator().next().then(
 function () {
   throw new Test262Error("AwaitExpression not permitted when calling context is a async generator");
 },
 function (e) {
   if (!(e instanceof SyntaxError)) {
    throw new Test262Error("Expected SyntaxError but got " + e);
   }
 }
).then($DONE, $DONE);
