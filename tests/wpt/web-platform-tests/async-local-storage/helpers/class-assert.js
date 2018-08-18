export function isConstructor(o) {
  assert_equals(typeof o, "function", "Must be a function according to typeof");
  assert_true(isConstructorTest(o), "Must be a constructor according to the meta-object protocol");
  assert_throws(new TypeError(), () => o(), "Attempting to call (not construct) must throw");
}

export function functionLength(o, expected, label) {
  const lengthExpected = { writable: false, enumerable: false, configurable: true };
  const { value } = propertyDescriptor(o, "length", lengthExpected);

  assert_equals(value, expected, `${formatLabel(label)}length value`);
}

export function functionName(o, expected, label) {
  const lengthExpected = { writable: false, enumerable: false, configurable: true };
  const { value } = propertyDescriptor(o, "name", lengthExpected);

  assert_equals(value, expected, `${formatLabel(label)}name value`);
}

export function hasClassPrototype(o) {
  const prototypeExpected = { writable: false, enumerable: false, configurable: false };
  const { value } = propertyDescriptor(o, "prototype", prototypeExpected);
  assert_equals(typeof value, "object", "prototype must be an object");
  assert_not_equals(value, null, "prototype must not be null");
}

export function hasPrototypeConstructorLink(klass) {
  const constructorExpected = { writable: true, enumerable: false, configurable: true };
  const { value } = propertyDescriptor(klass.prototype, "constructor", constructorExpected);
  assert_equals(value, klass, "constructor property must match");
}

export function propertyKeys(o, expectedNames, expectedSymbols, label) {
  label = formatLabel(label);
  assert_array_equals(Object.getOwnPropertyNames(o), expectedNames, `${label}property names`);
  assert_array_equals(Object.getOwnPropertySymbols(o), expectedSymbols,
    `${label}property symbols`);
}

export function methods(o, expectedMethods) {
  for (const [name, length] of Object.entries(expectedMethods)) {
    method(o, name, length);
  }
}

export function accessors(o, expectedAccessors) {
  for (const [name, accessorTypes] of Object.entries(expectedAccessors)) {
    accessor(o, name, accessorTypes);
  }
}

function method(o, prop, length) {
  const methodExpected = { writable: true, enumerable: false, configurable: true };
  const { value } = propertyDescriptor(o, prop, methodExpected);

  assert_equals(typeof value, "function", `${prop} method must be a function according to typeof`);
  assert_false(isConstructorTest(value),
    `${prop} method must not be a constructor according to the meta-object protocol`);
  functionLength(value, length, prop);
  functionName(value, prop, prop);
  propertyKeys(value, ["length", "name"], [], prop);
}

function accessor(o, prop, expectedAccessorTypes) {
  const accessorExpected = { enumerable: false, configurable: true };
  const propDesc = propertyDescriptor(o, prop, accessorExpected);

  for (const possibleType of ["get", "set"]) {
    const accessorFunc = propDesc[possibleType];
    if (expectedAccessorTypes.includes(possibleType)) {
      const label = `${prop}'s ${possibleType}ter`;

      assert_equals(typeof accessorFunc, "function",
        `${label} must be a function according to typeof`);
      assert_false(isConstructorTest(accessorFunc),
        `${label} must not be a constructor according to the meta-object protocol`);

      functionLength(accessorFunc, possibleType === "get" ? 0 : 1, label);
      functionName(accessorFunc, `${possibleType} ${prop}`, label);
      propertyKeys(accessorFunc, ["length", "name"], [], label);
    } else {
      assert_equals(accessorFunc, undefined, `${prop} must not have a ${possibleType}ter`);
    }
  }
}

function propertyDescriptor(obj, prop, mustMatch) {
  const propDesc = Object.getOwnPropertyDescriptor(obj, prop);
  for (const key in Object.keys(mustMatch)) {
    assert_equals(propDesc[key], mustMatch[key], `${prop} ${key}`);
  }
  return propDesc;
}

function isConstructorTest(o) {
  try {
    new (new Proxy(o, {construct: () => ({})}));
    return true;
  } catch (e) {
    return false;
  }
}

function formatLabel(label) {
  return label !== undefined ? ` ${label}` : "";
}
