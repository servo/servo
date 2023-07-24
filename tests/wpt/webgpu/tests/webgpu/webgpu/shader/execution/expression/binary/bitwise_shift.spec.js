/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for the bitwise shift binary expression operations
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { i32, scalarType, TypeU32, u32 } from '../../../../util/conversion.js';
import { allInputSources, run } from '../expression.js';

import { binary, compoundBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

function is_unsiged(type) {
  return type === 'u32';
}

const bitwidth = 32;

// Returns true if e1 << e2 is valid for const evaluation
function is_valid_const_shift_left(e1, e1Type, e2) {
  // Shift by 0 is always valid
  if (e2 === 0) {
    return true;
  }

  // Cannot shift by bitwidth or greater
  if (e2 >= bitwidth) {
    return false;
  }

  if (is_unsiged(e1Type)) {
    // If T is an unsigned integer type, and any of the e2 most significant bits of e1 are 1, then invalid.
    const must_be_zero_msb = e2;
    const mask = ~0 << (bitwidth - must_be_zero_msb);
    if ((e1 & mask) !== 0) {
      return false;
    }
  } else {
    // If T is a signed integer type, and the e2+1 most significant bits of e1 do
    // not have the same bit value, then error.
    const must_match_msb = e2 + 1;
    const mask = ~0 << (bitwidth - must_match_msb);
    if ((e1 & mask) !== 0 && (e1 & mask) !== mask) {
      return false;
    }
  }
  return true;
}

// Returns true if e1 >> e2 is valid for const evaluation
function is_valid_const_shift_right(e1, e1Type, e2) {
  // Shift by 0 is always valid
  if (e2 === 0) {
    return true;
  }

  // Cannot shift by bitwidth or greater
  if (e2 >= bitwidth) {
    return false;
  }

  return true;
}

// Returns all cases of shifting e1 left by [0,63]. If `is_const` is true, cases that are
// invalid for const eval are not returned.
function generate_shift_left_cases(e1, e1Type, is_const) {
  const V = e1Type === 'i32' ? i32 : u32;
  const cases = [];
  for (let shift = 0; shift < 64; ++shift) {
    const e2 = shift;
    if (is_const && !is_valid_const_shift_left(e1, e1Type, e2)) {
      continue;
    }
    const expected = e1 << e2 % bitwidth;
    cases.push({ input: [V(e1), u32(e2)], expected: V(expected) });
  }
  return cases;
}

// Returns all cases of shifting e1 right by [0,63]. If `is_const` is true, cases that are
// invalid for const eval are not returned.
function generate_shift_right_cases(e1, e1Type, is_const) {
  const V = e1Type === 'i32' ? i32 : u32;
  const cases = [];
  for (let shift = 0; shift < 64; ++shift) {
    const e2 = shift;
    if (is_const && !is_valid_const_shift_right(e1, e1Type, e2)) {
      continue;
    }

    let expected = 0;
    if (is_unsiged(e1Type)) {
      // zero-fill right shift
      expected = e1 >>> e2;
    } else {
      // arithmetic right shift
      expected = e1 >> e2;
    }
    cases.push({ input: [V(e1), u32(e2)], expected: V(expected) });
  }
  return cases;
}

function makeShiftLeftConcreteCases(inputType, inputSource, type) {
  const V = inputType === 'i32' ? i32 : u32;
  const is_const = inputSource === 'const';

  const cases = [
    {
      input: /*  */ [V(0b00000000000000000000000000000001), u32(1)],
      expected: /**/ V(0b00000000000000000000000000000010),
    },
    {
      input: /*  */ [V(0b00000000000000000000000000000011), u32(1)],
      expected: /**/ V(0b00000000000000000000000000000110),
    },
  ];

  const add_unsigned_overflow_cases = !is_const || is_unsiged(inputType);
  const add_signed_overflow_cases = !is_const || !is_unsiged(inputType);

  if (add_unsigned_overflow_cases) {
    // Cases that are fine for unsigned values, but would overflow (sign change) signed
    // values when const evaluated.
    cases.push(
      ...[
        {
          input: [/*  */ V(0b01000000000000000000000000000000), u32(1)],
          expected: /**/ V(0b10000000000000000000000000000000),
        },
        {
          input: [/*  */ V(0b01111111111111111111111111111111), u32(1)],
          expected: /**/ V(0b11111111111111111111111111111110),
        },
        {
          input: [/*  */ V(0b00000000000000000000000000000001), u32(31)],
          expected: /**/ V(0b10000000000000000000000000000000),
        },
      ]
    );
  }
  if (add_signed_overflow_cases) {
    // Cases that are fine for signed values (no sign change), but would overflow
    // unsigned values when const evaluated.
    cases.push(
      ...[
        {
          input: [/*  */ V(0b11000000000000000000000000000000), u32(1)],
          expected: /**/ V(0b10000000000000000000000000000000),
        },
        {
          input: [/*  */ V(0b11111111111111111111111111111111), u32(1)],
          expected: /**/ V(0b11111111111111111111111111111110),
        },
        {
          input: [/*  */ V(0b11111111111111111111111111111111), u32(31)],
          expected: /**/ V(0b10000000000000000000000000000000),
        },
      ]
    );
  }

  // Generate cases that shift input value by [0,63] (invalid const eval cases are not returned).
  cases.push(...generate_shift_left_cases(0b00000000000000000000000000000000, inputType, is_const));
  cases.push(...generate_shift_left_cases(0b00000000000000000000000000000001, inputType, is_const));
  cases.push(...generate_shift_left_cases(0b00000000000000000000000000000010, inputType, is_const));
  cases.push(...generate_shift_left_cases(0b00000000000000000000000000000011, inputType, is_const));
  cases.push(...generate_shift_left_cases(0b10000000000000000000000000000000, inputType, is_const));
  cases.push(...generate_shift_left_cases(0b01000000000000000000000000000000, inputType, is_const));
  cases.push(...generate_shift_left_cases(0b11000000000000000000000000000000, inputType, is_const));
  cases.push(...generate_shift_left_cases(0b00010000001000001000010001010101, inputType, is_const));
  cases.push(...generate_shift_left_cases(0b11101111110111110111101110101010, inputType, is_const));
  return cases;
}

g.test('shift_left_concrete')
  .specURL('https://www.w3.org/TR/WGSL/#bit-expr')
  .desc(
    `
e1 << e2

Shift left (shifted value is concrete)
`
  )
  .params(u =>
    u
      .combine('type', ['i32', 'u32'])
      .combine('inputSource', allInputSources)
      .combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(async t => {
    const type = scalarType(t.params.type);
    const cases = makeShiftLeftConcreteCases(t.params.type, t.params.inputSource, type);
    await run(t, binary('<<'), [type, TypeU32], type, t.params, cases);
  });

g.test('shift_left_concrete_compound')
  .specURL('https://www.w3.org/TR/WGSL/#bit-expr')
  .desc(
    `
e1 <<= e2

Shift left (shifted value is concrete)
`
  )
  .params(u =>
    u
      .combine('type', ['i32', 'u32'])
      .combine('inputSource', allInputSources)
      .combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(async t => {
    const type = scalarType(t.params.type);
    const cases = makeShiftLeftConcreteCases(t.params.type, t.params.inputSource, type);
    await run(t, compoundBinary('<<='), [type, TypeU32], type, t.params, cases);
  });

function makeShiftRightConcreteCases(inputType, inputSource, type) {
  const V = inputType === 'i32' ? i32 : u32;
  const is_const = inputSource === 'const';

  const cases = [
    {
      input: /*  */ [V(0b00000000000000000000000000000001), u32(1)],
      expected: /**/ V(0b00000000000000000000000000000000),
    },
    {
      input: /*  */ [V(0b00000000000000000000000000000011), u32(1)],
      expected: /**/ V(0b00000000000000000000000000000001),
    },
    {
      input: /*  */ [V(0b01000000000000000000000000000000), u32(1)],
      expected: /**/ V(0b00100000000000000000000000000000),
    },
    {
      input: /*  */ [V(0b01100000000000000000000000000000), u32(1)],
      expected: /**/ V(0b00110000000000000000000000000000),
    },
  ];

  if (is_unsiged(inputType)) {
    // No sign extension
    cases.push(
      ...[
        {
          input: /*  */ [V(0b10000000000000000000000000000000), u32(1)],
          expected: /**/ V(0b01000000000000000000000000000000),
        },
        {
          input: /*  */ [V(0b11000000000000000000000000000000), u32(1)],
          expected: /**/ V(0b01100000000000000000000000000000),
        },
      ]
    );
  } else {
    cases.push(
      // Sign extension if msb is 1
      ...[
        {
          input: /*  */ [V(0b10000000000000000000000000000000), u32(1)],
          expected: /**/ V(0b11000000000000000000000000000000),
        },
        {
          input: /*  */ [V(0b11000000000000000000000000000000), u32(1)],
          expected: /**/ V(0b11100000000000000000000000000000),
        },
      ]
    );
  }

  // Generate cases that shift input value by [0,63] (invalid const eval cases are not returned).
  cases.push(
    ...generate_shift_right_cases(0b00000000000000000000000000000000, inputType, is_const)
  );

  cases.push(
    ...generate_shift_right_cases(0b00000000000000000000000000000001, inputType, is_const)
  );

  cases.push(
    ...generate_shift_right_cases(0b00000000000000000000000000000010, inputType, is_const)
  );

  cases.push(
    ...generate_shift_right_cases(0b00000000000000000000000000000011, inputType, is_const)
  );

  cases.push(
    ...generate_shift_right_cases(0b10000000000000000000000000000000, inputType, is_const)
  );

  cases.push(
    ...generate_shift_right_cases(0b01000000000000000000000000000000, inputType, is_const)
  );

  cases.push(
    ...generate_shift_right_cases(0b11000000000000000000000000000000, inputType, is_const)
  );

  cases.push(
    ...generate_shift_right_cases(0b00010000001000001000010001010101, inputType, is_const)
  );

  cases.push(
    ...generate_shift_right_cases(0b11101111110111110111101110101010, inputType, is_const)
  );

  return cases;
}

g.test('shift_right_concrete')
  .specURL('https://www.w3.org/TR/WGSL/#bit-expr')
  .desc(
    `
e1 >> e2

Shift right (shifted value is concrete)
`
  )
  .params(u =>
    u
      .combine('type', ['i32', 'u32'])
      .combine('inputSource', allInputSources)
      .combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(async t => {
    const type = scalarType(t.params.type);
    const cases = makeShiftRightConcreteCases(t.params.type, t.params.inputSource, type);
    await run(t, binary('>>'), [type, TypeU32], type, t.params, cases);
  });

g.test('shift_right_concrete_compound')
  .specURL('https://www.w3.org/TR/WGSL/#bit-expr')
  .desc(
    `
e1 >>= e2

Shift right (shifted value is concrete)
`
  )
  .params(u =>
    u
      .combine('type', ['i32', 'u32'])
      .combine('inputSource', allInputSources)
      .combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(async t => {
    const type = scalarType(t.params.type);
    const cases = makeShiftRightConcreteCases(t.params.type, t.params.inputSource, type);
    await run(t, compoundBinary('>>='), [type, TypeU32], type, t.params, cases);
  });
