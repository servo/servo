// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Generator is closed from suspended-yield state and IteratorClose calls return.
info: |
  %IteratorHelperPrototype%.return ( )
    ...
    5. Let C be ReturnCompletion(undefined).
    6. Return ? GeneratorResumeAbrupt(O, C, "Iterator Helper").

  GeneratorResumeAbrupt ( generator, abruptCompletion, generatorBrand )
    1. Let state be ? GeneratorValidate(generator, generatorBrand).
    ...
    4. Assert: state is suspended-yield.
    ...
    8. Set generator.[[GeneratorState]] to executing.
    ...
    10. Resume the suspended evaluation of genContext using abruptCompletion as
        the result of the operation that suspended it. Let result be the
        Completion Record returned by the resumed computation.
    ...

  GeneratorValidate ( generator, generatorBrand )
    ...
    5. Let state be generator.[[GeneratorState]].
    6. If state is executing, throw a TypeError exception.
    7. Return state.

  IteratorZip ( iters, mode, padding, finishResults )
    ...
    3. Let closure be a new Abstract Closure with no parameters that captures
       iters, iterCount, openIters, mode, padding, and finishResults, and
       performs the following steps when called:
      ...
      b. Repeat,
        ...
        v. Let completion be Completion(Yield(results)).
        vi. If completion is an abrupt completion, then
          1. Return ? IteratorCloseAll(openIters, completion).
    ...

  IteratorCloseAll ( iters, completion )
    1. For each element iter of iters, in reverse List order, do
      a. Set completion to Completion(IteratorClose(iter, completion)).
    2. Return ? completion.

  IteratorClose ( iteratorRecord, completion )
    ...
    3. Let innerResult be Completion(GetMethod(iterator, "return")).
    4. If innerResult is a normal completion, then
       a. Let return be innerResult.[[Value]].
       b. If return is undefined, return ? completion.
       c. Set innerResult to Completion(Call(return, iterator)).
    ...
features: [joint-iteration]
---*/

var returnCallCount = 0;

var underlying = {
  next() {
    return {value: 123, done: false};
  },
  return() {
    returnCallCount += 1;

    // The generator state is set to "executing", so this `return()` call throws
    // a TypeError when `GeneratorResumeAbrupt` performs `GeneratorValidate`.
    assert.throws(TypeError, function() {
      it.return();
    });

    return {};
  },
};

var it = Iterator.zipKeyed({a: underlying});

// Move generator into "suspended-yield" state.
var result = it.next();
assert.sameValue(result.value.a, 123);
assert.sameValue(result.done, false);

assert.sameValue(returnCallCount, 0);

// This `return()` call continues execution in the suspended generator.
var result = it.return();
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);

assert.sameValue(returnCallCount, 1);
