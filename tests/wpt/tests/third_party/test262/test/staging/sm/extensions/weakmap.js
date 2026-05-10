/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  js weak maps
info: bugzilla.mozilla.org/show_bug.cgi?id=547941
esid: pending
features: [host-gc-required]
---*/

test();

function test()
{
    function check(fun) {
        assert.sameValue(fun(), true);
    }

    function checkThrows(fun) {
        assert.throws(TypeError, fun);
    }

    var key = {};
    var map = new WeakMap();

    check(() => !map.has(key));
    check(() => map.delete(key) == false);
    check(() => map.set(key, 42) === map);
    check(() => map.get(key) == 42);
    check(() => typeof map.get({}) == "undefined");
    check(() => map.get({}, "foo") == undefined);

    $262.gc(); $262.gc(); $262.gc();

    check(() => map.get(key) == 42);
    check(() => map.delete(key) == true);
    check(() => map.delete(key) == false);
    check(() => map.delete({}) == false);

    check(() => typeof map.get(key) == "undefined");
    check(() => !map.has(key));
    check(() => map.delete(key) == false);

    var value = { };
    check(() => map.set(new Object(), value) === map);
    $262.gc(); $262.gc(); $262.gc();

    check(() => map.has("non-object key") == false);
    check(() => map.has() == false);
    check(() => map.get("non-object key") == undefined);
    check(() => map.get() == undefined);
    check(() => map.delete("non-object key") == false);
    check(() => map.delete() == false);

    check(() => map.set(key) === map);
    check(() => map.get(key) == undefined);

    checkThrows(() => map.set("non-object key", value));
}
