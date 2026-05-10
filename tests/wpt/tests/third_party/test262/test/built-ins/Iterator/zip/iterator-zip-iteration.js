// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zip
description: >
  Perform iteration in IteratorZip.
info: |
  Iterator.zip ( iterables [ , options ] )
    ...
    16. Return IteratorZip(iters, mode, padding, finishResults).

  IteratorZip ( iters, mode, padding, finishResults )
    3. Let closure be a new Abstract Closure with no parameters that captures
       iters, iterCount, openIters, mode, padding, and finishResults, and
       performs the following steps when called:
      ...
      b. Repeat,
        ...
        iii. For each integer i such that 0 ≤ i < iterCount, in ascending order, do
          ...
          3. Else,
            a. Let result be Completion(IteratorStepValue(iter)).
          ...
includes: [compareArray.js]
features: [joint-iteration]
---*/

var modes = [
  "shortest",
  "longest",
  "strict",
];

function makeIterator(log, name, elements) {
  var elementsIter = elements.values();
  var iterator = {
    next() {
      log.push(`call ${name} next`);

      // Called with the correct receiver and no arguments.
      assert.sameValue(this, iterator);
      assert.sameValue(arguments.length, 0);

      var result = elementsIter.next();
      return {
        get done() {
          log.push(`get ${name}.result.done`);
          return result.done;
        },
        get value() {
          log.push(`get ${name}.result.value`);
          return result.value;
        },
      };
    },
    return() {
      log.push(`call ${name} return`);

      // Called with the correct receiver and no arguments.
      assert.sameValue(this, iterator);
      assert.sameValue(arguments.length, 0);

      return {
        get done() {
          log.push(`unexpected get ${name}.result.done`);
          return result.done;
        },
        get value() {
          log.push(`unexpected get ${name}.result.value`);
          return result.value;
        },
      };
    }
  };
  return iterator;
}

for (var mode of modes) {
  var log = [];
  var iterables = [
    makeIterator(log, "first", [1, 2, 3]),
    makeIterator(log, "second", [4, 5, 6]),
    makeIterator(log, "third", [7, 8, 9]),
  ];
  var it = Iterator.zip(iterables, {mode});

  log.push("start");
  for (var v of it) {
    log.push("loop");
  }

  var expected = [
    "start",

    "call first next",
      "get first.result.done",
      "get first.result.value",
    "call second next",
      "get second.result.done",
      "get second.result.value",
    "call third next",
      "get third.result.done",
      "get third.result.value",
    "loop",

    "call first next",
      "get first.result.done",
      "get first.result.value",
    "call second next",
      "get second.result.done",
      "get second.result.value",
    "call third next",
      "get third.result.done",
      "get third.result.value",
    "loop",

    "call first next",
      "get first.result.done",
      "get first.result.value",
    "call second next",
      "get second.result.done",
      "get second.result.value",
    "call third next",
      "get third.result.done",
      "get third.result.value",
    "loop",
  ];

  switch (mode) {
    case "shortest": {
      expected.push(
        "call first next",
          "get first.result.done",
        "call third return",
        "call second return",
      );
      break;
    }
    case "longest":
    case "strict": {
      expected.push(
        "call first next",
          "get first.result.done",
        "call second next",
          "get second.result.done",
        "call third next",
          "get third.result.done",
      );
      break;
    }
  }

  assert.compareArray(log, expected);
}
