// META: global=window,worker
// META: title=IndexedDB: keys and values
// META: script=resources/support.js
// @author Odin Hørthe Omdal <mailto:odinho@opera.com>

'use_strict';

function setOnUpgradeNeeded(t, predicate, _instanceof, value) {
  createdb(t).onupgradeneeded = t.step_func(e => {
    const db = e.target.result;
    const store = db.createObjectStore("store");
    store.add(value, 1);

    e.target.onsuccess = t.step_func(e => {
      const transaction = db.transaction("store", "readonly", { durability: "relaxed" });
      const objectStore = transaction.objectStore("store");
      objectStore.get(1).onsuccess = t.step_func(e => {
        if (predicate) {
          assert_true(predicate(e.target.result),
            "Predicate should return true for the deserialized result.");
        } else if (_instanceof) {
          assert_true(e.target.result instanceof _instanceof, "instanceof");
        }
        t.done();
      });
    });
  });
}

// BigInt and BigInt objects are supported in serialization, per
// https://github.com/whatwg/html/pull/3480
// This support allows them to be used as IndexedDB values.

function value_test(value, predicate, name) {
  async_test(t => {
    t.step(function () {
      assert_true(predicate(value),
        "Predicate should return true for the initial value.");
    });

    setOnUpgradeNeeded(t, predicate, null, value);
  }, "BigInts as values in IndexedDB - " + name);
}

value_test(1n,
  x => x === 1n,
  "primitive BigInt");
value_test(Object(1n),
  x => typeof x === 'object' &&
    x instanceof BigInt &&
    x.valueOf() === 1n,
  "BigInt object");
value_test({ val: 1n },
  x => x.val === 1n,
  "primitive BigInt inside object");
value_test({ val: Object(1n) },
  x => x.val.valueOf() === 1n &&
    x.val instanceof BigInt &&
    x.val.valueOf() === 1n,
  "BigInt object inside object");

// However, BigInt is not supported as an IndexedDB key; support
// has been proposed in the following PR, but that change has not
// landed at the time this patch was written
// https://github.com/w3c/IndexedDB/pull/231

function invalidKey(key, name) {
  test(t => {
    assert_throws_dom("DataError", () => indexedDB.cmp(0, key));
  }, "BigInts as keys in IndexedDB - " + name);
}

invalidKey(1n, "primitive BigInt");
// Still an error even if the IndexedDB patch lands
invalidKey(Object(1n), "BigInt object");

function value(value, _instanceof) {
  async_test(t => {
    t.step(function () {
      assert_true(value instanceof _instanceof, "TEST ERROR, instanceof");
    });

    setOnUpgradeNeeded(t, null, _instanceof, value);
  }, "Values - " + _instanceof.name);
}

value(new Date(), Date);
value(new Array(), Array);
