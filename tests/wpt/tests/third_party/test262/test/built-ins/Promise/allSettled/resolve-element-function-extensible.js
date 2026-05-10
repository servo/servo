// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled-resolve-element-functions
description: The [[Extensible]] slot of Promise.allSettled Resolve Element functions
info: |
  17 ECMAScript Standard Built-in Objects:
    Unless specified otherwise, the [[Extensible]] internal slot
    of a built-in object initially has the value true.
features: [Promise.allSettled]
---*/

var resolveElementFunction;
var thenable = {
  then(fulfill) {
    resolveElementFunction = fulfill;
  }
};

function NotPromise(executor) {
  executor(function() {}, function() {});
}
NotPromise.resolve = function(v) {
  return v;
};
Promise.allSettled.call(NotPromise, [thenable]);

assert(Object.isExtensible(resolveElementFunction));
