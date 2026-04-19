// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.groupby
description: Object.groupBy calls function with correct arguments
info: |
  Object.groupBy ( items, callbackfn )

  ...
  GroupBy ( items, callbackfn, coercion )

  6. Repeat,

      e. Let key be Completion(Call(callbackfn, undefined, « value, 𝔽(k) »)).
  ...
features: [array-grouping]
---*/


const arr = [-0, 0, 1, 2, 3];

let calls = 0;

Object.groupBy(arr, function (n, i) {
  calls++;
  assert.sameValue(n, arr[i], "selected element aligns with index");
  assert.sameValue(arguments.length, 2, "only two arguments are passed");
  return null;
});

assert.sameValue(calls, 5, 'called for all 5 elements');
