/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

function nonstrict() { return nonstrict.caller; }
function strict() { "use strict"; return nonstrict(); }

assert.sameValue(strict(), null);
