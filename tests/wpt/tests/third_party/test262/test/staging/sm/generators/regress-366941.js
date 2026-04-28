/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

//-----------------------------------------------------------------------------
var BUGNUMBER = 366941;
var summary = 'Destructuring enumerations, iterations';
var actual = '';
var expect = '';


//-----------------------------------------------------------------------------
test();
//-----------------------------------------------------------------------------

function test()
{
  var list1 = [[1,2],[3,4],[5,6]];
  var list2 = [[1,2,3],[4,5,6],[7,8,9]];

  expect = '1,2;3,4;5,6;';
  actual = '';

  for (var [foo, bar] of list1) {
    actual += foo + "," + bar + ";";
  }

  assert.sameValue(expect, actual, summary + ': 1');

  expect = '1,2,3;4,5,6;7,8,9;';
  actual = '';
  for (var [foo, bar, baz] of list2) {
    actual += foo + "," + bar + "," + baz + ";";
  }

  assert.sameValue(expect, actual, summary + ': 2');

  function* gen(list) {
    for (var test of list) {
      yield test;
    }
  }

  var iter1 = gen(list1);

  expect = '1,2;3,4;5,6;';
  actual = '';

  for (var [foo, bar] of iter1) {
    actual += foo + "," + bar + ";";
  }

  assert.sameValue(expect, actual, summary + ': 3');

  // Before JS1.7's destructuring for…in was fixed to match JS1.8's,
  // the expected result was a SyntaxError about the for…in loop's lhs.
  var iter2 = gen(list2);
  expect = '1,2,3;4,5,6;7,8,9;';
  actual = '';

  try
  {
    eval('for (var [foo, bar, baz] of iter2) {' +
         'actual += foo + "," + bar + "," + baz + ";";' +
         '}');
  }
  catch(ex)
  {
    actual = ex + '';
  }

  assert.sameValue(expect, actual, summary + ': 4');
}
