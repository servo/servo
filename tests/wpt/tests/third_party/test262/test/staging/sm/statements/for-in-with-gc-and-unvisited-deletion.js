/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Don't mishandle deletion of a property from the internal iterator created for a for-in loop, when a gc occurs just after it
info: bugzilla.mozilla.org/show_bug.cgi?id=1462939
esid: pending
features: [host-gc-required]
---*/

function testOneDeletion()
{
  var o = {
    p: 1,
    r: 3,
    s: 4,
  };

  for (var i in o)
  {
    $262.gc();
    delete o.s;
  }
}
testOneDeletion();

function testTwoDeletions()
{
  var o = {
    p: 1,
    r: 3,
    s: 4,
    t: 5,
  };

  for (var i in o)
  {
    $262.gc();
    delete o.t;
    delete o.s;
  }
}
testTwoDeletions();

function testThreeDeletions()
{
  var o = {
    p: 1,
    r: 3,
    s: 4,
    t: 5,
    x: 7,
  };

  for (var i in o)
  {
    $262.gc();
    delete o.x;
    delete o.t;
    delete o.s;
  }
}
testThreeDeletions();
