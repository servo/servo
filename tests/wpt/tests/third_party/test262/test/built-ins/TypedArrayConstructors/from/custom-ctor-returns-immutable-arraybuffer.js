// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.from
description: >
  Throws a TypeError if species constructor returns an immutable ArrayBuffer.
info: |
  %TypedArray%.from ( source [ , mapper [ , thisArg ] ] )
  1. Let C be the this value.
  2. If IsConstructor(C) is false, throw a TypeError exception.
  3. If mapper is undefined, then
     a. Let mapping be false.
  4. Else,
     a. If IsCallable(mapper) is false, throw a TypeError exception.
     b. Let mapping be true.
  5. Let usingIterator be ? GetMethod(source, %Symbol.iterator%).
  6. If usingIterator is not undefined, then
     a. Let values be ? IteratorToList(? GetIteratorFromMethod(source, usingIterator)).
     b. Let len be the number of elements in values.
     c. Let targetObj be ? TypedArrayCreateFromConstructor(C, « 𝔽(len) », write).
     d. Let k be 0.
     e. Repeat, while k < len,
        i. Let Pk be ! ToString(𝔽(k)).
        ii. Let kValue be the first element of values.
        iii. Remove the first element from values.
        iv. If mapping is true, then
            1. Let mappedValue be ? Call(mapper, thisArg, « kValue, 𝔽(k) »).
        v. Else,
           1. Let mappedValue be kValue.
        vi. Perform ? Set(targetObj, Pk, mappedValue, true).
        vii. Set k to k + 1.
     f. Assert: values is now an empty List.
     g. Return targetObj.
  7. NOTE: source is not an iterable object, so assume it is already an array-like object.
  8. Let arrayLike be ! ToObject(source).
  9. Let len be ? LengthOfArrayLike(arrayLike).
  10. Let targetObj be ? TypedArrayCreateFromConstructor(C, « 𝔽(len) », write).
  11. Let k be 0.
  12. Repeat, while k < len,
      a. Let Pk be ! ToString(𝔽(k)).
      b. Let kValue be ? Get(arrayLike, Pk).
      c. If mapping is true, then
         i. Let mappedValue be ? Call(mapper, thisArg, « kValue, 𝔽(k) »).
features: [Symbol.iterator, TypedArray, immutable-arraybuffer]
includes: [testTypedArray.js, compareArray.js]
---*/

testWithAllTypedArrayConstructors((TA, makeCtorArg) => {
  var calls = [];
  var expectCalls = [];

  var custom = new TA(makeCtorArg(2));
  var ctor = function(len) {
    calls.push("construct(" + len + ")");
    return custom;
  };
  var mapper = function(value, index) {
    calls.push("map index " + index);
    return value + value;
  };

  // arraylike source
  calls = [];
  var srcArraylike = {
    get length() {
      calls.push("get source.length");
      return 1;
    },
    get 0() {
      calls.push("get source[0]");
      return "8";
    }
  };
  Object.defineProperty(srcArraylike, Symbol.iterator, {
    get: function() {
      calls.push("get source[Symbol.iterator]");
      return undefined;
    }
  });
  assert.throws(TypeError, function() {
    TA.from.call(ctor, srcArraylike, mapper);
  }, "arraylike source");
  expectCalls = [
    "get source[Symbol.iterator]",
    "get source.length",
    "construct(1)"
  ];
  assert.compareArray(calls, expectCalls,
    "Must construct the result before visiting arraylike source elements.");

  // iterable source
  calls = [];
  var srcIterable = Object.defineProperty({}, Symbol.iterator, {
    get: function() {
      calls.push("get source[Symbol.iterator]");
      function getIterator() {
        calls.push("call source[Symbol.iterator]");
        var itor = {
          get next() {
            calls.push("get source iterator.next");
            var iterationResults = [
              { done: false, value: "4", msg: "yield 4" },
              { done: true, value: "8", msg: "done" },
              { done: true, value: "9", msg: "unexpected" }
            ];
            function next() {
              var result = iterationResults.shift();
              calls.push("source iterator " + result.msg);
              var resultSpy = {
                get done() {
                  calls.push("get iterationResult.done " + result.done);
                  return result.done;
                },
                get value() {
                  calls.push("get iterationResult.value " + result.value);
                  return result.value;
                }
              };
              return resultSpy;
            };
            return next;
          }
        };
        return itor;
      }
      return getIterator;
    }
  });
  assert.throws(TypeError, function() {
    TA.from.call(ctor, srcIterable, mapper);
  }, "iterable source");
  expectCalls = [
    "get source[Symbol.iterator]",
    "call source[Symbol.iterator]",
    "get source iterator.next",
    "source iterator yield 4",
    "get iterationResult.done false",
    "get iterationResult.value 4",
    "source iterator done",
    "get iterationResult.done true",
    "construct(1)"
  ];
  assert.compareArray(calls, expectCalls,
    "Must construct the result before visiting iterable source elements.");
}, null, ["immutable"]);
