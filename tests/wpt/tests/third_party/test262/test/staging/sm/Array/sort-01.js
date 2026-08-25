/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  array.sort compare-function gets incorrect this
info: bugzilla.mozilla.org/show_bug.cgi?id=604971
esid: pending
---*/

[1, 2, 3].sort(function() { "use strict"; assert.sameValue(this, undefined); });
