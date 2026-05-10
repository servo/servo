// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Anonymous function name should be set based on property name
info: bugzilla.mozilla.org/show_bug.cgi?id=883377
esid: pending
---*/

var fooSymbol = Symbol("foo");
var emptySymbol = Symbol("");
var undefSymbol = Symbol();

var exprs = [
    ["function() {}", false],
    ["function named() {}", true],
    ["function*() {}", false],
    ["function* named() {}", true],
    ["async function() {}", false],
    ["async function named() {}", true],
    ["() => {}", false],
    ["async () => {}", false],
    ["class {}", false],
    ["class named {}", true],
];

function testPropertyDefinition(expr, named) {
    var obj = eval(`({
        prop: ${expr},
        "literal": ${expr},
        "": ${expr},
        5: ${expr},
        0.4: ${expr},
        [Symbol.iterator]: ${expr},
        [fooSymbol]: ${expr},
        [emptySymbol]: ${expr},
        [undefSymbol]: ${expr},
        [/a/]: ${expr},
    })`);
    assert.sameValue(obj.prop.name, named ? "named" : "prop");
    assert.sameValue(obj["literal"].name, named ? "named" : "literal");
    assert.sameValue(obj[""].name, named ? "named" : "");
    assert.sameValue(obj[5].name, named ? "named" : "5");
    assert.sameValue(obj[0.4].name, named ? "named" : "0.4");
    assert.sameValue(obj[Symbol.iterator].name, named ? "named" : "[Symbol.iterator]");
    assert.sameValue(obj[fooSymbol].name, named ? "named" : "[foo]");
    assert.sameValue(obj[emptySymbol].name, named ? "named" : "[]");
    assert.sameValue(obj[undefSymbol].name, named ? "named" : "");
    assert.sameValue(obj[/a/].name, named ? "named" : "/a/");

    // Not applicable cases: __proto__.
    obj = {
        __proto__: function() {}
    };
    assert.sameValue(obj.__proto__.name, "");
}
for (var [expr, named] of exprs) {
    testPropertyDefinition(expr, named);
}
