// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

const startIndices = [
  -10, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 10,
];

const deleteCounts = [
  0, 1, 2, 3, 4, 5, 10,
];

const insertCounts = [
  0, 1, 2, 3, 4, 5, 10,
];

const itemsList = insertCounts.map(count => {
  return new Array(count).fill(0);
});

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

for (let thisValue of thisValues) {
  for (let startIndex of startIndices) {
    for (let deleteCount of deleteCounts) {
      for (let items of itemsList) {
        let res = Array.prototype.toSpliced.call(thisValue, startIndex, deleteCount, ...items);

        // Array.prototype.toSpliced(), steps 3-6.
        let actualStart;
        if (startIndex < 0) {
          actualStart = Math.max(thisValue.length + startIndex, 0);
        } else {
          actualStart = Math.min(startIndex, thisValue.length);
        }

        // Array.prototype.toSpliced(), step 10.
        let actualDeleteCount = Math.min(Math.max(0, deleteCount), thisValue.length - actualStart);

        let newLength = thisValue.length + items.length - actualDeleteCount;
        assert.sameValue(res.length, newLength);

        for (let i = 0; i < actualStart; ++i) {
          assert.sameValue(Object.hasOwn(res, i), true);
          assert.sameValue(res[i], thisValue[i]);
        }

        for (let i = 0; i < items.length; ++i) {
          assert.sameValue(Object.hasOwn(res, actualStart + i), true);
          assert.sameValue(res[actualStart + i], items[i]);
        }

        for (let i = 0; i < newLength - actualStart - items.length; ++i) {
          assert.sameValue(Object.hasOwn(res, actualStart + items.length + i), true);
          assert.sameValue(res[actualStart + items.length + i],
                   thisValue[actualStart + actualDeleteCount + i]);
        }
      }
    }
  }
}

