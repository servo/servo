/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

function checkConstruct(thing) {
  assert.throws(TypeError, function() {
    new thing();
  });
}

var re = /aaa/
checkConstruct(re);

var boundFunctionPrototype = Function.prototype.bind();
checkConstruct(boundFunctionPrototype);

var boundBuiltin = Math.sin.bind();
checkConstruct(boundBuiltin);

var proxiedFunctionPrototype = new Proxy(Function.prototype, {});
checkConstruct(proxiedFunctionPrototype);

var proxiedBuiltin = new Proxy(parseInt, {});
checkConstruct(proxiedBuiltin);
