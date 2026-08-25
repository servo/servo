/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Array.prototype.pop shouldn't touch elements greater than length on non-arrays
info: bugzilla.mozilla.org/show_bug.cgi?id=909602
esid: pending
---*/

function doTest(obj, index)
{
  assert.sameValue(Array.prototype.pop.call(obj), undefined);
  assert.sameValue(index in obj, true);
  assert.sameValue(obj[index], 42);
}

// not-super-much-later element

// non-zero length
function testPop1()
{
  var obj = { length: 2, 3: 42 };
  doTest(obj, 3);
}
for (var i = 0; i < 50; i++)
  testPop1();

// zero length
function testPop2()
{
  var obj = { length: 0, 3: 42 };
  doTest(obj, 3);
}
for (var i = 0; i < 50; i++)
  testPop2();

// much-later (but dense) element

// non-zero length
function testPop3()
{
  var obj = { length: 2, 55: 42 };
  doTest(obj, 55);
}
for (var i = 0; i < 50; i++)
  testPop3();

// zero length
function testPop4()
{
  var obj = { length: 0, 55: 42 };
  doTest(obj, 55);
}
for (var i = 0; i < 50; i++)
  testPop4();

// much much much later (sparse) element

// non-zero length
function testPop5()
{
  var obj = { length: 2, 65530: 42 };
  doTest(obj, 65530);
}
for (var i = 0; i < 50; i++)
  testPop5();

// zero length
function testPop6()
{
  var obj = { length: 0, 65530: 42 };
  doTest(obj, 65530);
}
for (var i = 0; i < 50; i++)
  testPop6();
