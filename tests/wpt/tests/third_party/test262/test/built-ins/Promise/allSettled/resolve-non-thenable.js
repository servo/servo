// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a non-thenable object value
esid: sec-promise.allsettled
info: |
  Promise.allSettled Resolve Element Functions

  14. If remainingElementsCount.[[Value]] is 0, then
    a. Let valuesArray be ! CreateArrayFromList(values).
    b. Return ? Call(promiseCapability.[[Resolve]], undefined, « valuesArray »).
flags: [async]
includes: [promiseHelper.js]
features: [Promise.allSettled]
---*/

var v1 = {};
var v2 = {};
var v3 = {};

Promise.allSettled([v1, v2, v3])
  .then(function(values) {
    checkSettledPromises(values, [
      {
        status: 'fulfilled',
        value: v1
      },
      {
        status: 'fulfilled',
        value: v2
      },
      {
        status: 'fulfilled',
        value: v3
      }
    ], 'values');
  }, function() {
    $DONE('The promise should not be rejected.');
  }).then($DONE, $DONE);
