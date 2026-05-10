// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createdynamicfunction
description: CreateDynamicFunction throws SyntaxError if there is some invalid private identifier on its body
info: |
  CreateDynamicFunction(constructor, newTarget, kind, args)
    ...
    29. Let privateIdentifiers be an empty List.
    30. If AllPrivateIdentifiersValid of body with the argument privateIdentifiers is false, throw a SyntaxError exception.
    31. If AllPrivateIdentifiersValid of parameters with the argument privateIdentifiers is false, throw a SyntaxError exception.
    ...
features: [class-fields-private]
---*/

assert.throws(SyntaxError, function() {
  let o = {};
  new Function("o.#f");
}, 'It should be a SyntaxError if AllPrivateIdentifiersValid returns false to dynamic function body');

