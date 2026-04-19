/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
/* These tests are not checking whether an exception is thrown or not for
 * proxies:  those tests should already exist in js/src/tests/non262/Proxy .
 * We expect TypeErrors to be thrown in these tests, with a stringification
 * of the error message showing whatever property name the error is being
 * reported for.
 *
 * Beyond the presence of the property name, these tests do not care about the
 * contents of the message.
 *
 * The reason for requiring the property name is simple:  with ECMAScript
 * proxies, it can be really hard to figure out what little assertion causes a
 * TypeError in the first place.
 */

const STR = "one", STR_NAME = `"one"`;
const SYM = Symbol("two"), SYM_NAME = `'Symbol("two")'`;

function errorHasPropertyTests(test) {
  assert.throws(TypeError, () => test(STR));
  assert.throws(TypeError, () => test(SYM));
}

function errorHasPropertyTestsWithDetails(test) {
  let [throwable, details] = test(STR);
  assert.throws(TypeError, throwable, details);

  [throwable, details] = test(SYM);
  assert.throws(TypeError, throwable, details);
}

// getOwnPropertyDescriptor

function testGetOwnPropertyDescriptor_OBJORUNDEF(propName) {
  // JSMSG_PROXY_GETOWN_OBJORUNDEF
  const h = {
    getOwnPropertyDescriptor: () => 2
  };

  const t = {};
  const p = new Proxy(t, h);

  Reflect.getOwnPropertyDescriptor(p, propName);
}

function testGetOwnPropertyDescriptor_NC_AS_NE(propName) {
  // JSMSG_CANT_REPORT_NC_AS_NE
  const h = {
    getOwnPropertyDescriptor: () => undefined
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: false
  });
  const p = new Proxy(t, h);

  Reflect.getOwnPropertyDescriptor(p, propName);
}

function testGetOwnPropertyDescriptor_E_AS_NE(propName) {
  // JSMSG_CANT_REPORT_E_AS_NE
  const h = {
    getOwnPropertyDescriptor: () => undefined,
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: true
  });
  Reflect.preventExtensions(t);
  const p = new Proxy(t, h);

  Reflect.getOwnPropertyDescriptor(p, propName);
}

function testGetOwnPropertyDescriptor_NE_AS_NC(propName) {
  // JSMSG_CANT_REPORT_NE_AS_NC
  const h = {
    getOwnPropertyDescriptor: function() {
      return {
        value: 1,
        writable: true,
        enumerable: true,
        configurable: false
      };
    }
  };

  const t = {};
  const p = new Proxy(t, h);

  Reflect.getOwnPropertyDescriptor(p, propName);
}

function testGetOwnPropertyDescriptor_C_AS_NC(propName) {
  // JSMSG_CANT_REPORT_C_AS_NC
  const h = {
    getOwnPropertyDescriptor: function() {
      return {
        value: 1,
        writable: true,
        enumerable: true,
        configurable: false // here's the difference
      };
    }
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: true
  });

  const p = new Proxy(t, h);

  Reflect.getOwnPropertyDescriptor(p, propName);
}

function testGetOwnPropertyDescriptor_INVALID_NOT_EXTENSIBLE(propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_NOT_EXTENSIBLE
  const h = {
    getOwnPropertyDescriptor: function() {
      return {
        value: 1,
        writable: true,
        enumerable: true,
        configurable: true
      };
    }
  };

  const t = {};
  Reflect.preventExtensions(t);

  const p = new Proxy(t, h);
  return [() => { Reflect.getOwnPropertyDescriptor(p, propName); },
          "proxy can't report an extensible object as non-extensible"];
}

function testGetOwnPropertyDescriptor_INVALID_C_AS_NC(propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_CANT_REPORT_NC_AS_C
  const h = {
    getOwnPropertyDescriptor: function() {
      return {
        value: 1,
        writable: true,
        enumerable: true,
        configurable: true
      };
    }
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: false
  });

  const p = new Proxy(t, h);
  return [() => { Reflect.getOwnPropertyDescriptor(p, propName); },
          "proxy can't report an existing non-configurable property as configurable"];
}

function testGetOwnPropertyDescriptor_INVALID_ENUM_DIFFERENT_CURRENT(cEnumerable, propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_ENUM_DIFFERENT
  const h = {
    getOwnPropertyDescriptor: function() {
      return {
        value: 1,
        writable: true,
        enumerable: !cEnumerable,
        configurable: false
      };
    }
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 1,
    writable: true,
    enumerable: cEnumerable,
    configurable: false
  });

  const p = new Proxy(t, h);
  return [() => { Reflect.getOwnPropertyDescriptor(p, propName); },
          "proxy can't report a different 'enumerable' from target when target is not configurable"];
}

function testGetOwnPropertyDescriptor_INVALID_CURRENT_NC_DIFF_TYPE(cAccessor, propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_CURRENT_NC_DIFF_TYPE
  const accDesc = {
    get: () => 1,
    enumerable: true,
    configurable: false,
  };
  const dataDesc = {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: false
  };

  const h = {
    getOwnPropertyDescriptor: () => { return (cAccessor ? dataDesc : accDesc); }
  };

  const t = {};
  Reflect.defineProperty(t, propName, cAccessor ? accDesc : dataDesc);
  const p = new Proxy(t, h);
  return [() => { Reflect.getOwnPropertyDescriptor(p, propName); },
          "proxy can't report a different descriptor type when target is not configurable"];
}

function testGetOwnPropertyDescriptor_INVALID_NW_AS_W(propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_CANT_REPORT_NW_AS_W
  const h = {
    getOwnPropertyDescriptor: function() {
      return {
        value: 1,
        writable: true,
        enumerable: true,
        configurable: false
      };
    }
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 2,
    writable: false,
    enumerable: true,
    configurable: false
  });
  const p = new Proxy(t, h);

  return [() => { Reflect.getOwnPropertyDescriptor(p, propName); },
          "proxy can't report a non-configurable, non-writable property as writable"];
}

function testGetOwnPropertyDescriptor_INVALID_DIFFERENT_VALUE(propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_DIFFERENT_VALUE
  const h = {
    getOwnPropertyDescriptor: function() {
      return {
        value: 1,
        writable: false,
        enumerable: true,
        configurable: false
      };
    }
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 2,
    writable: false,
    enumerable: true,
    configurable: false
  });
  const p = new Proxy(t, h);

  return [() => { Reflect.getOwnPropertyDescriptor(p, propName); },
          "proxy must report the same value for the non-writable, non-configurable property"];
}

function testGetOwnPropertyDescriptor_INVALID_SETTERS_DIFFERENT(propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_SETTERS_DIFFERENT
  const g = () => 1;
  const h = {
    getOwnPropertyDescriptor: function() {
      return {
        get: g,
        set: () => 2,
        enumerable: true,
        configurable: false
      };
    }
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    get: g,
    set: () => 2,
    enumerable: true,
    configurable: false
  });
  const p = new Proxy(t, h);

  return [() => { Reflect.getOwnPropertyDescriptor(p, propName); },
          "proxy can't report different setters for a currently non-configurable property"];
}

function testGetOwnPropertyDescriptor_INVALID_GETTERS_DIFFERENT(propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_GETTERS_DIFFERENT
  const h = {
    getOwnPropertyDescriptor: function() {
      return {
        get: () => 2,
        enumerable: true,
        configurable: false
      };
    }
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    get: () => 2,
    enumerable: true,
    configurable: false
  });
  const p = new Proxy(t, h);

  return [() => { Reflect.getOwnPropertyDescriptor(p, propName); },
          "proxy can't report different getters for a currently non-configurable property"];
}

// defineProperty
function testDefineProperty_CANT_DEFINE_NEW(propName) {
  // JSMSG_CANT_DEFINE_NEW
  const h = {
    defineProperty: () => true
  };

  const t = {};
  Reflect.preventExtensions(t);

  const p = new Proxy(t, h);
  Reflect.defineProperty(p, propName, {});
}

function testDefineProperty_NE_AS_NC(propName) {
  // JSMSG_CANT_DEFINE_NE_AS_NC
  const h = {
    defineProperty: () => true
  };

  const t = {};

  const p = new Proxy(t, h);
  Reflect.defineProperty(p, propName, {
    value: 1,
    enumerable: true,
    writable: true,
    configurable: false,
  });
}

/* Reflect.defineProperty(proxy, propName, desc) cannot throw
 * JSMSG_CANT_REPORT_INVALID with DETAILS_NOT_EXTENSIBLE.  Here's why:
 *
 * To throw with DETAILS_NOT_EXTENSIBLE, current must be undefined and the
 * target must not be extensible, inside ValidateAndApplyPropertyDescriptor.
 *
 * ValidateAndApplyPropertyDescriptor's current is also
 * IsCompatiblePropertyDescriptor's current, and therefore also
 * targetDesc in [[DefineOwnProperty]] for proxies at step 16b.
 *
 * BUT step 16 is not reached if targetDesc in [[DefineOwnProperty]] is
 * undefined:  instead step 15 is invoked.  QED.
 */

function testDefineProperty_INVALID_NC_AS_C(propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_CANT_REPORT_NC_AS_C
  const h = {
    defineProperty: function() {
      return true;
    }
  };

  const newDesc = {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: true
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: false
  });

  const p = new Proxy(t, h);
  return [() => { Reflect.defineProperty(p, propName, newDesc); },
          "proxy can't report an existing non-configurable property as configurable"];
}

function testDefineProperty_INVALID_ENUM_DIFFERENT_CURRENT(cEnumerable, propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_ENUM_DIFFERENT
  const h = {
    defineProperty: function() {
      return true;
    }
  };

  const newDesc = {
    value: 1,
    writable: true,
    enumerable: !cEnumerable,
    configurable: false
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 1,
    writable: true,
    enumerable: cEnumerable,
    configurable: false
  });

  const p = new Proxy(t, h);
  return [() => { Reflect.defineProperty(p, propName, newDesc); },
          "proxy can't report a different 'enumerable' from target when target is not configurable"];
}

function testDefineProperty_INVALID_CURRENT_NC_DIFF_TYPE(cAccessor, propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_CURRENT_NC_DIFF_TYPE
  const accDesc = {
    get: () => 1,
    enumerable: true,
    configurable: false,
  };
  const dataDesc = {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: false
  };

  const h = {
    defineProperty: () => true,
  };

  const t = {};
  Reflect.defineProperty(t, propName, cAccessor ? accDesc : dataDesc);
  const p = new Proxy(t, h);
  return [() => { Reflect.defineProperty(p, propName, cAccessor ? dataDesc : accDesc); },
          "proxy can't report a different descriptor type when target is not configurable"];
}

function testDefineProperty_INVALID_NW_AS_W(propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_CANT_REPORT_NW_AS_W
  const h = {
    defineProperty: function() {
      return true;
    }
  };

  const newDesc = {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: false
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 2,
    writable: false,
    enumerable: true,
    configurable: false
  });
  const p = new Proxy(t, h);

  return [() => { Reflect.defineProperty(p, propName, newDesc); },
          "proxy can't report a non-configurable, non-writable property as writable"];
}

function testDefineProperty_INVALID_DIFFERENT_VALUE(propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_DIFFERENT_VALUE
  const h = {
    defineProperty: function() {
      return true;
    }
  };

  const newDesc = {
    value: 1,
    writable: false,
    enumerable: true,
    configurable: false
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 2,
    writable: false,
    enumerable: true,
    configurable: false
  });
  const p = new Proxy(t, h);

  return [() => { Reflect.defineProperty(p, propName, newDesc); },
          "proxy must report the same value for the non-writable, non-configurable property"];
}

function testDefineProperty_INVALID_SETTERS_DIFFERENT(propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_SETTERS_DIFFERENT
  const g = () => 1;
  const h = {
    defineProperty: function() {
      return true;
    }
  };

  const newDesc = {
    get: g,
    set: () => 2,
    enumerable: true,
    configurable: false
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    get: g,
    set: () => 2,
    enumerable: true,
    configurable: false
  });
  const p = new Proxy(t, h);

  return [() => { Reflect.defineProperty(p, propName, newDesc); },
          "proxy can't report different setters for a currently non-configurable property"];
}

function testDefineProperty_INVALID_GETTERS_DIFFERENT(propName) {
  // JSMSG_CANT_REPORT_INVALID, DETAILS_GETTERS_DIFFERENT
  const h = {
    defineProperty: function() {
      return true;
    }
  };

  const newDesc = {
    get: () => 2,
    enumerable: true,
    configurable: false
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    get: () => 2,
    enumerable: true,
    configurable: false
  });
  const p = new Proxy(t, h);

  return [() => { Reflect.defineProperty(p, propName, newDesc); },
          "proxy can't report different getters for a currently non-configurable property"];
}

function testDefineProperty_INVALID_C_AS_NC(propName) {
  const h = {
    defineProperty: function() {
      return true;
    }
  };

  const newDesc = {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: false
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: true
  });
  const p = new Proxy(t, h);

  return [() => { Reflect.defineProperty(p, propName, newDesc); },
          "proxy can't define an existing configurable property as non-configurable"];
}

// ownKeys

function testOwnKeys_CANT_SKIP_NC(propName) {
  // JSMSG_CANT_SKIP_NC
  const h = {
    ownKeys: () => []
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: false
  });

  const p = new Proxy(t, h);

  Reflect.ownKeys(p);
}

function testOwnKeys_E_AS_NE(propName) {
  // JSMSG_CANT_REPORT_E_AS_NE
  const h = {
    ownKeys: () => []
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    configurable: true,
    value: 1,
    writable: true,
    enumerable: true
  });

  Reflect.preventExtensions(t);
  const p = new Proxy(t, h);

  Reflect.ownKeys(p);
}

// has

function testHas_NC_AS_NE(propName) {
  // JSMSG_CANT_REPORT_NC_AS_NE
  const h = {
    has: () => undefined
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: false
  });
  const p = new Proxy(t, h);

  Reflect.has(p, propName);
}

function testHas_E_AS_NE(propName) {
  // JSMSG_CANT_REPORT_E_AS_NE
  const h = {
    has: () => undefined
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 1,
    writable: true,
    enumerable: true,
    configurable: true
  });
  Reflect.preventExtensions(t);
  const p = new Proxy(t, h);

  Reflect.has(p, propName);
}

// get

function testGet_SAME_VALUE(propName) {
  const h = {
    get: () => 2
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 1,
    writable: false,
    enumerable: true,
    configurable: false
  });

  const p = new Proxy(t, h);
  Reflect.get(p, propName);
}

function testGet_MUST_REPORT_UNDEFINED(propName) {
  // JSMSG_MUST_REPORT_UNDEFINED
  const h = {
    get: () => 2
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    set: () => { /* do nothing */},
    enumerable: true,
    configurable: false
  });

  const p = new Proxy(t, h);
  Reflect.get(p, propName);
}

// set

function testSet_CANT_SET_NW_NC(propName) {
  // JSMSG_CANT_SET_NW_NC
  const h = {
    set: () => true,
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    value: 1,
    writable: false,
    enumerable: true,
    configurable: false
  });

  const p = new Proxy(t, h);
  Reflect.set(p, propName, 3);
}

function testSet_WO_SETTER(propName) {
  // JSMSG_MUST_REPORT_UNDEFINED
  const h = {
    set: () => true
  };

  const t = {};
  Reflect.defineProperty(t, propName, {
    get: () => { /* do nothing */},
    enumerable: true,
    configurable: false
  });

  const p = new Proxy(t, h);
  Reflect.set(p, propName, 1);
}

// test sequence

[
  testGetOwnPropertyDescriptor_OBJORUNDEF,
  testGetOwnPropertyDescriptor_NC_AS_NE,
  testGetOwnPropertyDescriptor_E_AS_NE,
  testGetOwnPropertyDescriptor_NE_AS_NC,
  testGetOwnPropertyDescriptor_C_AS_NC,

  testDefineProperty_CANT_DEFINE_NEW,
  testDefineProperty_NE_AS_NC,

  testOwnKeys_CANT_SKIP_NC,
  testOwnKeys_E_AS_NE,

  testHas_NC_AS_NE,
  testHas_E_AS_NE,

  testGet_SAME_VALUE,
  testGet_MUST_REPORT_UNDEFINED,

  testSet_CANT_SET_NW_NC,
  testSet_WO_SETTER,
].forEach(errorHasPropertyTests);

[
  testGetOwnPropertyDescriptor_INVALID_NOT_EXTENSIBLE,
  testGetOwnPropertyDescriptor_INVALID_C_AS_NC,
  testGetOwnPropertyDescriptor_INVALID_ENUM_DIFFERENT_CURRENT.bind(null, true),
  testGetOwnPropertyDescriptor_INVALID_ENUM_DIFFERENT_CURRENT.bind(null, false),
  testGetOwnPropertyDescriptor_INVALID_CURRENT_NC_DIFF_TYPE.bind(null, true),
  testGetOwnPropertyDescriptor_INVALID_CURRENT_NC_DIFF_TYPE.bind(null, false),
  testGetOwnPropertyDescriptor_INVALID_NW_AS_W,
  testGetOwnPropertyDescriptor_INVALID_DIFFERENT_VALUE,
  testGetOwnPropertyDescriptor_INVALID_SETTERS_DIFFERENT,
  testGetOwnPropertyDescriptor_INVALID_GETTERS_DIFFERENT,

  testDefineProperty_INVALID_NC_AS_C,
  testDefineProperty_INVALID_ENUM_DIFFERENT_CURRENT.bind(null, true),
  testDefineProperty_INVALID_ENUM_DIFFERENT_CURRENT.bind(null, false),
  testDefineProperty_INVALID_CURRENT_NC_DIFF_TYPE.bind(null, true),
  testDefineProperty_INVALID_CURRENT_NC_DIFF_TYPE.bind(null, false),
  testDefineProperty_INVALID_NW_AS_W,
  testDefineProperty_INVALID_DIFFERENT_VALUE,
  testDefineProperty_INVALID_SETTERS_DIFFERENT,
  testDefineProperty_INVALID_GETTERS_DIFFERENT,
  testDefineProperty_INVALID_C_AS_NC,
].forEach(errorHasPropertyTestsWithDetails);

