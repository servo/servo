// Copyright (C) 2015 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.values
description: Object.values should perform observable operations in the correct order
author: Jordan Harband
features: [Proxy]
includes: [proxyTrapsHelper.js]
---*/

var log = "";
var object = {
  a: 0,
  b: 0,
  c: 0
};
var handler = allowProxyTraps({
  get: function(target, propertyKey, receiver) {
    assert.sameValue(target, object, "get target");
    assert.sameValue(receiver, proxy, "get receiver");
    log += "|get:" + propertyKey;
    return target[propertyKey];
  },
  getOwnPropertyDescriptor: function(target, propertyKey) {
    assert.sameValue(target, object, "getOwnPropertyDescriptor");
    log += "|getOwnPropertyDescriptor:" + propertyKey;
    return Object.getOwnPropertyDescriptor(target, propertyKey);
  },
  ownKeys: function(target) {
    assert.sameValue(target, object, "ownKeys");
    log += "|ownKeys";
    return Object.getOwnPropertyNames(target);
  }
});
var check = allowProxyTraps({
  get: function(target, propertyKey, receiver) {
    assert(propertyKey in target, "handler check: " + propertyKey);
    return target[propertyKey];
  }
});
var proxy = new Proxy(object, new Proxy(handler, check));
var result = Object.values(proxy);
assert.sameValue(log, "|ownKeys|getOwnPropertyDescriptor:a|get:a|getOwnPropertyDescriptor:b|get:b|getOwnPropertyDescriptor:c|get:c", log);
