// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.parse
description: Behavior when revived value is a Proxy exotic object
info: |
  [...]
  7. If IsCallable(reviver) is true, then
     a. Let root be ObjectCreate(%ObjectPrototype%).
     b. Let rootName be the empty String.
     c. Let status be CreateDataProperty(root, rootName, unfiltered).
     d. Assert: status is true.
     e. Return ? InternalizeJSONProperty(root, rootName).

  24.3.1.1 Runtime Semantics: InternalizeJSONProperty

  [...]
  2. If Type(val) is Object, then
     a. Let isArray be ? IsArray(val).
     b. If isArray is true, then
        [...]
     c. Else,
        [...]

  7.2.2 IsArray

  [...]
  3. If argument is a Proxy exotic object, then
     a. If the value of the [[ProxyHandler]] internal slot of argument is null,
        throw a TypeError exception.
     b. Let target be the value of the [[ProxyTarget]] internal slot of
        argument.
     c. Return ? IsArray(target).
features: [Proxy]
---*/

var objectProxy = new Proxy({
  length: 0,
  other: 0
}, {});
var arrayProxy = new Proxy([], {});
var arrayProxyProxy = new Proxy(arrayProxy, {});
var visitedOther, injectProxy;

arrayProxy.other = 0;

injectProxy = function(name, val) {
  if (name === 'other') {
    visitedOther = true;
  }
  this[1] = objectProxy;
  return val;
};
visitedOther = false;

JSON.parse('[null, null]', injectProxy);

assert.sameValue(visitedOther, true, 'proxy for ordinary object');

injectProxy = function(name, val) {
  if (name === 'other') {
    visitedOther = true;
  }
  this[1] = arrayProxy;
  return val;
};
visitedOther = false;

JSON.parse('[null, null]', injectProxy);

assert.sameValue(visitedOther, false, 'proxy for array');

injectProxy = function(name, val) {
  if (name === 'other') {
    visitedOther = true;
  }
  this[1] = arrayProxyProxy;
  return val;
};
visitedOther = false;

JSON.parse('[null, null]', injectProxy);

assert.sameValue(visitedOther, false, 'proxy for array proxy');
