/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

function innermost() { return arguments.callee.caller; }
function nest() { return eval("innermost();"); }
function nest2() { return nest(); }

assert.sameValue(nest2(), nest);

var innermost = function innermost() { return arguments.callee.caller.caller; };

assert.sameValue(nest2(), nest2);

function nestTwice() { return eval("eval('innermost();');"); }
var nest = nestTwice;

assert.sameValue(nest2(), nest2);

function innermostEval() { return eval("arguments.callee.caller"); }
var innermost = innermostEval;

assert.sameValue(nest2(), nestTwice);

function innermostEvalTwice() { return eval('eval("arguments.callee.caller");'); }
var innermost = innermostEvalTwice;

assert.sameValue(nest2(), nestTwice);
