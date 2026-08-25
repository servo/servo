/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
/* Test JSOP_UNBRANDTHIS's behavior on object and non-object |this| values. */

function strict() {
  "use strict";
  this.insert = function(){ bar(); };
  function bar() {}
}

// Try 'undefined' as a |this| value.
assert.throws(TypeError, function() {
  strict.call(undefined);
});

// Try 'null' as a |this| value.
assert.throws(TypeError, function() {
  strict.call(null);
});

// An object as a |this| value should be fine.
strict.call({});
