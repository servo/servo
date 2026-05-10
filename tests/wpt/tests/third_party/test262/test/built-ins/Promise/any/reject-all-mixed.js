// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Promise.any rejection reasons from various rejections are all present
flags: [async]
features: [Promise.any, arrow-function]
---*/

let rejections = [
  Promise.reject('a'),
  new Promise((_, reject) => reject('b')),
  Promise.all([Promise.reject('c')]),
  Promise.resolve(Promise.reject('d')),
];

Promise.any(rejections)
  .then(
    () => $DONE('The promise should be rejected, but was resolved'),
    error => {
      assert.sameValue(error.errors.length, rejections.length);
      assert.sameValue(error.errors.join(''), 'abcd');
    }
  ).then($DONE, $DONE);
