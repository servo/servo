/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Test for correct implementation of |Date == boolean| and vice versa
esid: pending
---*/

Date.prototype.toString = function() { return 1; };
Date.prototype.valueOf = function() { return 0; };

/*
 * ES5 11.9.3 doesn't directly handle obj == boolean.  Instead it translates it
 * as follows:
 *
 *   obj == boolean
 *   ↳ obj == ToNumber(boolean), per step 7
 *     ↳ ToPrimitive(obj) == ToNumber(boolean), per step 9
 *
 * ToPrimitive calls [[DefaultValue]] with no hint.  For Date objects this is
 * treated as if it were instead called with hint String.  That calls toString,
 * which returns 1, so Date objects here should compare equal to true and
 * unequal to false.
 */
assert.sameValue(new Date == true, true);
assert.sameValue(new Date == false, false);

/* == is symmetric. */
assert.sameValue(true == new Date, true);
assert.sameValue(false == new Date, false);
