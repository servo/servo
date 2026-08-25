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
var BUGNUMBER = 521456;
var summary =
  'Incorrect decompilation of new (eval(v)).s and new (f.apply(2)).s';

function foo(c) { return new (eval(c)).s; }
function bar(f) { var a = new (f.apply(2).s); return a; }

assert.sameValue(bar.toString().search(/new\s+f/), -1);
assert.sameValue(foo.toString().search(/new\s+eval/), -1);

