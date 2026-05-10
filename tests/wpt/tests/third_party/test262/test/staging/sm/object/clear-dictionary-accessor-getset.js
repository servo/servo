/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Properly handle GC of a dictionary accessor property whose [[Get]] or [[Set]] has been changed to |undefined|
info: bugzilla.mozilla.org/show_bug.cgi?id=1082662
esid: pending
features: [host-gc-required]
---*/

function test(field)
{
  function inner()
  {
     // Create an object with an accessor property.
     var obj = { x: 42, get y() {}, set y(v) {} };

     // 1) convert it to dictionary mode, in the process 2) creating a new
     // version of that accessor property whose [[Get]] and [[Set]] are objects
     // that trigger post barriers.
     delete obj.x;

     var desc = {};
     desc[field] = undefined;

     // Overwrite [[field]] with undefined.  Note #1 above is necessary so this
     // is an actual *overwrite*, and not (possibly) a shape-tree fork that
     // doesn't overwrite.
     Object.defineProperty(obj, "y", desc);

  }

  inner();
  $262.gc(); // In unfixed code, this crashes trying to mark a null [[field]].
}

test("get");
test("set");
