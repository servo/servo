// Copyright 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asyncgenerator-prototype-throw
description: throw rejects promise when `this` value is not an async generator
info: |
  AsyncGenerator.prototype.throw ( exception )
  1. Let generator be the this value.
  2. Let completion be Completion{[[Type]]: throw, [[Value]]: exception,
     [[Target]]: empty}.
  3. Return ! AsyncGeneratorEnqueue(generator, completion).

  AsyncGeneratorEnqueue ( generator, completion )
  ...
  3. If Type(generator) is not Object, or if generator does not have an
     [[AsyncGeneratorState]] internal slot, then
    a. Let badGeneratorError be a newly created TypeError object.
    b. Perform ! Call(promiseCapability.[[Reject]], undefined, « badGeneratorError »).
    c. Return promiseCapability.[[Promise]].

flags: [async]
features: [async-iteration]
---*/

async function* g() {}
var AsyncGeneratorPrototype = Object.getPrototypeOf(g).prototype;

function* syncGenerator() {}
var syncIterator = syncGenerator()

var testPromises = [
  AsyncGeneratorPrototype.throw.call({}).then(
    function () {
      throw new Test262Error("AsyncGeneratorPrototype.throw should reject promise" +
                             " when `this` value is an object");
    },
    function (e) {
      if (!(e instanceof TypeError)) {
       throw new Test262Error("(object) expected TypeError but got " + e);
      }
    }
  ),
  AsyncGeneratorPrototype.throw.call(function() {}).then(
    function () {
      throw new Test262Error("AsyncGeneratorPrototype.throw should reject promise" +
                             " when `this` value is a function");
    },
    function (e) {
      if (!(e instanceof TypeError)) {
       throw new Test262Error("(function) expected TypeError but got " + e);
      }
    }
  ),
  AsyncGeneratorPrototype.throw.call(g).then(
    function () {
      throw new Test262Error("AsyncGeneratorPrototype.throw should reject promise" +
                             " when `this` value is an async generator function");
    },
    function (e) {
      if (!(e instanceof TypeError)) {
       throw new Test262Error("(async generator function) expected TypeError but got " + e);
      }
    }
  ),
  AsyncGeneratorPrototype.throw.call(g.prototype).then(
    function () {
      throw new Test262Error("AsyncGeneratorPrototype.throw should reject promise" +
                             " when `this` value is an async generator function prototype object");
    },
    function (e) {
      if (!(e instanceof TypeError)) {
       throw new Test262Error("(async generator function prototype) expected TypeError but got " + e);
      }
    },
  ),
  AsyncGeneratorPrototype.throw.call(syncIterator).then(
    function () {
      throw new Test262Error("AsyncGeneratorPrototype.throw should reject promise" +
                             " when `this` value is a generator");
    },
    function (e) {
      if (!(e instanceof TypeError)) {
       throw new Test262Error("(generator) expected TypeError but got " + e);
      }
    }
  )
]

Promise.all(testPromises).then(() => {}).then($DONE, $DONE)
