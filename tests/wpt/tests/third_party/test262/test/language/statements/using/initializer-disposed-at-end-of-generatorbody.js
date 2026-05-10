// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-generatorstart
description: Initialized value is disposed at end of GeneratorBody
info: |
  GeneratorStart ( generator, generatorBody )
  
  1. Assert: The value of generator.[[GeneratorState]] is undefined.
  2. Let genContext be the running execution context.
  3. Set the Generator component of genContext to generator.
  4. Let closure be a new Abstract Closure with no parameters that captures generatorBody and performs the following steps when called:
    a. Let acGenContext be the running execution context.
    b. Let acGenerator be the Generator component of acGenContext.
    c. If generatorBody is a Parse Node, then
      i. Let result be Completion(Evaluation of generatorBody).
    d. Else,
      i. Assert: generatorBody is an Abstract Closure with no parameters.
      ii. Let result be generatorBody().
    e. Assert: If we return here, the generator either threw an exception or performed either an implicit or explicit return.
    f. Remove acGenContext from the execution context stack and restore the execution context that is at the top of the execution context stack as the running execution context.
    g. Set acGenerator.[[GeneratorState]] to completed.
    h. NOTE: Once a generator enters the completed state it never leaves it and its associated execution context is never resumed. Any execution state associated with acGenerator can be discarded at this point.
    i. Let env be genContext's LexicalEnvironment.
    j. If env is not undefined, then
      i. Assert: env is a Declarative Environment Record.
      i. Set result to DisposeResources(env.[[DisposeCapability]], result).
    k. If result.[[Type]] is normal, then
      i. Let resultValue be undefined.
    l. Else if result.[[Type]] is return, then
      i. Let resultValue be result.[[Value]].
    m. Else,
      i. Assert: result.[[Type]] is throw.
      ii. Return ? result.
    n. Return CreateIterResultObject(resultValue, true).
  5. Set the code evaluation state of genContext such that when evaluation is resumed for that execution context, closure will be called with no arguments.
  6. Set generator.[[GeneratorContext]] to genContext.
  7. Set generator.[[GeneratorState]] to suspendedStart.
  8. Return unused.

  DisposeResources ( disposeCapability, completion )

  1. For each resource of disposeCapability.[[DisposableResourceStack]], in reverse list order, do
    a. Let result be Dispose(resource.[[ResourceValue]], resource.[[Hint]], resource.[[DisposeMethod]]).
    b. If result.[[Type]] is throw, then
      i. If completion.[[Type]] is throw, then
        1. Set result to result.[[Value]].
        2. Let suppressed be completion.[[Value]].
        3. Let error be a newly created SuppressedError object.
        4. Perform ! CreateNonEnumerableDataPropertyOrThrow(error, "error", result).
        5. Perform ! CreateNonEnumerableDataPropertyOrThrow(error, "suppressed", suppressed).
        6. Set completion to ThrowCompletion(error).
      ii. Else,
        1. Set completion to result.
  2. Return completion.

  Dispose ( V, hint, method )

  1. If method is undefined, let result be undefined.
  2. Else, let result be ? Call(method, V).
  3. If hint is async-dispose, then
    a. ...
  4. Return undefined.

features: [explicit-resource-management]
---*/

var resource = {
    disposed: false,
    [Symbol.dispose]() {
        this.disposed = true;
    }
};

function * f() {
    using _ = resource;
    yield;
}

var g = f();
var wasDisposedBeforeGeneratorStarted = resource.disposed;
g.next();
var wasDisposedWhileSuspended = resource.disposed;
assert.sameValue(g.next().done, true);
var isDisposedAfterGeneratorCompleted = resource.disposed;

assert.sameValue(wasDisposedBeforeGeneratorStarted, false, 'Expected resource to not have been disposed prior to generator start');
assert.sameValue(wasDisposedWhileSuspended, false, 'Expected resource to not have been disposed while generator is suspended');
assert.sameValue(isDisposedAfterGeneratorCompleted, true, 'Expected resource to have been disposed after generator completed');
