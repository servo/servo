// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.13
description: >
    Iterators that are implemented as proxies should behave identically to
    non-proxy versions.
features: [Proxy, Symbol.iterator]
---*/

var iterable = {};
var nextResult = { value: 23, done: false };
var lastResult = { value: null, done: true };
var i;

var iterator = {
  next: function() {
    var result = nextResult;
    nextResult = lastResult;
    return result;
  }
};
var proxiedIterator = new Proxy(iterator, {
  get: function(target, name) {
    return target[name];
  }
});
iterable[Symbol.iterator] = function() { return proxiedIterator; };

i = 0;
for (var x of iterable) {
  assert.sameValue(x, 23);
  i++;
}

assert.sameValue(i, 1);
