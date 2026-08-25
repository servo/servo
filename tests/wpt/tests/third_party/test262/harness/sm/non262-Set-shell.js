/*---
defines: [assertSetContainsExactOrderedItems, SetLike, SetIteratorLike, LoggingProxy]
---*/
(function(global) {
  // Save the primordial values.
  const {Array, Error, Object, Proxy, Reflect, Set} = global;

  const ArrayIsArray = Array.isArray;
  const ReflectApply = Reflect.apply;
  const ReflectDefineProperty = Reflect.defineProperty;
  const ReflectGet = Reflect.get;
  const ReflectGetPrototypeOf = Reflect.getPrototypeOf;
  const SetPrototype = Set.prototype;
  const SetPrototypeHas = SetPrototype.has;
  const SetPrototypeSize = Object.getOwnPropertyDescriptor(SetPrototype, "size").get;
  const SetPrototypeKeys = SetPrototype.keys;
  const SetIteratorPrototypeNext = new Set().values().next;

  function assertSetContainsExactOrderedItems(actual, expected) {
    assert.sameValue(ReflectGetPrototypeOf(actual), SetPrototype, "actual is a native Set object");
    assert.sameValue(ArrayIsArray(expected), true, "expected is an Array object");

    assert.sameValue(ReflectApply(SetPrototypeSize, actual, []), expected.length);

    let index = 0;
    let keys = ReflectApply(SetPrototypeKeys, actual, []);

    while (true) {
      let {done, value: item} = ReflectApply(SetIteratorPrototypeNext, keys, []);
      if (done) {
        break;
      }
      assert.sameValue(item, expected[index], `Element at index ${index}:`);
      index++;
    }
  }
  global.assertSetContainsExactOrderedItems = assertSetContainsExactOrderedItems;

  class SetLike {
    #set;

    constructor(values) {
      this.#set = new Set(values);
    }

    get size() {
      return ReflectApply(SetPrototypeSize, this.#set, []);
    }

    has(value) {
      return ReflectApply(SetPrototypeHas, this.#set, [value]);
    }

    keys() {
      let keys = ReflectApply(SetPrototypeKeys, this.#set, []);
      return new SetIteratorLike(keys);
    }
  }
  global.SetLike = SetLike;

  class SetIteratorLike {
    #keys;

    constructor(keys) {
      this.#keys = keys;
    }

    next() {
      return ReflectApply(SetIteratorPrototypeNext, this.#keys, []);
    }

    // The |return| method of the iterator protocol is never called.
    return() {
      throw new Error("Unexpected call to |return| method");
    }

    // The |throw| method of the iterator protocol is never called.
    throw() {
      throw new Error("Unexpected call to |throw| method");
    }
  }

  function LoggingProxy(obj, log) {
    assert.sameValue(ArrayIsArray(log), true);

    let handler = new Proxy({
      get(t, pk, r) {
        ReflectDefineProperty(log, log.length, {
          value: pk, writable: true, enumerable: true, configurable: true,
        });
        return ReflectGet(t, pk, r);
      }
    }, {
      get(t, pk, r) {
        ReflectDefineProperty(log, log.length, {
          value: `[[${pk}]]`, writable: true, enumerable: true, configurable: true,
        });
        return ReflectGet(t, pk, r);
      }
    });

    return {obj, proxy: new Proxy(obj, handler)};
  }
  global.LoggingProxy = LoggingProxy;
})(this);
