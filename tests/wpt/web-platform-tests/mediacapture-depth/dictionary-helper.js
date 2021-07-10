"use strict";

// Helper assertion functions to validate dictionary fields
// on dictionary objects returned from APIs

function assert_string(object) {
  assert_equals(typeof object, "string",
    `Expect ${object} to be string`);
}

function assert_string_field(object, field) {
  const str = object[field];
  assert_equals(typeof str, "string",
    `Expect dictionary.${field} to be string`);
}

function assert_number_field(object, field) {
  const num = object[field];
  assert_equals(typeof num, "number",
    `Expect dictionary.${field} to be number`);
}

function assert_boolean_field(object, field, value = "") {
  const bool = object[field];
  assert_equals(typeof bool, "boolean",
    `Expect dictionary.${field} to be boolean`);
  if (object[field] !== "") {
    assert_equals(object[field], value,
      `Expect default value of dictionary.${field} to be ${value}`);
  }
}

function assert_array_field(object, field) {
  assert_true(Array.isArray(object[field]),
    `Expect dictionary.${field} to be array`);
}

function assert_enum_field(object, field, validValues) {
  assert_string_field(object, field);
  assert_true(validValues.includes(object[field]),
    `Expect dictionary.${field} to have one of the valid enum values: ${validValues}`);
}

function assert_number_range_field(object, field, key) {
  const num = object[field][key];
  assert_equals(typeof num, "number",
    `Expect dictionary.${field}.${key} to be number`);
}

function assert_boolean_range_field(object, field, key) {
  const bool = object[field][key];
  assert_equals(typeof bool, "boolean",
    `Expect dictionary.${field}.${key} to be boolean`);
}

function assert_number_or_number_range_field(object, field) {
  if (typeof object[field] !== "object") {
    assert_number_field(object, field);
  } else {
    if (object[field]["max"] !== undefined)
      assert_number_range_field(object, field, "max");
    if (object[field]["min"] !== undefined)
      assert_number_range_field(object, field, "min");
    if (object[field]["max"] === undefined &&
        object[field]["min"] === undefined)
      assert_unreached();
  }
}

function assert_constrain_string_field(object, field) {
  // test DOMString type
  if (typeof object[field] !== "object") {
    assert_string_field(object, field);
  // test ConstrainDOMStringParameters type
  } else if (typeof object[field]["exact"] !== undefined || typeof object[field]["ideal"] !== undefined) {
    if (object[field]["exact"] !== undefined) {
      // test DOMString type key value of ConstrainDOMStringParameters dictionary
      if (typeof object[field] !== "object") {
        assert_string_field(object[field], "exact");
      // test sequence<DOMString> type key value of ConstrainDOMStringParameters dictionary
      } else {
        assert_array_field(object[field], "exact");
        for(const item of object[field]["exact"]) {
          assert_string(item);
        }
      }
    }
    if (object[field]["ideal"] !== undefined) {
      // test DOMString type key value of ConstrainDOMStringParameters dictionary
      if (typeof object[field] !== "object") {
        assert_string_field(object[field], "ideal");
      // test sequence<DOMString> type key value of ConstrainDOMStringParameters dictionary
      } else {
        assert_array_field(object[field], "ideal");
        for(const item of object[field]["ideal"]) {
          assert_string(item);
        }
      }
    }
  // test sequence<DOMString> type
  } else {
    assert_array_field(object, field);
    for(const item of object[field]) {
      assert_string(item);
    }
  }
}

function assert_constrain_number_field(object, field) {
  if (typeof object[field] !== "object") {
    assert_number_field(object, field);
  } else {
    if (object[field]["max"] !== undefined)
      assert_number_range_field(object, field, "max");
    if (object[field]["min"] !== undefined)
      assert_number_range_field(object, field, "min");
    if (object[field]["exact"] !== undefined)
      assert_number_range_field(object, field, "exact");
    if (object[field]["ideal"] !== undefined)
      assert_number_range_field(object, field, "ideal");
    if (object[field]["max"] === undefined &&
        object[field]["min"] === undefined &&
        object[field]["exact"] === undefined &&
        object[field]["ideal"] === undefined)
      assert_unreached();
  }
}

function assert_constrain_boolean_field(object, field) {
  if (typeof object[field] !== "object") {
    assert_boolean_field(object, field);
  } else {
    if (object[field]["exact"] !== undefined)
      assert_boolean_range_field(object, field, "exact");
    if (object[field]["ideal"] !== undefined)
      assert_boolean_range_field(object, field, "ideal");
    if (object[field]["exact"] === undefined &&
        object[field]["ideal"] === undefined)
      assert_unreached();
  }
}
