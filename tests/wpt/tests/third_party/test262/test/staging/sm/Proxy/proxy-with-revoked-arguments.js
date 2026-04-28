// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Proxy constructor should not throw if either the target or handler is a revoked proxy.
info: bugzilla.mozilla.org/show_bug.cgi?id=1151149
esid: pending
---*/

var p = new Proxy({}, {});

new Proxy(p, {});
new Proxy({}, p);

var r = Proxy.revocable({}, {});
p = r.proxy;

new Proxy(p, {});
new Proxy({}, p);

r.revoke();

new Proxy(p, {});
new Proxy({}, p);


var r2 = Proxy.revocable({}, {});
r = Proxy.revocable(r2.proxy, {});
p = r.proxy;

new Proxy(p, {});
new Proxy({}, p);

r2.revoke();

new Proxy(p, {});
new Proxy({}, p);

r.revoke();

new Proxy(p, {});
new Proxy({}, p);


var g = $262.createRealm().global;
p = g.eval(`var r = Proxy.revocable({}, {}); r.proxy;`);

new Proxy(p, {});
new Proxy({}, p);

g.eval(`r.revoke();`);

new Proxy(p, {});
new Proxy({}, p);
