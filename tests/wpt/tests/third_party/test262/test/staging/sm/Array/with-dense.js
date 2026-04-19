// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

const indices = [
  -10, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 10,
];

const arrays = [
  // Dense no holes.
  [],
  [1],
  [1,2],
  [1,2,3],
  [1,2,3,4],
  [1,2,3,4,5,6,7,8],

  // Dense trailing holes.
  [,],
  [1,,],
  [1,2,,],
  [1,2,3,,],
  [1,2,3,4,,],
  [1,2,3,4,5,6,7,8,,],

  // Dense leading holes.
  [,],
  [,1],
  [,1,2],
  [,1,2,3],
  [,1,2,3,4],
  [,1,2,3,4,5,6,7,8],

  // Dense with holes.
  [1,,3],
  [1,2,,4],
  [1,,3,,5,6,,8],
];

const objects = arrays.map(array => {
  let obj = {
    length: array.length,
  };
  for (let i = 0; i < array.length; ++i) {
    if (i in array) {
      obj[i] = array[i];
    }
  }
  return obj;
});

const objectsWithLargerDenseInitializedLength = arrays.map(array => {
  let obj = {
    length: array.length,
  };
  for (let i = 0; i < array.length; ++i) {
    if (i in array) {
      obj[i] = array[i];
    }
  }

  // Add some extra dense elements after |length|.
  for (let i = 0; i < 5; ++i) {
    obj[array.length + i] = "extra";
  }

  return obj;
});

const thisValues = [
  ...arrays,
  ...objects,
  ...objectsWithLargerDenseInitializedLength,
];

const replacement = {};

for (let thisValue of thisValues) {
  for (let index of indices) {
    let actualIndex = index;
    if (actualIndex < 0) {
      actualIndex += thisValue.length;
    }

    if (actualIndex < 0 || actualIndex >= thisValue.length) {
      continue;
    }

    let res = Array.prototype.with.call(thisValue, index, replacement);
    assert.sameValue(res.length, thisValue.length);

    for (let i = 0; i < thisValue.length; ++i) {
      assert.sameValue(Object.hasOwn(res, i), true);

      if (i === actualIndex) {
        assert.sameValue(res[i], replacement);
      } else {
        assert.sameValue(res[i], thisValue[i]);
      }
    }
  }
}

