// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Use ArraySpeciesCreate in Array.prototype.{concat,filter,map,slice,splice}.
info: bugzilla.mozilla.org/show_bug.cgi?id=1165052
esid: pending
---*/

var g = $262.createRealm().global;

function test(funcName, args, expectedLength, expectedLogs) {
  // modified @@species
  function FakeArray(n) {
    this.length = n;
  }
  var a = [1, 2, 3, 4, 5];
  a.constructor = {
    [Symbol.species]: FakeArray
  };
  var b = a[funcName](...args);
  assert.sameValue(b.constructor, FakeArray);

  function FakeArrayWithSpecies(n) {
    this.length = n;
  }
  FakeArrayWithSpecies[Symbol.species] = FakeArrayWithSpecies;
  a = [1, 2, 3, 4, 5];
  a.constructor = FakeArrayWithSpecies;
  b = a[funcName](...args);
  assert.sameValue(b.constructor, FakeArrayWithSpecies);

  function FakeArrayWithHook(n) {
    return new Proxy(new FakeArray(n), {
      set(that, name, value) {
        logs += "set:" + name + ":" + value + ",";
        return true;
      },
      defineProperty(that, name, desc) {
        logs += "define:" + name + ":" + desc.value + ":" + desc.configurable + ":" + desc.enumerable + ":" + desc.writable + ",";
        return true;
      }
    });
  }
  var logs = "";
  var ctorProxy = new Proxy({}, {
    get(that, name) {
      logs += "c-get:" + name.toString() + ",";
      if (name == Symbol.species)
        return FakeArrayWithHook;

      return undefined;
    }
  });
  a = new Proxy([1, 2, 3, 4, 5], {
    get(that, name) {
      logs += "get:" + name.toString() + ",";
      if (name == "constructor")
        return ctorProxy;
      return that[name];
    }
  });
  b = a[funcName](...args);
  assert.sameValue(b.constructor, FakeArray);
  assert.sameValue(Object.keys(b).sort().join(","), "length");
  assert.sameValue(b.length, expectedLength);
  assert.sameValue(logs, expectedLogs);

  // no @@species
  a = [1, 2, 3, 4, 5];
  a.constructor = FakeArray;
  b = a[funcName](...args);
  assert.sameValue(b.constructor, Array);

  a = [1, 2, 3, 4, 5];
  a.constructor = {
    [Symbol.species]: undefined
  };
  b = a[funcName](...args);
  assert.sameValue(b.constructor, Array);

  a = [1, 2, 3, 4, 5];
  a.constructor = {
    [Symbol.species]: null
  };
  b = a[funcName](...args);
  assert.sameValue(b.constructor, Array);

  // invalid @@species
  for (var species of [0, 1.1, true, false, "a", /a/, Symbol.iterator, [], {}]) {
    a = [1, 2, 3, 4, 5];
    a.constructor = {
      [Symbol.species]: species
    };
    assert.throws(TypeError, () => a[funcName](...args));
  }

  // undefined constructor
  a = [1, 2, 3, 4, 5];
  a.constructor = undefined;
  b = a[funcName](...args);
  assert.sameValue(b.constructor, Array);

  // invalid constructor
  for (var ctor of [null, 0, 1.1, true, false, "a", Symbol.iterator]) {
    a = [1, 2, 3, 4, 5];
    a.constructor = ctor;
    assert.throws(TypeError, () => a[funcName](...args));
  }

  // not an array
  a = new Proxy({
    0: 1, 1: 2, 2: 3, 3: 4, 4: 5,
    length: 5,
    [funcName]: Array.prototype[funcName]
  }, {
    get(that, name) {
      assert.sameValue(name !== "constructor", true);
      return that[name];
    }
  });
  b = a[funcName](...args);
  assert.sameValue(b.constructor, Array);

  // @@species from different global
  g.eval("function FakeArray(n) { this.length = n; }");
  a = [1, 2, 3, 4, 5];
  a.constructor = {
    [Symbol.species]: g.FakeArray
  };
  b = a[funcName](...args);
  assert.sameValue(b.constructor, g.FakeArray);

  a = [1, 2, 3, 4, 5];
  a.constructor = {
    [Symbol.species]: g.Array
  };
  b = a[funcName](...args);
  assert.sameValue(b.constructor, g.Array);

  // constructor from different global
  g.eval("function FakeArrayWithSpecies(n) { this.length = n; }");
  g.eval("FakeArrayWithSpecies[Symbol.species] = FakeArrayWithSpecies;");
  a = [1, 2, 3, 4, 5];
  a.constructor = g.FakeArrayWithSpecies;
  b = a[funcName](...args);
  assert.sameValue(b.constructor, g.FakeArrayWithSpecies);

  g.eval("var a = [1, 2, 3, 4, 5];");
  b = Array.prototype[funcName].apply(g.a, args);
  assert.sameValue(b.constructor, Array);

  // running in different global
  b = g.a[funcName](...args);
  assert.sameValue(b.constructor, g.Array);

  // subclasses
  // not-modified @@species
  eval(`
class ${funcName}Class extends Array {
}
a = new ${funcName}Class(1, 2, 3, 4, 5);
b = a[funcName](...args);
assert.sameValue(b.constructor, ${funcName}Class);
`);

  // modified @@species
  eval(`
class ${funcName}Class2 extends Array {
  static get [Symbol.species]() {
    return Date;
  }
}
a = new ${funcName}Class2(1, 2, 3, 4, 5);
b = a[funcName](...args);
assert.sameValue(b.constructor, Date);
`);
}

test("concat", [], 0, "get:concat,get:constructor,c-get:Symbol(Symbol.species),get:Symbol(Symbol.isConcatSpreadable),get:length,get:0,define:0:1:true:true:true,get:1,define:1:2:true:true:true,get:2,define:2:3:true:true:true,get:3,define:3:4:true:true:true,get:4,define:4:5:true:true:true,set:length:5,");
test("filter", [x => x % 2], 0, "get:filter,get:length,get:constructor,c-get:Symbol(Symbol.species),get:0,define:0:1:true:true:true,get:1,get:2,define:1:3:true:true:true,get:3,get:4,define:2:5:true:true:true,");
test("map", [x => x * 2], 5, "get:map,get:length,get:constructor,c-get:Symbol(Symbol.species),get:0,define:0:2:true:true:true,get:1,define:1:4:true:true:true,get:2,define:2:6:true:true:true,get:3,define:3:8:true:true:true,get:4,define:4:10:true:true:true,");
test("slice", [], 5, "get:slice,get:length,get:constructor,c-get:Symbol(Symbol.species),get:0,define:0:1:true:true:true,get:1,define:1:2:true:true:true,get:2,define:2:3:true:true:true,get:3,define:3:4:true:true:true,get:4,define:4:5:true:true:true,set:length:5,");
test("splice", [0, 5], 5, "get:splice,get:length,get:constructor,c-get:Symbol(Symbol.species),get:0,define:0:1:true:true:true,get:1,define:1:2:true:true:true,get:2,define:2:3:true:true:true,get:3,define:3:4:true:true:true,get:4,define:4:5:true:true:true,set:length:5,");
