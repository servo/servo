/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

assert.throws(
    TypeError,
    () => {
      {let i=1}
      {let j=1; [][j][2]}
    }
);

