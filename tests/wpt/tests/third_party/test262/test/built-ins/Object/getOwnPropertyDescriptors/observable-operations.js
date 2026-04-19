// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Object.getOwnPropertyDescriptors should perform observable operations in the correct order
esid: sec-object.getownpropertydescriptors
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
var result = Object.getOwnPropertyDescriptors(proxy);
assert.sameValue(log, "|ownKeys|getOwnPropertyDescriptor:a|getOwnPropertyDescriptor:b|getOwnPropertyDescriptor:c", 'log');
