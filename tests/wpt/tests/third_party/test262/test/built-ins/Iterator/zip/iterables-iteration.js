// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zip
description: >
  Perform iteration of the "iterables" argument.
info: |
  Iterator.zip ( iterables [ , options ] )
    ...
    10. Let inputIter be ? GetIterator(iterables, sync).
    11. Let next be not-started.
    12. Repeat, while next is not done,
      a. Set next to Completion(IteratorStepValue(inputIter)).
      b. IfAbruptCloseIterators(next, iters).
      c. If next is not done, then
        i. Let iter be Completion(GetIteratorFlattenable(next, reject-strings)).
        ii. IfAbruptCloseIterators(iter, the list-concatenation of « inputIter » and iters).
        iii. Append iter to iters.
    ...

  GetIterator ( obj, kind )
    ...
    2. Else,
      a. Let method be ? GetMethod(obj, %Symbol.iterator%).
    3. If method is undefined, throw a TypeError exception.
    4. Return ? GetIteratorFromMethod(obj, method).

  GetIteratorFromMethod ( obj, method )
    1. Let iterator be ? Call(method, obj).
    2. If iterator is not an Object, throw a TypeError exception.
    3. Return ? GetIteratorDirect(iterator).

  GetIteratorDirect ( obj )
    1. Let nextMethod be ? Get(obj, "next").
    2. Let iteratorRecord be the Iterator Record { [[Iterator]]: obj, [[NextMethod]]: nextMethod, [[Done]]: false }.
    3. Return iteratorRecord.

  GetIteratorFlattenable ( obj, primitiveHandling )
    1. If obj is not an Object, then
      a. If primitiveHandling is reject-primitives, throw a TypeError exception.
      b. Assert: primitiveHandling is iterate-string-primitives.
      c. If obj is not a String, throw a TypeError exception.
    2. Let method be ? GetMethod(obj, %Symbol.iterator%).
    3. If method is undefined, then
      a. Let iterator be obj.
    4. Else,
      a. Let iterator be ? Call(method, obj).
    5. If iterator is not an Object, throw a TypeError exception.
    6. Return ? GetIteratorDirect(iterator).
includes: [proxyTrapsHelper.js, compareArray.js]
features: [joint-iteration]
---*/

// Object implementing Iterator protocol, but throws when calling any Iterator methods.
var throwingIterator = {
  next() {
    throw new Test262Error();
  },
  return() {
    throw new Test262Error();
  }
};

var iterableReturningThrowingIterator = {
  [Symbol.iterator]() {
    return throwingIterator;
  }
};

// "iterables" argument must be an iterable.
assert.throws(TypeError, function() {
  Iterator.zip(Object.create(null));
});

// GetIteratorFlattenable accepts both iterables and iterators.
Iterator.zip([
  throwingIterator,
  iterableReturningThrowingIterator,
]);

// GetIteratorFlattenable rejects non-objects.
var badIterators = [
  undefined,
  null,
  true,
  "",
  Symbol(),
  0,
  0n,
];

for (var iterator of badIterators) {
  assert.throws(TypeError, function() {
    Iterator.zip([iterator]);
  });
}

// GetIterator and GetIteratorFlattenable read properties in the correct order.
var log = [];

function makeProxyWithGetHandler(name, obj) {
  return new Proxy(obj, allowProxyTraps({
    get(target, propertyKey, receiver) {
      log.push(`${name}::${String(propertyKey)}`);
      return Reflect.get(target, propertyKey, receiver);
    }
  }));
}

var elements = [
  // An iterator.
  makeProxyWithGetHandler("first", throwingIterator),

  // An iterable.
  makeProxyWithGetHandler("second", iterableReturningThrowingIterator),

  // An object without any iteration methods.
  makeProxyWithGetHandler("third", Object.create(null)),
];

var elementsIter = elements.values();

var iterables = makeProxyWithGetHandler("iterables", {
  [Symbol.iterator]() {
    // Called with the correct receiver and no arguments.
    assert.sameValue(this, iterables);
    assert.sameValue(arguments.length, 0);

    return this;
  },
  next() {
    log.push("call next");

    // Called with the correct receiver and no arguments.
    assert.sameValue(this, iterables);
    assert.sameValue(arguments.length, 0);

    return elementsIter.next();
  },
  return() {
    throw new Test262Error("unexpected call to return method");
  }
});

Iterator.zip(iterables);

assert.compareArray(log, [
  "iterables::Symbol(Symbol.iterator)",
  "iterables::next",

  "call next",
  "first::Symbol(Symbol.iterator)",
  "first::next",

  "call next",
  "second::Symbol(Symbol.iterator)",

  "call next",
  "third::Symbol(Symbol.iterator)",
  "third::next",

  "call next",
]);
