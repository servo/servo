// Copyright (C) 2026 Saúl Ibarra Corretgé. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  The iterator helper is completed after an abrupt completion
features: [iterator-sequencing]
---*/

// The closure of Iterator.concat is the body of a generator, so an abrupt
// completion from it completes the generator: a later next() returns an
// undefined, done result without stepping the iterator again, the iterables that
// follow are never opened, and a later return() is not forwarded either.
function concatWithStep(next) {
  const calls = {
    nextCalls: 0,
    returnCalls: 0,
    openCalls: 0,
  };

  const iterable = {
    [Symbol.iterator]() {
      return {
        next() {
          ++calls.nextCalls;
          return next();
        },
        return() {
          ++calls.returnCalls;
          return {};
        },
      };
    }
  };

  const laterIterable = {
    [Symbol.iterator]() {
      ++calls.openCalls;
      return {
        next() {
          return {
            done: false,
            value: 1,
          };
        },
      };
    }
  };

  return {
    calls,
    iterator: Iterator.concat(iterable, laterIterable),
  };
}

function assertCompleted(concat) {
  const resultAfterThrow = concat.iterator.next();

  assert.sameValue(resultAfterThrow.done, true, 'the iterator helper is completed');
  assert.sameValue(resultAfterThrow.value, undefined, 'the iterator helper is completed');
  assert.sameValue(concat.calls.nextCalls, 1, 'the underlying iterator is not stepped again');
  assert.sameValue(concat.calls.openCalls, 0, 'the remaining iterables are never opened');

  // Only the calls made from here on are counted, so that this assertion does
  // not overlap with the one made in
  // underlying-iterator-not-closed-when-step-throws.js.
  const returnCallsBefore = concat.calls.returnCalls;
  const resultAfterReturn = concat.iterator.return();

  assert.sameValue(resultAfterReturn.done, true, 'the iterator helper is completed');
  assert.sameValue(resultAfterReturn.value, undefined, 'the iterator helper is completed');
  assert.sameValue(concat.calls.returnCalls, returnCallsBefore,
                   'return is not forwarded to the underlying iterator');
}

// Underlying iterator has a throwing next method.
const concatWithThrowingNext = concatWithStep(function() {
  throw new Test262Error();
});

assert.throws(Test262Error, function() {
  concatWithThrowingNext.iterator.next();
});

assertCompleted(concatWithThrowingNext);

// Underlying iterator next returns an object with a throwing done getter.
const concatWithThrowingDoneGetter = concatWithStep(function() {
  return {
    get done() {
      throw new Test262Error();
    },
    value: 1,
  };
});

assert.throws(Test262Error, function() {
  concatWithThrowingDoneGetter.iterator.next();
});

assertCompleted(concatWithThrowingDoneGetter);

// Underlying iterator next returns an object with a throwing value getter.
const concatWithThrowingValueGetter = concatWithStep(function() {
  return {
    done: false,
    get value() {
      throw new Test262Error();
    },
  };
});

assert.throws(Test262Error, function() {
  concatWithThrowingValueGetter.iterator.next();
});

assertCompleted(concatWithThrowingValueGetter);

// Underlying iterator next returns a non-object.
const concatWithNonObjectResult = concatWithStep(function() {
  return null;
});

assert.throws(TypeError, function() {
  concatWithNonObjectResult.iterator.next();
});

assertCompleted(concatWithNonObjectResult);
