/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Accessing a revocable proxy's [[Prototype]] shouldn't crash
info: bugzilla.mozilla.org/show_bug.cgi?id=1052139
esid: pending
---*/

function checkFunctionAppliedToRevokedProxy(fun)
{
  var p = Proxy.revocable({}, {});
  p.revoke();

  assert.throws(TypeError, function() {
    fun(p.proxy);
  });
}

checkFunctionAppliedToRevokedProxy(proxy => Object.getPrototypeOf(proxy));
checkFunctionAppliedToRevokedProxy(proxy => Object.setPrototypeOf(proxy, null));
