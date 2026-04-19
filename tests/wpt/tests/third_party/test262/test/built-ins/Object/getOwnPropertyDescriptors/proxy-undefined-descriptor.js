// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Object.getOwnPropertyDescriptors should filter out undefined OwnPropertyDescriptors
esid: sec-object.getownpropertydescriptors
author: Jordan Harband
features: [Proxy]
includes: [proxyTrapsHelper.js]
---*/

var key = "a";
var ownKeys = [key];
var badProxyHandlers = allowProxyTraps({
  getOwnPropertyDescriptor: function() {},
  ownKeys: function() {
    return ownKeys;
  }
});
var proxy = new Proxy({}, badProxyHandlers);

var keys = Reflect.ownKeys(proxy);
assert.notSameValue(keys, ownKeys, 'Object.keys returns a new Array');
assert.sameValue(Array.isArray(keys), true, 'Object.keys returns an Array');
assert.sameValue(keys.length, ownKeys.length, 'keys and ownKeys have the same length');
assert.sameValue(keys[0], ownKeys[0], 'keys and ownKeys have the same contents');

var descriptor = Object.getOwnPropertyDescriptor(proxy, key);
assert.sameValue(descriptor, undefined, "Descriptor matches result of [[GetOwnPropertyDescriptor]] trap");

var result = Object.getOwnPropertyDescriptors(proxy);
assert.sameValue(key in result, false, "key is not present in result");
