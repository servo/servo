/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Be more careful with string math to avoid wrong results
info: bugzilla.mozilla.org/show_bug.cgi?id=805121
esid: pending
---*/

function puff(x, n)
{
  while(x.length < n)
    x += x;
  return x.substring(0, n);
}

var x = puff("1", 1 << 20);
var rep = puff("$1", 1 << 16);

try
{
  var y = x.replace(/(.+)/g, rep);
  assert.sameValue(y.length, Math.pow(2, 36));
}
catch (e)
{
  // OOM also acceptable
}
