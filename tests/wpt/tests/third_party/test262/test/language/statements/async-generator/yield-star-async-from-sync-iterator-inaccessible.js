// Copyright (C) 2019 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createasyncfromsynciterator
description: >
  Async-from-Sync Iterator instances are not accessible from user code.
info: |
  25.1.4.1 CreateAsyncFromSyncIterator ( syncIteratorRecord )
    1. Let asyncIterator be ! ObjectCreate(%AsyncFromSyncIteratorPrototype%, « [[SyncIteratorRecord]] »).
    2. Set asyncIterator.[[SyncIteratorRecord]] to syncIteratorRecord.
    3. Let nextMethod be ! Get(asyncIterator, "next").
    4. Let iteratorRecord be Record { [[Iterator]]: asyncIterator, [[NextMethod]]: nextMethod, [[Done]]: false }.
    5. Return iteratorRecord.

  14.4.14 Runtime Semantics: Evaluation
    YieldExpression : yield * AssignmentExpression
      1. Let generatorKind be ! GetGeneratorKind().
      ...
      4. Let iteratorRecord be ? GetIterator(value, generatorKind).
      ...

  7.4.1 GetIterator ( obj [ , hint [ , method ] ] )
    ...
    3. If method is not present, then
      a. If hint is async, then
        i. Set method to ? GetMethod(obj, @@asyncIterator).
        ii. If method is undefined, then
          1. Let syncMethod be ? GetMethod(obj, @@iterator).
          2. Let syncIteratorRecord be ? GetIterator(obj, sync, syncMethod).
          3. Return ? CreateAsyncFromSyncIterator(syncIteratorRecord).
      ...

flags: [async]
features: [async-iteration]
---*/

var AsyncIteratorPrototype = Object.getPrototypeOf(async function*(){}.constructor.prototype.prototype);

Object.defineProperty(AsyncIteratorPrototype, Symbol.iterator, {
  get() {
    throw new Error("@@iterator accessed");
  }
});

Object.defineProperty(AsyncIteratorPrototype, Symbol.asyncIterator, {
  get() {
    throw new Error("@@asyncIterator accessed");
  }
});

async function* g() {
    yield* [];
}
g().next().then(() => $DONE(), $DONE);
