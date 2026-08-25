// Copyright 2019 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: >
    The class-name is present when executing static field initializers of default-exported classes.
info: |
    14.6.13 Runtime Semantics: ClassDefinitionEvaluation

    [...]
    17. Perform MakeClassConstructor(F).
    18. If className is not undefined, then
        a. Perform SetFunctionName(F, className).
    [...]

flags: [module]
features: [class-static-fields-public]
---*/

var className;

export default class {
    static f = (className = this.name);
}

assert.sameValue(className, "default");
