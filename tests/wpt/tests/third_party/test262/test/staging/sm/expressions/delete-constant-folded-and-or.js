/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  Deletion of a && or || expression that constant-folds to a name must not attempt to delete the name
info: bugzilla.mozilla.org/show_bug.cgi?id=1183400
esid: pending
---*/

Object.defineProperty(this, "nonconfigurable", { value: 42 });
assert.sameValue(nonconfigurable, 42);

assert.sameValue(delete nonconfigurable, false);
assert.sameValue(delete (true && nonconfigurable), true);

function nested()
{
  assert.sameValue(delete nonconfigurable, false);
  assert.sameValue(delete (true && nonconfigurable), true);
}
nested();

function nestedStrict()
{
  "use strict";
  assert.sameValue(delete (true && nonconfigurable), true);
}
nestedStrict();
