/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Object.preventExtensions([]).length = 0 should do nothing, not throw
info: bugzilla.mozilla.org/show_bug.cgi?id=600392
esid: pending
---*/

function testEmpty()
{
  var a = [];
  assert.sameValue(a.length, 0);
  assert.sameValue(Object.preventExtensions(a), a);
  assert.sameValue(a.length, 0);
  a.length = 0;
  assert.sameValue(a.length, 0);
}
testEmpty();

function testEmptyStrict()
{
  "use strict";
  var a = [];
  assert.sameValue(a.length, 0);
  assert.sameValue(Object.preventExtensions(a), a);
  assert.sameValue(a.length, 0);
  a.length = 0;
  assert.sameValue(a.length, 0);
}
testEmptyStrict();

function testNonEmpty()
{
  var a = [1, 2, 3];
  assert.sameValue(a.length, 3);
  assert.sameValue(Object.preventExtensions(a), a);
  assert.sameValue(a.length, 3);
  a.length = 0;
  assert.sameValue(a.length, 0);
}
testNonEmpty();

function testNonEmptyStrict()
{
  "use strict";
  var a = [1, 2, 3];
  assert.sameValue(a.length, 3);
  assert.sameValue(Object.preventExtensions(a), a);
  assert.sameValue(a.length, 3);
  a.length = 0;
  assert.sameValue(a.length, 0);
}
testNonEmptyStrict();
