/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Array.reduce should ignore holes
info: bugzilla.mozilla.org/show_bug.cgi?id=386030
esid: pending
---*/

var actual = '';
var expect = '';

test();

function test()
{
  function add(a, b) { return a + b; }
  function testreduce(v) { return v == 3 ? "PASS" : "FAIL"; }

  var a;

  expect = 'PASS';

  try {
    a = new Array(2);
    a[1] = 3;
    actual = testreduce(a.reduce(add));
  } catch (e) {
    actual = "FAIL, reduce";
  }
  assert.sameValue(expect, actual, '1');

  try {
    a = new Array(2);
    a[0] = 3;
    actual = testreduce(a.reduceRight(add));
  } catch (e) {
    actual = "FAIL, reduceRight";
  }
  assert.sameValue(expect, actual, '2');

  try {
    a = new Array(2);
    a.reduce(add);
    actual = "FAIL, empty reduce";
  } catch (e) {
    actual = "PASS";
  }
  assert.sameValue(expect, actual, '3');

  try {
    a = new Array(2);
    a.reduceRight(add);
    actual = "FAIL, empty reduceRight";
  } catch (e) {
    actual = "PASS";
  }
  assert.sameValue(expect, actual, '4');
}
