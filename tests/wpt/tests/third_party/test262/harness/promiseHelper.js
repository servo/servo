// Copyright (C) 2017 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    Check that an array contains a numeric sequence starting at 1
    and incrementing by 1 for each entry in the array. Used by
    Promise tests to assert the order of execution in deep Promise
    resolution pipelines.
defines: [checkSequence, checkSettledPromises]
---*/

function checkSequence(arr, message) {
  arr.forEach(function(e, i) {
    if (e !== (i+1)) {
      throw new Test262Error((message ? message : "Steps in unexpected sequence:") +
             " '" + arr.join(',') + "'");
    }
  });

  return true;
}

function checkSettledPromises(settleds, expected, message) {
  const prefix = message ? `${message}: ` : '';

  assert.sameValue(Array.isArray(settleds), true, `${prefix}Settled values is an array`);

  assert.sameValue(
    settleds.length,
    expected.length,
    `${prefix}The settled values has a different length than expected`
  );

  settleds.forEach((settled, i) => {
    assert.sameValue(
      Object.prototype.hasOwnProperty.call(settled, 'status'),
      true,
      `${prefix}The settled value has a property status`
    );

    assert.sameValue(settled.status, expected[i].status, `${prefix}status for item ${i}`);

    if (settled.status === 'fulfilled') {
      assert.sameValue(
        Object.prototype.hasOwnProperty.call(settled, 'value'),
        true,
        `${prefix}The fulfilled promise has a property named value`
      );

      assert.sameValue(
        Object.prototype.hasOwnProperty.call(settled, 'reason'),
        false,
        `${prefix}The fulfilled promise has no property named reason`
      );

      assert.sameValue(settled.value, expected[i].value, `${prefix}value for item ${i}`);
    } else {
      assert.sameValue(settled.status, 'rejected', `${prefix}Valid statuses are only fulfilled or rejected`);

      assert.sameValue(
        Object.prototype.hasOwnProperty.call(settled, 'value'),
        false,
        `${prefix}The fulfilled promise has no property named value`
      );

      assert.sameValue(
        Object.prototype.hasOwnProperty.call(settled, 'reason'),
        true,
        `${prefix}The fulfilled promise has a property named reason`
      );

      assert.sameValue(settled.reason, expected[i].reason, `${prefix}Reason value for item ${i}`);
    }
  });
}
