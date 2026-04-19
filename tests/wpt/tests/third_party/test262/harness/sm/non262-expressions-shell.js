/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*---
defines: [testDestructuringArrayDefault]
---*/

(function(global) {
    function func() {
    }
    class C {
        foo() {
        }
        static foo() {
        }
    }

    function test_one(pattern, val, opt) {
        var stmts = [];
        var i = 0;
        var c;

        stmts.push(`var ${pattern} = ${val};`);

        for (var stmt of stmts) {
            if (!opt.no_plain) {
                eval(`
${stmt}
`);
            }

            if (!opt.no_func) {
                eval(`
function f${i}() {
  ${stmt}
}
f${i}();
`);
                i++;

                eval(`
var f${i} = function foo() {
  ${stmt}
};
f${i}();
`);
                i++;

                eval(`
var f${i} = () => {
  ${stmt}
};
f${i}();
`);
                i++;
            }

            if (!opt.no_gen) {
                eval(`
function* g${i}() {
  ${stmt}
}
[...g${i}()];
`);
                i++;

                eval(`
var g${i} = function* foo() {
  ${stmt}
};
[...g${i}()];
`);
                i++;
            }

            if (!opt.no_ctor) {
                eval(`
class D${i} {
  constructor() {
    ${stmt}
  }
}
new D${i}();
`);
                i++;
            }

            if (!opt.no_derived_ctor) {
                if (opt.no_pre_super) {
                    eval(`
class D${i} extends C {
  constructor() {
    ${stmt}
    try { super(); } catch (e) {}
  }
}
new D${i}();
`);
                    i++;
                } else {
                    eval(`
class D${i} extends C {
  constructor() {
    super();
    ${stmt}
  }
}
new D${i}();
`);
                    i++;
                }
            }

            if (!opt.no_method) {
                eval(`
class D${i} extends C {
  method() {
    ${stmt}
  }
  static staticMethod() {
    ${stmt}
  }
}
new D${i}().method();
D${i}.staticMethod();
`);
                i++;
            }
        }

        if (!opt.no_func_arg) {
            eval(`
function f${i}(${pattern}) {}
f${i}(${val});
`);
            i++;

            eval(`
var f${i} = function foo(${pattern}) {};
f${i}(${val});
`);
            i++;

            eval(`
var f${i} = (${pattern}) => {};
f${i}(${val});
`);
            i++;
        }

        if (!opt.no_gen_arg) {
            eval(`
function* g${i}(${pattern}) {}
[...g${i}(${val})];
`);
            i++;

            eval(`
var g${i} = function* foo(${pattern}) {};
[...g${i}(${val})];
`);
            i++;
        }
    }

    function test(expr, opt={}) {
        var pattern = `[a=${expr}, ...c]`;
        test_one(pattern, "[]", opt);
        test_one(pattern, "[1]", opt);

        pattern = `[,a=${expr}]`;
        test_one(pattern, "[]", opt);
        test_one(pattern, "[1]", opt);
        test_one(pattern, "[1, 2]", opt);

        pattern = `[{x: [a=${expr}]}]`;
        test_one(pattern, "[{x: [1]}]", opt);

        pattern = `[x=[a=${expr}]=[]]`;
        test_one(pattern, "[]", opt);
        test_one(pattern, "[1]", opt);

        pattern = `[x=[a=${expr}]=[1]]`;
        test_one(pattern, "[]", opt);
        test_one(pattern, "[1]", opt);
    }

    global.testDestructuringArrayDefault = test;
})(this);
