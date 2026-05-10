// Copyright (c) 2021 Rick Waldron.  All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-array.from
description: >
    Creates an array with length that is equal to the value of the
    length property of the given array-like, regardless of
    the presence of corresponding indices and values.
info: |
    Array.from ( items [ , mapfn [ , thisArg ] ] )

    7. Let arrayLike be ! ToObject(items).
    8. Let len be ? LengthOfArrayLike(arrayLike).
    9. If IsConstructor(C) is true, then
      a. Let A be ? Construct(C, « 𝔽(len) »).
    10. Else,
      a. Let A be ? ArrayCreate(len).

includes: [compareArray.js]
---*/

const length = 5;

const newlyCreatedArray = Array.from({ length });
assert.sameValue(
  newlyCreatedArray.length,
  length,
  "The newly created array's length is equal to the value of the length property for the provided array like object"
);
assert.compareArray(newlyCreatedArray, [undefined, undefined, undefined, undefined, undefined]);

const newlyCreatedAndMappedArray = Array.from({ length }).map(x => 1);
assert.sameValue(
  newlyCreatedAndMappedArray.length,
  length,
  "The newly created and mapped array's length is equal to the value of the length property for the provided array like object"
);
assert.compareArray(newlyCreatedAndMappedArray, [1, 1, 1, 1, 1]);
