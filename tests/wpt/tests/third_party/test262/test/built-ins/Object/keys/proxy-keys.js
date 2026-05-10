// Copyright (C) 2017 Caitlin Potter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.keys
description: >
  Object.keys() observably performs [[GetOwnProperty]]
info: |
  19.1.2.16 Object.keys ( O )

  1. Let obj be ? ToObject(O).
  2. Let nameList be ? EnumerableOwnProperties(obj, "key").
  ...

  7.3.21 EnumerableOwnProperties ( O, kind )

  1. Assert: Type(O) is Object.
  2. Let ownKeys be ? O.[[OwnPropertyKeys]]().
  3. Let properties be a new empty List.
  4. For each element key of ownKeys in List order, do
    a. If Type(key) is String, then
      i. Let desc be ? O.[[GetOwnProperty]](key).
      ...
features: [Symbol]
---*/

let log = [];
let s = Symbol("test");
let target = {
  x: true
};

let ownKeys = {
  get length() {
    log.push({
      name: "get ownKeys['length']",
      receiver: this
    });
    return 3;
  },

  get 0() {
    log.push({
      name: "get ownKeys[0]",
      receiver: this
    });
    return "a";
  },

  get 1() {
    log.push({
      name: "get ownKeys[1]",
      receiver: this
    });
    return s;
  },

  get 2() {
    log.push({
      name: "get ownKeys[2]",
      receiver: this
    });
    return "b";
  }
};

let ownKeysDescriptors = {
  "a": {
    enumerable: true,
    configurable: true,
    value: 1
  },

  "b": {
    enumerable: false,
    configurable: true,
    value: 2
  },

  [s]: {
    enumerable: true,
    configurable: true,
    value: 3
  }
};

let handler = {
  get ownKeys() {
    log.push({
      name: "get handler.ownKeys",
      receiver: this
    });
    return (...args) => {
      log.push({
        name: "call handler.ownKeys",
        receiver: this,
        args
      });
      return ownKeys;
    };
  },

  get getOwnPropertyDescriptor() {
    log.push({
      name: "get handler.getOwnPropertyDescriptor",
      receiver: this
    });
    return (...args) => {
      log.push({
        name: "call handler.getOwnPropertyDescriptor",
        receiver: this,
        args
      });
      const name = args[1];
      return ownKeysDescriptors[name];
    };
  }
};

let proxy = new Proxy(target, handler);
let keys = Object.keys(proxy);

assert.sameValue(log.length, 10);

assert.sameValue(log[0].name, "get handler.ownKeys");
assert.sameValue(log[0].receiver, handler);

assert.sameValue(log[1].name, "call handler.ownKeys");
assert.sameValue(log[1].receiver, handler);
assert.sameValue(log[1].args.length, 1);
assert.sameValue(log[1].args[0], target);

// CreateListFromArrayLike(trapResultArray, « String, Symbol »).
assert.sameValue(log[2].name, "get ownKeys['length']");
assert.sameValue(log[2].receiver, ownKeys);

assert.sameValue(log[3].name, "get ownKeys[0]");
assert.sameValue(log[3].receiver, ownKeys);

assert.sameValue(log[4].name, "get ownKeys[1]");
assert.sameValue(log[4].receiver, ownKeys);

assert.sameValue(log[5].name, "get ownKeys[2]");
assert.sameValue(log[5].receiver, ownKeys);

// Let desc be ? O.[[GetOwnProperty]]("a").
assert.sameValue(log[6].name, "get handler.getOwnPropertyDescriptor");
assert.sameValue(log[6].receiver, handler);

assert.sameValue(log[7].name, "call handler.getOwnPropertyDescriptor");
assert.sameValue(log[7].receiver, handler);
assert.sameValue(log[7].args.length, 2);
assert.sameValue(log[7].args[0], target);
assert.sameValue(log[7].args[1], "a");

// Let desc be ? O.[[GetOwnProperty]]("b").
assert.sameValue(log[8].name, "get handler.getOwnPropertyDescriptor");
assert.sameValue(log[8].receiver, handler);

assert.sameValue(log[9].name, "call handler.getOwnPropertyDescriptor");
assert.sameValue(log[9].receiver, handler);
assert.sameValue(log[9].args.length, 2);
assert.sameValue(log[9].args[0], target);
assert.sameValue(log[9].args[1], "b");

// "a" is the only enumerable String-keyed property.
assert.sameValue(keys.length, 1);
assert.sameValue(keys[0], "a");
