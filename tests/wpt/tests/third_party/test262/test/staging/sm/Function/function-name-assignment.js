// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  Anonymous function name should be set based on assignment
info: bugzilla.mozilla.org/show_bug.cgi?id=883377
esid: pending
---*/

var fooSymbol = Symbol("foo");
var emptySymbol = Symbol("");
var undefSymbol = Symbol();
var globalVar;

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

function testAssignmentExpression(expr, named) {
    eval(`
    var assignment;
    assignment = ${expr};
    assert.sameValue(assignment.name, named ? "named" : "assignment");

    globalVar = ${expr};
    assert.sameValue(globalVar.name, named ? "named" : "globalVar");

    var obj = { dynamic: null };
    with (obj) {
        dynamic = ${expr};
    }
    assert.sameValue(obj.dynamic.name, named ? "named" : "dynamic");

    (function namedLambda(param1, param2) {
        var assignedToNamedLambda;
        assignedToNamedLambda = namedLambda = ${expr};
        assert.sameValue(namedLambda.name, "namedLambda");
        assert.sameValue(assignedToNamedLambda.name, named ? "named" : "namedLambda");

        param1 = ${expr};
        assert.sameValue(param1.name, named ? "named" : "param1");

        {
            let param1 = ${expr};
            assert.sameValue(param1.name, named ? "named" : "param1");

            param2 = ${expr};
            assert.sameValue(param2.name, named ? "named" : "param2");
        }
    })();

    {
        let nextedLexical1, nextedLexical2;
        {
            let nextedLexical1 = ${expr};
            assert.sameValue(nextedLexical1.name, named ? "named" : "nextedLexical1");

            nextedLexical2 = ${expr};
            assert.sameValue(nextedLexical2.name, named ? "named" : "nextedLexical2");
        }
    }
    `);

    // Not applicable cases: not IsIdentifierRef.
    eval(`
    var inParen;
    (inParen) = ${expr};
    assert.sameValue(inParen.name, named ? "named" : "");
    `);

    // Not applicable cases: not direct RHS.
    if (!expr.includes("=>")) {
        eval(`
        var a = true && ${expr};
        assert.sameValue(a.name, named ? "named" : "");
        `);
    } else {
        // Arrow function cannot be RHS of &&.
        eval(`
        var a = true && (${expr});
        assert.sameValue(a.name, named ? "named" : "");
        `);
    }

    // Not applicable cases: property.
    eval(`
    var obj = {};

    obj.prop = ${expr};
    assert.sameValue(obj.prop.name, named ? "named" : "");

    obj["literal"] = ${expr};
    assert.sameValue(obj["literal"].name, named ? "named" : "");
    `);

    // Not applicable cases: assigned again.
    eval(`
    var tmp = [${expr}];
    assert.sameValue(tmp[0].name, named ? "named" : "");

    var assignment;
    assignment = tmp[0];
    assert.sameValue(assignment.name, named ? "named" : "");
    `);
}
for (var [expr, named] of exprs) {
    testAssignmentExpression(expr, named);
}

function testVariableDeclaration(expr, named) {
    eval(`
    var varDecl = ${expr};
    assert.sameValue(varDecl.name, named ? "named" : "varDecl");
    `);
}
for (var [expr, named] of exprs) {
    testVariableDeclaration(expr, named);
}

function testLexicalBinding(expr, named) {
    eval(`
    let lexical = ${expr};
    assert.sameValue(lexical.name, named ? "named" : "lexical");

    const constLexical = ${expr};
    assert.sameValue(constLexical.name, named ? "named" : "constLexical");
    `);
}
for (var [expr, named] of exprs) {
    testLexicalBinding(expr, named);
}
