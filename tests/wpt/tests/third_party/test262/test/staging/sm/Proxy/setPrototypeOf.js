/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/assertThrowsValue.js]
description: |
  Scripted proxies' [[SetPrototypeOf]] behavior
info: bugzilla.mozilla.org/show_bug.cgi?id=888969
esid: pending
---*/

const log = [];

function observe(obj)
{
  var observingHandler = new Proxy({}, {
    get(target, p, receiver) {
      log.push(p);
      return Reflect.get(target, p, receiver);
    }
  });

  return new Proxy(obj, observingHandler);
}

var p, h;

// 1. Assert: Either Type(V) is Object or Type(V) is Null.
// 2. Let handler be the value of the [[ProxyHandler]] internal slot of O.
// 3. If handler is null, throw a TypeError exception.
// 4. Assert: Type(handler) is Object.
// 5. Let target be the value of the [[ProxyTarget]] internal slot of O.

var rev = Proxy.revocable({}, {});
p = rev.proxy;

var originalProto = Reflect.getPrototypeOf(p);
assert.sameValue(originalProto, Object.prototype);

rev.revoke();
assert.throws(TypeError, () => Reflect.setPrototypeOf(p, originalProto));
assert.throws(TypeError, () => Reflect.setPrototypeOf(p, null));

// 6. Let trap be ? GetMethod(handler, "setPrototypeOf").

// handler has uncallable (and not null/undefined) property
p = new Proxy({}, { setPrototypeOf: 9 });
assert.throws(TypeError, () => Reflect.setPrototypeOf(p, null));

p = new Proxy({}, { setPrototypeOf: -3.7 });
assert.throws(TypeError, () => Reflect.setPrototypeOf(p, null));

p = new Proxy({}, { setPrototypeOf: NaN });
assert.throws(TypeError, () => Reflect.setPrototypeOf(p, null));

p = new Proxy({}, { setPrototypeOf: Infinity });
assert.throws(TypeError, () => Reflect.setPrototypeOf(p, null));

p = new Proxy({}, { setPrototypeOf: true });
assert.throws(TypeError, () => Reflect.setPrototypeOf(p, null));

p = new Proxy({}, { setPrototypeOf: /x/ });
assert.throws(TypeError, () => Reflect.setPrototypeOf(p, null));

p = new Proxy({}, { setPrototypeOf: Symbol(42) });
assert.throws(TypeError, () => Reflect.setPrototypeOf(p, null));

p = new Proxy({}, { setPrototypeOf: class X {} });
assert.throws(TypeError, () => Reflect.setPrototypeOf(p, null));

p = new Proxy({}, observe({}));

assert.sameValue(Reflect.setPrototypeOf(p, Object.prototype), true);
assert.sameValue(log.length, 1);
assert.sameValue(log[0], "get");

h = observe({ setPrototypeOf() { throw 3.14; } });
p = new Proxy(Object.create(Object.prototype), h);

// "setting" without change
log.length = 0;
assertThrowsValue(() => Reflect.setPrototypeOf(p, Object.prototype),
                  3.14);
assert.sameValue(log.length, 1);
assert.sameValue(log[0], "get");

// "setting" with change
log.length = 0;
assertThrowsValue(() => Reflect.setPrototypeOf(p, /foo/),
                  3.14);
assert.sameValue(log.length, 1);
assert.sameValue(log[0], "get");

// 7. If trap is undefined, then
//    a. Return ? target.[[SetPrototypeOf]](V).

var settingProtoThrows =
  new Proxy({}, { setPrototypeOf() { throw "agnizes"; } });

p = new Proxy(settingProtoThrows, { setPrototypeOf: undefined });
assertThrowsValue(() => Reflect.setPrototypeOf(p, null),
                  "agnizes");

p = new Proxy(settingProtoThrows, { setPrototypeOf: null });
assertThrowsValue(() => Reflect.setPrototypeOf(p, null),
                  "agnizes");

var anotherProto =
  new Proxy({},
            { setPrototypeOf(t, p) {
                log.push("reached");
                return Reflect.setPrototypeOf(t, p);
              } });

p = new Proxy(anotherProto, { setPrototypeOf: undefined });

log.length = 0;
assert.sameValue(Reflect.setPrototypeOf(p, null), true);
assert.sameValue(log.length, 1);
assert.sameValue(log[0], "reached");

p = new Proxy(anotherProto, { setPrototypeOf: null });

log.length = 0;
assert.sameValue(Reflect.setPrototypeOf(p, null), true);
assert.sameValue(log.length, 1);
assert.sameValue(log[0], "reached");

// 8. Let booleanTrapResult be ToBoolean(? Call(trap, handler, « target, V »)).

// The trap callable might throw.
p = new Proxy({}, { setPrototypeOf() { throw "ohai"; } });

assertThrowsValue(() => Reflect.setPrototypeOf(p, /x/),
                  "ohai");

var throwingTrap =
  new Proxy(function() { throw "not called"; },
            { apply() { throw 37; } });

p = new Proxy({}, { setPrototypeOf: throwingTrap });

assertThrowsValue(() => Reflect.setPrototypeOf(p, Object.prototype),
                  37);

// The trap callable must *only* be called.
p = new Proxy({},
              {
                setPrototypeOf: observe(function() { throw "boo-urns"; })
              });

log.length = 0;
assertThrowsValue(() => Reflect.setPrototypeOf(p, Object.prototype),
                  "boo-urns");

assert.sameValue(log.length, 1);
assert.sameValue(log[0], "apply");

// 9. If booleanTrapResult is false, return false.

p = new Proxy({}, { setPrototypeOf() { return false; } });
assert.sameValue(Reflect.setPrototypeOf(p, Object.prototype), false);

p = new Proxy({}, { setPrototypeOf() { return +0; } });
assert.sameValue(Reflect.setPrototypeOf(p, Object.prototype), false);

p = new Proxy({}, { setPrototypeOf() { return -0; } });
assert.sameValue(Reflect.setPrototypeOf(p, Object.prototype), false);

p = new Proxy({}, { setPrototypeOf() { return NaN; } });
assert.sameValue(Reflect.setPrototypeOf(p, Object.prototype), false);

p = new Proxy({}, { setPrototypeOf() { return ""; } });
assert.sameValue(Reflect.setPrototypeOf(p, Object.prototype), false);

p = new Proxy({}, { setPrototypeOf() { return undefined; } });
assert.sameValue(Reflect.setPrototypeOf(p, Object.prototype), false);

// 10. Let extensibleTarget be ? IsExtensible(target).

var targetThrowIsExtensible =
  new Proxy({}, { isExtensible() { throw "psych!"; } });

p = new Proxy(targetThrowIsExtensible, { setPrototypeOf() { return true; } });
assertThrowsValue(() => Reflect.setPrototypeOf(p, Object.prototype),
                  "psych!");

// 11. If extensibleTarget is true, return true.

var targ = {};

p = new Proxy(targ, { setPrototypeOf() { return true; } });
assert.sameValue(Reflect.setPrototypeOf(p, /x/), true);
assert.sameValue(Reflect.getPrototypeOf(targ), Object.prototype);
assert.sameValue(Reflect.getPrototypeOf(p), Object.prototype);

p = new Proxy(targ, { setPrototypeOf() { return /x/; } });
assert.sameValue(Reflect.setPrototypeOf(p, /x/), true);
assert.sameValue(Reflect.getPrototypeOf(targ), Object.prototype);
assert.sameValue(Reflect.getPrototypeOf(p), Object.prototype);

p = new Proxy(targ, { setPrototypeOf() { return Infinity; } });
assert.sameValue(Reflect.setPrototypeOf(p, /x/), true);
assert.sameValue(Reflect.getPrototypeOf(targ), Object.prototype);
assert.sameValue(Reflect.getPrototypeOf(p), Object.prototype);

p = new Proxy(targ, { setPrototypeOf() { return Symbol(true); } });
assert.sameValue(Reflect.setPrototypeOf(p, /x/), true);
assert.sameValue(Reflect.getPrototypeOf(targ), Object.prototype);
assert.sameValue(Reflect.getPrototypeOf(p), Object.prototype);

// 12. Let targetProto be ? target.[[GetPrototypeOf]]().

var targetNotExtensibleGetProtoThrows =
  new Proxy(Object.preventExtensions({}),
            { getPrototypeOf() { throw NaN; } });

p = new Proxy(targetNotExtensibleGetProtoThrows,
              { setPrototypeOf() { log.push("goober"); return true; } });

log.length = 0;
assertThrowsValue(() => Reflect.setPrototypeOf(p, /abcd/),
                  NaN);

// 13. If SameValue(V, targetProto) is false, throw a TypeError exception.

var newProto;

p = new Proxy(Object.preventExtensions(Object.create(Math)),
              { setPrototypeOf(t, p) { return true; } });

assert.throws(TypeError, () => Reflect.setPrototypeOf(p, null));

// 14. Return true.

assert.sameValue(Reflect.setPrototypeOf(p, Math), true);
