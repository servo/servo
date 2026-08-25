/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Passing a Date object to |new Date()| should copy it, not convert it to a primitive and create it from that.
info: bugzilla.mozilla.org/show_bug.cgi?id=1187233
esid: pending
---*/

Date.prototype.toString = Date.prototype.valueOf = null;
var d = new Date(new Date(8675309));
assert.sameValue(d.getTime(), 8675309);

Date.prototype.valueOf = () => 42;
d = new Date(new Date(8675309));
assert.sameValue(d.getTime(), 8675309);

var D = $262.createRealm().global.Date;

D.prototype.toString = D.prototype.valueOf = null;
var d = new Date(new D(3141592654));
assert.sameValue(d.getTime(), 3141592654);

D.prototype.valueOf = () => 525600;
d = new Date(new D(3141592654));
assert.sameValue(d.getTime(), 3141592654);
