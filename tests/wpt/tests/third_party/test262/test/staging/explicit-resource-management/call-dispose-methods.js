// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Test if disposed methods are called in correct syntax.
includes: [compareArray.js]
features: [explicit-resource-management]
---*/

// Block ----------------
let blockValues = [];

(function TestUsingInBlock() {
  {
    using x = {
      value: 1,
      [Symbol.dispose]() {
        blockValues.push(42);
      }
    };
    blockValues.push(43);
  }
})();
assert.compareArray(blockValues, [43, 42]);

// ForStatement --------------
let forStatementValues = [];

(function TestUsingInForStatement() {
  for (let i = 0; i < 3; i++) {
    using x = {
      value: i,
      [Symbol.dispose]() {
        forStatementValues.push(this.value);
      }
    };
  }
  forStatementValues.push(3);
})();
assert.compareArray(forStatementValues, [0, 1, 2, 3]);

// ForInStatement --------------
let forInStatementValues = [];

(function TestUsingInForInStatement() {
  for (let i in [0, 1]) {
    using x = {
      value: i,
      [Symbol.dispose]() {
        forInStatementValues.push(this.value);
      }
    };
  }
  forInStatementValues.push('2');
})();
assert.compareArray(forInStatementValues, ['0', '1', '2']);

// ForOfStatement --------------
let forOfStatementValues = [];

(function TestUsingInForOfStatement() {
  for (let i of [0, 1]) {
    using x = {
      value: i,
      [Symbol.dispose]() {
        forOfStatementValues.push(this.value);
      }
    };
  }
  forOfStatementValues.push(2);
})();
assert.compareArray(forOfStatementValues, [0, 1, 2]);

// FunctionBody --------------
let functionBodyValues = [];

(function TestUsingInFunctionBody() {
  using x = {
    value: 1,
    [Symbol.dispose]() {
      functionBodyValues.push(42);
    }
  };
  using y = {
    value: 2,
    [Symbol.dispose]() {
      functionBodyValues.push(43);
    }
  };
})();
assert.compareArray(functionBodyValues, [43, 42]);

// GeneratorBody --------------
let generatorBodyValues = [];

function* gen() {
  using x = {
    value: 1,
    [Symbol.dispose]() {
      generatorBodyValues.push(42);
    }
  };
  yield x;
}

(function TestUsingInGeneratorBody() {
  let iter = gen();
  iter.next();
  iter.next();
  generatorBodyValues.push(43);
})();
assert.compareArray(generatorBodyValues, [42, 43]);

// ClassStaticBlockBody --------------
let classStaticBlockBodyValues = [];

class staticBlockClass {
  static {
    using x = {
      value: 1,
      [Symbol.dispose]() {
        classStaticBlockBodyValues.push(42);
      }
    };
  }
}

(function TestUsingInAsyncFunctionBody() {
  let example = new staticBlockClass();
})();
assert.compareArray(classStaticBlockBodyValues, [42]);

// Derived constructor case
let derivedConstructorValues = [];

class baseClass {
  constructor() {
    derivedConstructorValues.push(43);
  }
}

class subClass extends baseClass {
  constructor() {
    try {
      using x = {
        value: 1,
        [Symbol.dispose]() {
          derivedConstructorValues.push(42);
        }
      };
    } catch (e) {
      return;
    } finally {
      super();
    }
  }
}

(function TestUsingInDerivedConstructor() {
  let example = new subClass();
})();
assert.compareArray(derivedConstructorValues, [42, 43]);

// Lack of dispose method
let values = [];

function TestUsingWithoutDisposeMethod() {
  {
    using x = {value: 1};
    values.push(43);
  }
}
assert.throws(TypeError, TestUsingWithoutDisposeMethod, 'No dispose method');
