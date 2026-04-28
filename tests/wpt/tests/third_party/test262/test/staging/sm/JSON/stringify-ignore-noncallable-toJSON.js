/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  If the toJSON property isn't callable, don't try to call it
info: bugzilla.mozilla.org/show_bug.cgi?id=584909
esid: pending
---*/

var obj =
  {
    p: { toJSON: null },
    m: { toJSON: {} }
  };

assert.sameValue(JSON.stringify(obj), '{"p":{"toJSON":null},"m":{"toJSON":{}}}');
