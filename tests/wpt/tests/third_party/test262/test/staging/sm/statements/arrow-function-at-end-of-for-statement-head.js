/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Don't assert when an arrow function occurs at the end of a declaration init-component of a for(;;) loop head
info: bugzilla.mozilla.org/show_bug.cgi?id=1302994
esid: pending
---*/

function f1()
{
  for (var x = a => b; false; false)
  {}
}
f1();

function f2()
{
  for (var x = a => b, y = c => d; false; false)
  {}
}
f2();

function f3()
{
  for (var x = a => {}; false; false)
  {}
}
f3();

function f4()
{
  for (var x = a => {}, y = b => {}; false; false)
  {}
}
f4();

function g1()
{
  for (a => b; false; false)
  {}
}
g1();

function g2()
{
  for (a => {}; false; false)
  {}
}
g2();
