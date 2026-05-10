// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Generator is closed from suspended-start state and IteratorClose calls return.
info: |
  %IteratorHelperPrototype%.return ( )
    ...
    4. If O.[[GeneratorState]] is suspended-start, then
       a. Set O.[[GeneratorState]] to completed.
       ...
       c. Perform ? IteratorCloseAll(O.[[UnderlyingIterators]], ReturnCompletion(undefined)).
       d. Return CreateIteratorResultObject(undefined, true).
    5. Let C be ReturnCompletion(undefined).
    6. Return ? GeneratorResumeAbrupt(O, C, "Iterator Helper").

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
    8. Return ? completion.

  GeneratorResumeAbrupt ( generator, abruptCompletion, generatorBrand )
    1. Let state be ? GeneratorValidate(generator, generatorBrand).
    2. If state is suspended-start, then
      ...
    3. If state is completed, then
      a. If abruptCompletion is a return completion, then
        i. Return CreateIteratorResultObject(abruptCompletion.[[Value]], true).
      ...

  GeneratorValidate ( generator, generatorBrand )
    ...
    5. Let state be generator.[[GeneratorState]].
    6. If state is executing, throw a TypeError exception.
    7. Return state.
features: [joint-iteration]
---*/

var returnCallCount = 0;

var underlying = {
  next() {
    throw new Test262Error("Unexpected call to next");
  },
  return() {
    returnCallCount += 1;

    // The generator state is already set to "completed", so this `return()`
    // call proceeds to `GeneratorResumeAbrupt`. The generator state is not
    // "executing", so `GeneratorValidate` succeeds and `GeneratorResumeAbrupt`
    // returns with `CreateIteratorResultObject()`.
    var result = it.return();
    assert.sameValue(result.value, undefined);
    assert.sameValue(result.done, true);

    return {};
  },
};

var it = Iterator.zipKeyed({a: underlying});

assert.sameValue(returnCallCount, 0);

// This `return()` call sets the generator state to "completed" and then calls
// `IteratorClose()`.
var result = it.return();
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);

assert.sameValue(returnCallCount, 1);
