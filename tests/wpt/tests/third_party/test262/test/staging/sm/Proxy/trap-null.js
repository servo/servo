/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  null as a trap value on a handler should operate on the target
info: bugzilla.mozilla.org/show_bug.cgi?id=1257102
esid: pending
---*/

// This might seem like overkill, but this proxying trick caught typos of
// several trap names before this test landed.  \o/  /o\
var allTraps = {
  getPrototypeOf: null,
  setPrototypeOf: null,
  isExtensible: null,
  preventExtensions: null,
  getOwnPropertyDescriptor: null,
  defineProperty: null,
  has: null,
  get: null,
  set: null,
  deleteProperty: null,
  ownKeys: null,
  apply: null,
  construct: null,
};

var complainingHandler = new Proxy(allTraps, {
  get(target, p, receiver) {
    var v = Reflect.get(target, p, receiver);
    if (v !== null)
      throw new TypeError("failed to set one of the traps to null?  " + p);

    return v;
  }
});

var proxyTarget = function(x) {
  "use strict";
  var str = this + x;
  str += new.target ? "constructing" : "calling";
  return new.target ? new String(str) : str;
};
proxyTarget.prototype.toString = () => "###";
proxyTarget.prototype.valueOf = () => "@@@";

var proxy = new Proxy(proxyTarget, complainingHandler);

assert.sameValue(Reflect.getPrototypeOf(proxy), Function.prototype);

assert.sameValue(Object.getPrototypeOf(proxyTarget), Function.prototype);
assert.sameValue(Reflect.setPrototypeOf(proxy, null), true);
assert.sameValue(Object.getPrototypeOf(proxyTarget), null);

assert.sameValue(Reflect.isExtensible(proxy), true);

assert.sameValue(Reflect.isExtensible(proxyTarget), true);
assert.sameValue(Reflect.preventExtensions(proxy), true);
assert.sameValue(Reflect.isExtensible(proxy), false);
assert.sameValue(Reflect.isExtensible(proxyTarget), false);

var desc = Reflect.getOwnPropertyDescriptor(proxy, "length");
assert.sameValue(desc.value, 1);

assert.sameValue(desc.configurable, true);
assert.sameValue(Reflect.defineProperty(proxy, "length", { value: 3, configurable: false }), true);
desc = Reflect.getOwnPropertyDescriptor(proxy, "length");
assert.sameValue(desc.configurable, false);

assert.sameValue(Reflect.has(proxy, "length"), true);

assert.sameValue(Reflect.get(proxy, "length"), 3);

assert.sameValue(Reflect.set(proxy, "length", 3), false);

assert.sameValue(Reflect.deleteProperty(proxy, "length"), false);

var keys = Reflect.ownKeys(proxy);
assert.sameValue(keys.length, 3);
keys.sort();
assert.sameValue(keys[0], "length");
assert.sameValue(keys[1], "name");
assert.sameValue(keys[2], "prototype");

assert.sameValue(Reflect.apply(proxy, "hi!", [" "]), "hi! calling");

var res = Reflect.construct(proxy, [" - "]);
assert.sameValue(typeof res, "object");
assert.sameValue(res instanceof String, true);
assert.sameValue(res.valueOf(), "@@@ - constructing");
