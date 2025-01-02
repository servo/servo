'use strict';

// Helper assertion functions to validate dictionary fields
// on dictionary objects returned from APIs

function assert_unsigned_int_field(object, field) {
  const num = object[field];
  assert_true(Number.isInteger(num) && (num >= 0),
    `Expect dictionary.${field} to be unsigned integer`);
}

function assert_int_field(object, field) {
  const num = object[field];
  assert_true(Number.isInteger(num),
    `Expect dictionary.${field} to be integer`);
}

function assert_string_field(object, field) {
  const str = object[field];
  assert_equals(typeof str, 'string',
    `Expect dictionary.${field} to be string`);
}

function assert_number_field(object, field) {
  const num = object[field];
  assert_equals(typeof num, 'number',
    `Expect dictionary.${field} to be number`);
}

function assert_boolean_field(object, field) {
  const bool = object[field];
  assert_equals(typeof bool, 'boolean',
    `Expect dictionary.${field} to be boolean`);
}

function assert_array_field(object, field) {
  assert_true(Array.isArray(object[field]),
    `Expect dictionary.${field} to be array`);
}

function assert_dict_field(object, field) {
  assert_equals(typeof object[field], 'object',
    `Expect dictionary.${field} to be plain object`);

  assert_not_equals(object[field], null,
    `Expect dictionary.${field} to not be null`);
}

function assert_enum_field(object, field, validValues) {
  assert_string_field(object, field);
  assert_true(validValues.includes(object[field]),
    `Expect dictionary.${field} to have one of the valid enum values: ${validValues}`);
}

function assert_optional_unsigned_int_field(object, field) {
  if(object[field] !== undefined) {
    assert_unsigned_int_field(object, field);
  }
}

function assert_optional_int_field(object, field) {
  if(object[field] !== undefined) {
    assert_int_field(object, field);
  }
}

function assert_optional_string_field(object, field) {
  if(object[field] !== undefined) {
    assert_string_field(object, field);
  }
}

function assert_optional_number_field(object, field) {
  if(object[field] !== undefined) {
    assert_number_field(object, field);
  }
}

function assert_optional_boolean_field(object, field) {
  if(object[field] !== undefined) {
    assert_boolean_field(object, field);
  }
}

function assert_optional_array_field(object, field) {
  if(object[field] !== undefined) {
    assert_array_field(object, field);
  }
}

function assert_optional_dict_field(object, field) {
  if(object[field] !== undefined) {
    assert_dict_field(object, field);
  }
}

function assert_optional_enum_field(object, field, validValues) {
  if(object[field] !== undefined) {
    assert_enum_field(object, field, validValues);
  }
}
