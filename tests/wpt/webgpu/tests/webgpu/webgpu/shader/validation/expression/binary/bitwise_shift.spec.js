/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Validation tests for the bitwise shift binary expression operations
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

// Converts v to signed decimal number.
// Required because JS binary literals are always interpreted as unsigned numbers.
function signed(v) {
  return new Int32Array([v])[0];
}

// Return vector form of size `size` of input value `v`, or `v` if size is undefined.
function vectorize(v, size) {
  if (size !== undefined) {
    return `vec${size}(${v})`;
  }
  return v;
}

const kLeftShiftCases = [
  // rhs >= bitwidth fails
  { lhs: `0u`, rhs: `31u`, pass: true },
  { lhs: `0u`, rhs: `32u`, pass: false },
  { lhs: `0u`, rhs: `33u`, pass: false },
  { lhs: `0u`, rhs: `1000u`, pass: false },
  { lhs: `0u`, rhs: `0xFFFFFFFFu`, pass: false },

  { lhs: `0i`, rhs: `31u`, pass: true },
  { lhs: `0i`, rhs: `32u`, pass: false },
  { lhs: `0i`, rhs: `33u`, pass: false },
  { lhs: `0i`, rhs: `1000u`, pass: false },
  { lhs: `0i`, rhs: `0xFFFFFFFFu`, pass: false },

  // Signed overflow (sign change)
  { lhs: `${0b01000000000000000000000000000000}i`, rhs: `1u`, pass: false },
  { lhs: `${0b01111111111111111111111111111111}i`, rhs: `1u`, pass: false },
  { lhs: `${0b00000000000000000000000000000001}i`, rhs: `31u`, pass: false },
  // Same cases should pass if lhs is unsigned
  { lhs: `${0b01000000000000000000000000000000}u`, rhs: `1u`, pass: true },
  { lhs: `${0b01111111111111111111111111111111}u`, rhs: `1u`, pass: true },
  { lhs: `${0b00000000000000000000000000000001}u`, rhs: `31u`, pass: true },

  // Unsigned overflow
  { lhs: `${0b11000000000000000000000000000000}u`, rhs: `1u`, pass: false },
  { lhs: `${0b11111111111111111111111111111111}u`, rhs: `1u`, pass: false },
  { lhs: `${0b11111111111111111111111111111111}u`, rhs: `31u`, pass: false },
  // Same cases should pass if lhs is signed
  { lhs: `${signed(0b11000000000000000000000000000000)}i`, rhs: `1u`, pass: true },
  { lhs: `${signed(0b11111111111111111111111111111111)}i`, rhs: `1u`, pass: true },
  { lhs: `${signed(0b11111111111111111111111111111111)}i`, rhs: `31u`, pass: true },

  // Shift by negative is an error
  { lhs: `1`, rhs: `-1`, pass: false },
  { lhs: `1i`, rhs: `-1`, pass: false },
  { lhs: `1u`, rhs: `-1`, pass: false },
];

g.test('shift_left_concrete')
  .desc('Tests validation of binary left shift of concrete values')
  .params(u =>
    u
      .combine('case', kLeftShiftCases) //
      .combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(t => {
    const lhs = t.params.case.lhs;
    const rhs = t.params.case.rhs;
    const vec_size = t.params.vectorize;

    const code = `
@compute @workgroup_size(1)
fn main() {
    const r = ${vectorize(lhs, vec_size)} << ${vectorize(rhs, vec_size)};
}
    `;
    t.expectCompileResult(t.params.case.pass, code);
  });

g.test('shift_left_vec_size_mismatch')
  .desc('Tests validation of binary left shift of vectors with mismatched sizes')
  .params(u =>
    u
      .combine('vectorize_lhs', [2, 3, 4]) //
      .combine('vectorize_rhs', [2, 3, 4])
  )
  .fn(t => {
    const lhs = `1`;
    const rhs = `1`;
    const lhs_vec_size = t.params.vectorize_lhs;
    const rhs_vec_size = t.params.vectorize_rhs;
    const code = `
@compute @workgroup_size(1)
fn main() {
    const r = ${vectorize(lhs, lhs_vec_size)} << ${vectorize(rhs, rhs_vec_size)};
}
    `;
    const pass = lhs_vec_size === rhs_vec_size;
    t.expectCompileResult(pass, code);
  });

const kRightShiftCases = [
  // rhs >= bitwidth fails
  { lhs: `0u`, rhs: `31u`, pass: true },
  { lhs: `0u`, rhs: `32u`, pass: false },
  { lhs: `0u`, rhs: `33u`, pass: false },
  { lhs: `0u`, rhs: `1000u`, pass: false },
  { lhs: `0u`, rhs: `0xFFFFFFFFu`, pass: false },

  { lhs: `0i`, rhs: `31u`, pass: true },
  { lhs: `0i`, rhs: `32u`, pass: false },
  { lhs: `0i`, rhs: `33u`, pass: false },
  { lhs: `0i`, rhs: `1000u`, pass: false },
  { lhs: `0i`, rhs: `0xFFFFFFFFu`, pass: false },

  // Shift by negative is an error
  { lhs: `1`, rhs: `-1`, pass: false },
  { lhs: `1i`, rhs: `-1`, pass: false },
  { lhs: `1u`, rhs: `-1`, pass: false },
];

g.test('shift_right_concrete')
  .desc('Tests validation of binary right shift of concrete values')
  .params(u =>
    u
      .combine('case', kRightShiftCases) //
      .combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(t => {
    const lhs = t.params.case.lhs;
    const rhs = t.params.case.rhs;
    const vec_size = t.params.vectorize;

    const code = `
@compute @workgroup_size(1)
fn main() {
    const r = ${vectorize(lhs, vec_size)} >> ${vectorize(rhs, vec_size)};
}
    `;
    t.expectCompileResult(t.params.case.pass, code);
  });

g.test('shift_right_vec_size_mismatch')
  .desc('Tests validation of binary right shift of vectors with mismatched sizes')
  .params(u =>
    u
      .combine('vectorize_lhs', [2, 3, 4]) //
      .combine('vectorize_rhs', [2, 3, 4])
  )
  .fn(t => {
    const lhs = `1`;
    const rhs = `1`;
    const lhs_vec_size = t.params.vectorize_lhs;
    const rhs_vec_size = t.params.vectorize_rhs;
    const code = `
@compute @workgroup_size(1)
fn main() {
    const r = ${vectorize(lhs, lhs_vec_size)} >> ${vectorize(rhs, rhs_vec_size)};
}
    `;
    const pass = lhs_vec_size === rhs_vec_size;
    t.expectCompileResult(pass, code);
  });
