// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-aggregate-error
description: >
  Return abrupt completion from IterableToList(errors)
info: |
  AggregateError ( errors, message )

  ...
  3. Let errorsList be ? IterableToList(errors).
  4. Set O.[[AggregateErrors]] to errorsList.
  ...
  6. Return O.

  Runtime Semantics: IterableToList ( items [ , method ] )

  1. If method is present, then
    ...
  2. Else,
    b. Let iteratorRecord be ? GetIterator(items, sync).
  3. Let values be a new empty List.
  4. Let next be true.
  5. Repeat, while next is not false
    a. Set next to ? IteratorStep(iteratorRecord).
    b. If next is not false, then
      i. Let nextValue be ? IteratorValue(next).
      ii. Append nextValue to the end of the List values.
  6. Return values.

  GetIterator ( obj [ , hint [ , method ] ] )

  ...
  3. If method is not present, then
    a. If hint is async, then
      ...
    b. Otherwise, set method to ? GetMethod(obj, @@iterator).
  4. Let iterator be ? Call(method, obj).
  5. If Type(iterator) is not Object, throw a TypeError exception.
  6. Let nextMethod be ? GetV(iterator, "next").
  ...
  8. Return iteratorRecord.
features: [AggregateError, Symbol.iterator]
---*/

var case1 = {
  get [Symbol.iterator]() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => {
  var obj = new AggregateError(case1);
}, 'get Symbol.iterator');

var case2 = {
  get [Symbol.iterator]() {
    return {};
  }
};

assert.throws(TypeError, () => {
  var obj = new AggregateError(case2);
}, 'GetMethod(obj, @@iterator) abrupts from non callable');

var case3 = {
  [Symbol.iterator]() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => {
  var obj = new AggregateError(case3);
}, 'Abrupt from @@iterator call');

var case4 = {
  [Symbol.iterator]() {
    return 'a string';
  }
};

assert.throws(TypeError, () => {
  var obj = new AggregateError(case4);
}, '@@iterator call returns a string');

var case5 = {
  [Symbol.iterator]() {
    return undefined;
  }
};

assert.throws(TypeError, () => {
  var obj = new AggregateError(case5);
}, '@@iterator call returns undefined');

var case6 = {
  [Symbol.iterator]() {
    return {
      get next() {
        throw new Test262Error();
      }
    }
  }
};

assert.throws(Test262Error, () => {
  var obj = new AggregateError(case6);
}, 'GetV(iterator, next) returns abrupt');

var case7 = {
  [Symbol.iterator]() {
    return {
      get next() {
        return {};
      }
    }
  }
};

assert.throws(TypeError, () => {
  var obj = new AggregateError(case7);
}, 'GetV(iterator, next) returns a non callable');

var case8 = {
  [Symbol.iterator]() {
    return {
      next() {
        throw new Test262Error();
      }
    }
  }
};

assert.throws(Test262Error, () => {
  var obj = new AggregateError(case8);
}, 'abrupt from iterator.next()');

var case9 = {
  [Symbol.iterator]() {
    return {
      next() {
        return undefined;
      }
    }
  }
};

assert.throws(TypeError, () => {
  var obj = new AggregateError(case9);
}, 'iterator.next() returns undefined');

var case10 = {
  [Symbol.iterator]() {
    return {
      next() {
        return 'a string';
      }
    }
  }
};

assert.throws(TypeError, () => {
  var obj = new AggregateError(case10);
}, 'iterator.next() returns a string');

var case11 = {
  [Symbol.iterator]() {
    return {
      next() {
        return {
          get done() {
            throw new Test262Error();
          }
        };
      }
    }
  }
};

assert.throws(Test262Error, () => {
  var obj = new AggregateError(case11);
}, 'IteratorCompete abrupts getting the done property');
