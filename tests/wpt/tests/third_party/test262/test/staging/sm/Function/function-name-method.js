// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Anonymous function name should be set based on method definition
info: bugzilla.mozilla.org/show_bug.cgi?id=883377
esid: pending
---*/

var fooSymbol = Symbol("foo");
var emptySymbol = Symbol("");
var undefSymbol = Symbol();

function testMethod(prefix, classPrefix="", prototype=false) {
    var param = (prefix == "set" || prefix == "static set") ? "v" : "";
    var sep = classPrefix ? "" : ",";
    var objOrClass = eval(`(${classPrefix}{
        ${prefix} prop(${param}) {} ${sep}
        ${prefix} "literal"(${param}) {} ${sep}
        ${prefix} ""(${param}) {} ${sep}
        ${prefix} 5(${param}) {} ${sep}
        ${prefix} [Symbol.iterator](${param}) {} ${sep}
        ${prefix} [fooSymbol](${param}) {} ${sep}
        ${prefix} [emptySymbol](${param}) {} ${sep}
        ${prefix} [undefSymbol](${param}) {} ${sep}
        ${prefix} [/a/](${param}) {} ${sep}
    })`);

    var target = prototype ? objOrClass.prototype : objOrClass;

    function testOne(methodName, expectedName) {
        var f;
        if (prefix == "get" || prefix == "static get") {
            f = Object.getOwnPropertyDescriptor(target, methodName).get;
            expectedName = "get " + expectedName;
        } else if (prefix == "set" || prefix == "static set") {
            f = Object.getOwnPropertyDescriptor(target, methodName).set;
            expectedName = "set " + expectedName;
        } else {
            f = Object.getOwnPropertyDescriptor(target, methodName).value;
        }

        assert.sameValue(f.name, expectedName);
    }
    testOne("prop", "prop");
    testOne("literal", "literal");
    testOne("", "");
    testOne(5, "5");
    testOne(Symbol.iterator, "[Symbol.iterator]");
    testOne(fooSymbol, "[foo]");
    testOne(emptySymbol, "[]");
    testOne(undefSymbol, "");
    testOne(/a/, "/a/");
}
testMethod("");
testMethod("*");
testMethod("async");
testMethod("get");
testMethod("set");

testMethod("", "class", true);
testMethod("*", "class", true);
testMethod("async", "class", true);
testMethod("get", "class", true);
testMethod("set", "class", true);

testMethod("static", "class");
testMethod("static *", "class");
testMethod("static async", "class");
testMethod("static get", "class");
testMethod("static set", "class");
