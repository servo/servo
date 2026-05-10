/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  { get x(v) { } } and { set x(v, v2) { } } should be syntax errors
info: bugzilla.mozilla.org/show_bug.cgi?id=536472
esid: pending
---*/

function expectOk(s)
{
  try
  {
    eval(s);
    return;
  }
  catch (e)
  {
    assert.sameValue(true, false,
             "expected no error parsing '" + "', got : " + e);
  }
}

function expectSyntaxError(s)
{
  assert.throws(SyntaxError, function() {
    eval(s);
  }, "expected syntax error parsing '" + s + "'");
}

expectSyntaxError("({ get x([]) { } })");
expectSyntaxError("({ get x({}) { } })");
expectSyntaxError("({ get x(a, []) { } })");
expectSyntaxError("({ get x(a, {}) { } })");
expectSyntaxError("({ get x([], a) { } })");
expectSyntaxError("({ get x({}, a) { } })");
expectSyntaxError("({ get x([], a, []) { } })");
expectSyntaxError("({ get x([], a, {}) { } })");
expectSyntaxError("({ get x({}, a, []) { } })");
expectSyntaxError("({ get x({}, a, {}) { } })");

expectOk("({ get x() { } })");


expectSyntaxError("({ set x() { } })");
expectSyntaxError("({ set x(a, []) { } })");
expectSyntaxError("({ set x(a, b, c) { } })");

expectOk("({ set x([]) { } })");
expectOk("({ set x({}) { } })");
expectOk("({ set x([a]) { } })");
expectOk("({ set x([a, b]) { } })");
expectOk("({ set x([a,]) { } })");
expectOk("({ set x([a, b,]) { } })");
expectOk("({ set x([, b]) { } })");
expectOk("({ set x([, b,]) { } })");
expectOk("({ set x([, b, c]) { } })");
expectOk("({ set x([, b, c,]) { } })");
expectOk("({ set x({ a: a }) { } })");
expectOk("({ set x({ a: a, b: b }) { } })");
