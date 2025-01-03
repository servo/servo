/*
 * Copyright 2017 WebAssembly Community Group participants
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
*/

'use strict';

let testNum = (function() {
    let count = 1;
    return function() {
        return `#${count++} `;
    }
})();

// WPT's assert_throw uses a list of predefined, hardcoded known errors. Since
// it is not aware of the WebAssembly error types (yet), implement our own
// version.
function assertThrows(func, err) {
    let caught = false;
    try {
        func();
    } catch(e) {
        assert_true(e instanceof err, `expected ${err.name}, observed ${e.constructor.name}`);
        caught = true;
    }
    assert_true(caught, testNum() + "assertThrows must catch any error.")
}

/******************************************************************************
***************************** WAST HARNESS ************************************
******************************************************************************/

// For assertions internal to our test harness.
function _assert(x) {
    if (!x) {
        throw new Error(`Assertion failure: ${x}`);
    }
}

// A simple sum type that can either be a valid Value or an Error.
function Result(type, maybeValue) {
    this.value = maybeValue;
    this.type = type;
};

Result.VALUE = 'VALUE';
Result.ERROR = 'ERROR';

function ValueResult(val) { return new Result(Result.VALUE, val); }
function ErrorResult(err) { return new Result(Result.ERROR, err); }

Result.prototype.isError = function() { return this.type === Result.ERROR; }

const EXPECT_INVALID = false;

/* DATA **********************************************************************/

let externrefs = {};
let externsym = Symbol("externref");
function externref(s) {
  if (! (s in externrefs)) externrefs[s] = {[externsym]: s};
  return externrefs[s];
}
function is_externref(x) {
  return (x !== null && externsym in x) ? 1 : 0;
}
function is_funcref(x) {
  return typeof x === "function" ? 1 : 0;
}
function eq_externref(x, y) {
  return x === y ? 1 : 0;
}
function eq_funcref(x, y) {
  return x === y ? 1 : 0;
}

let $$;

// Default imports.
var registry = {};

// Resets the registry between two different WPT tests.
function reinitializeRegistry() {
    if (typeof WebAssembly === 'undefined')
        return;

    let spectest = {
        externref: externref,
        is_externref: is_externref,
        is_funcref: is_funcref,
        eq_externref: eq_externref,
        eq_funcref: eq_funcref,
        print: console.log.bind(console),
        print_i32: console.log.bind(console),
        print_i64: console.log.bind(console),
        print_i32_f32: console.log.bind(console),
        print_f64_f64: console.log.bind(console),
        print_f32: console.log.bind(console),
        print_f64: console.log.bind(console),
        global_i32: 666,
        global_i64: 666n,
        global_f32: 666.6,
        global_f64: 666.6,
        table: new WebAssembly.Table({initial: 10, maximum: 20, element: 'anyfunc'}),
        memory: new WebAssembly.Memory({initial: 1, maximum: 2})
    };
    let handler = {
        get(target, prop) {
        return (prop in target) ?  target[prop] : {};
      }
    };
    registry = new Proxy({spectest}, handler);
}

reinitializeRegistry();

/* WAST POLYFILL *************************************************************/

function binary(bytes) {
    let buffer = new ArrayBuffer(bytes.length);
    let view = new Uint8Array(buffer);
    for (let i = 0; i < bytes.length; ++i) {
        view[i] = bytes.charCodeAt(i);
    }
    return buffer;
}

/**
 * Returns a compiled module, or throws if there was an error at compilation.
 */
function module(bytes, valid = true) {
    let buffer = binary(bytes);
    let validated;

    try {
        validated = WebAssembly.validate(buffer);
    } catch (e) {
        throw new Error(`WebAssembly.validate throws ${typeof e}: ${e}${e.stack}`);
    }

    if (validated !== valid) {
        // Try to get a more precise error message from the WebAssembly.CompileError.
        try {
            new WebAssembly.Module(buffer);
        } catch (e) {
            if (e instanceof WebAssembly.CompileError)
                throw new WebAssembly.CompileError(`WebAssembly.validate error: ${e.toString()}${e.stack}\n`);
            else
                throw new Error(`WebAssembly.validate throws ${typeof e}: ${e}${e.stack}`);
        }
        throw new Error(`WebAssembly.validate was expected to fail, but didn't`);
    }

    let module;
    try {
        module = new WebAssembly.Module(buffer);
    } catch(e) {
        if (valid)
            throw new Error('WebAssembly.Module ctor unexpectedly throws ${typeof e}: ${e}${e.stack}');
        throw e;
    }

    return module;
}

function uniqueTest(func, desc) {
    test(func, testNum() + desc);
}

function assert_invalid(bytes) {
    uniqueTest(() => {
        try {
            module(bytes, /* valid */ false);
            throw new Error('did not fail');
        } catch(e) {
            assert_true(e instanceof WebAssembly.CompileError, "expected invalid failure:");
        }
    }, "A wast module that should be invalid or malformed.");
}

const assert_malformed = assert_invalid;

function instance(bytes, imports = registry, valid = true) {
    if (imports instanceof Result) {
        if (imports.isError())
            return imports;
        imports = imports.value;
    }

    let err = null;

    let m, i;
    try {
        let m = module(bytes);
        i = new WebAssembly.Instance(m, imports);
    } catch(e) {
        err = e;
    }

    if (valid) {
        uniqueTest(() => {
            let instantiated = err === null;
            assert_true(instantiated, err);
        }, "module successfully instantiated");
    }

    return err !== null ? ErrorResult(err) : ValueResult(i);
}

function register(name, instance) {
    _assert(instance instanceof Result);

    if (instance.isError())
        return;

    registry[name] = instance.value.exports;
}

function call(instance, name, args) {
    _assert(instance instanceof Result);

    if (instance.isError())
        return instance;

    let err = null;
    let result;
    try {
        result = instance.value.exports[name](...args);
    } catch(e) {
        err = e;
    }

    return err !== null ? ErrorResult(err) : ValueResult(result);
};

function get(instance, name) {
    _assert(instance instanceof Result);

    if (instance.isError())
        return instance;

    let v = instance.value.exports[name];
    return ValueResult((v instanceof WebAssembly.Global) ? v.value : v);
}

function exports(instance) {
    _assert(instance instanceof Result);

    if (instance.isError())
        return instance;

    return ValueResult({ module: instance.value.exports, spectest: registry.spectest });
}

function run(action) {
    let result = action();

    _assert(result instanceof Result);

    uniqueTest(() => {
        if (result.isError())
            throw result.value;
    }, "A wast test that runs without any special assertion.");
}

function assert_unlinkable(bytes) {
    let result = instance(bytes, registry, EXPECT_INVALID);

    _assert(result instanceof Result);

    uniqueTest(() => {
        assert_true(result.isError(), 'expected error result');
        if (result.isError()) {
            let e = result.value;
            assert_true(e instanceof WebAssembly.LinkError, `expected link error, observed ${e}:`);
        }
    }, "A wast module that is unlinkable.");
}

function assert_uninstantiable(bytes) {
    let result = instance(bytes, registry, EXPECT_INVALID);

    _assert(result instanceof Result);

    uniqueTest(() => {
        assert_true(result.isError(), 'expected error result');
        if (result.isError()) {
            let e = result.value;
            assert_true(e instanceof WebAssembly.RuntimeError, `expected runtime error, observed ${e}:`);
        }
    }, "A wast module that is uninstantiable.");
}

function assert_trap(action) {
    let result = action();

    _assert(result instanceof Result);

    uniqueTest(() => {
        assert_true(result.isError(), 'expected error result');
        if (result.isError()) {
            let e = result.value;
            assert_true(e instanceof WebAssembly.RuntimeError, `expected runtime error, observed ${e}:`);
        }
    }, "A wast module that must trap at runtime.");
}

let StackOverflow;
try { (function f() { 1 + f() })() } catch (e) { StackOverflow = e.constructor }

function assert_exhaustion(action) {
    let result = action();

    _assert(result instanceof Result);

    uniqueTest(() => {
        assert_true(result.isError(), 'expected error result');
        if (result.isError()) {
            let e = result.value;
            assert_true(e instanceof StackOverflow, `expected stack overflow error, observed ${e}:`);
        }
    }, "A wast module that must exhaust the stack space.");
}

function assert_return(action, ...expected) {
    let result = action();
    _assert(result instanceof Result);

    uniqueTest(() => {
        assert_true(!result.isError(), `expected success result, got: ${result.value}.`);
        let actual = result.value;
        if (actual === undefined) {
            actual = [];
        } else if (!Array.isArray(actual)) {
            actual = [actual];
        }
        if (actual.length !== expected.length) {
            throw new Error(expected.length + " value(s) expected, got " + actual.length);
        }
        for (let i = 0; i < actual.length; ++i) {
            if (expected[i] instanceof Result) {
                if (expected[i].isError())
                    return;
                expected[i] = expected[i].value;
            }
            switch (expected[i]) {
                case "nan:canonical":
                case "nan:arithmetic":
                case "nan:any":
                    // Note that JS can't reliably distinguish different NaN values,
                    // so there's no good way to test that it's a canonical NaN.
                    assert_true(Number.isNaN(actual[i]), `expected NaN, observed ${actual[i]}.`);
                    return;
                case "ref.func":
                    assert_true(typeof actual[i] === "function", `expected Wasm function, got ${actual[i]}`);
                    return;
                case "ref.extern":
                    assert_true(actual[i] !== null, `expected Wasm reference, got ${actual[i]}`);
                    return;
                default:
                    assert_equals(actual[i], expected[i]);
            }
        }
    }, "A wast module that must return a particular value.");
}

function assert_return_nan(action) {
    let result = action();

    _assert(result instanceof Result);

    uniqueTest(() => {
        assert_true(!result.isError(), 'expected success result');
        if (!result.isError()) {
            assert_true(Number.isNaN(result.value), `expected NaN, observed ${result.value}.`);
        };
    }, "A wast module that must return NaN.");
}
