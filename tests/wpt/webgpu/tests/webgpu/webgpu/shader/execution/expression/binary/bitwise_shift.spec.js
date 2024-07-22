/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for the bitwise shift binary expression operations
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { assert } from '../../../../../common/util/util.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type, abstractInt, u32 } from '../../../../util/conversion.js';

import { allInputSources, onlyConstInputSource, run } from '../expression.js';

import { abstractIntBinary, binary, compoundBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

// Returns true if e1 << e2 is valid for const evaluation
function isValidConstShiftLeft(e1, e2) {
  // Shift by 0 is always valid
  if (e2 === 0) {
    return true;
  }

  // Cannot shift by bitwidth or greater.
  const bitwidth = e1.type.size * 8;
  if (e2 >= bitwidth) {
    return false;
  }

  if (!e1.type.signed) {
    // AbstractInt should never enter this branch, so value should always be a number
    assert(typeof e1.value === 'number');
    // If T is an unsigned integer type, and any of the e2 most significant bits of e1 are 1, then invalid.
    const must_be_zero_msb = e2;
    const mask = ~0 << bitwidth - must_be_zero_msb;
    if ((e1.value & mask) !== 0) {
      return false;
    }
  } else {
    // If T is a signed integer type, and the e2+1 most significant bits of e1 do
    // not have the same bit value, then error.
    // Working in bigint, because all i32 and AbstractInt values are representable in it.
    const value = BigInt(e1.value);
    const must_match_msb = BigInt(e2) + 1n;
    const mask = ~0n << BigInt(bitwidth) - must_match_msb;
    if ((value & mask) !== 0n && (value & mask) !== mask) {
      return false;
    }
  }
  return true;
}

// Returns true if e1 >> e2 is valid for const evaluation
function isValidConstShiftRight(e1, e2) {
  // Shift by 0 is always valid
  if (e2 === 0) {
    return true;
  }

  const bitwidth = e1.type.size * 8;
  // Cannot shift by bitwidth or greater
  if (e2 >= bitwidth) {
    return false;
  }
  return true;
}

// Returns all cases of shifting e1 left by [0,63]. If `isConst` is true, cases that are
// invalid for const eval are not returned.
function generateShiftLeftConcreteCases(e1, isConst) {
  assert(typeof e1.value === 'number');

  const bitwidth = e1.type.size * 8;
  const cases = [];
  for (let e2 = 0; e2 < 64; ++e2) {
    if (isConst && !isValidConstShiftLeft(e1, e2)) {
      continue;
    }
    const expected = e1.value << e2 % bitwidth;
    cases.push({ input: [e1, u32(e2)], expected: e1.type.create(expected) });
  }
  return cases;
}

// Returns all cases of shifting e1 left by [0,63]
function generateShiftLeftAbstractCases(e1) {
  assert(typeof e1.value === 'bigint');

  const cases = [];
  for (let e2 = 0; e2 < 64; ++e2) {
    if (!isValidConstShiftLeft(e1, e2)) {
      continue;
    }
    const expected = e1.value << BigInt(e2);
    cases.push({ input: [e1, u32(e2)], expected: abstractInt(expected) });
  }
  return cases;
}

// Returns all cases of shifting e1 right by [0,63]. If `is_const` is true, cases that are
// invalid for const eval are not returned.
function generateShiftRightConcreteCases(e1, isConst) {
  assert(typeof e1.value === 'number');
  const cases = [];
  for (let e2 = 0; e2 < 64; ++e2) {
    if (isConst && !isValidConstShiftRight(e1, e2)) {
      continue;
    }

    let expected = 0;
    if (!e1.type.signed) {
      // zero-fill right shift
      expected = e1.value >>> e2;
    } else {
      // arithmetic right shift
      expected = e1.value >> e2;
    }
    cases.push({ input: [e1, u32(e2)], expected: e1.type.create(expected) });
  }
  return cases;
}

// Returns all cases of shifting e1 right by [0,63], plus a selection of those from [65, 1025]
function generateShiftRightAbstractCases(e1) {
  assert(typeof e1.value === 'bigint');
  const cases = [];
  // Shifting within 64 bits
  for (let e2 = 0; e2 < 64; ++e2) {
    const expected = e1.value >> BigInt(e2);
    cases.push({ input: [e1, u32(e2)], expected: abstractInt(expected) });
  }
  // Always filled values
  for (let e2 = 64; e2 < 1025; e2 *= 2) {
    const expected = e1.value < 0n ? -1n : 0n;
    cases.push({ input: [e1, u32(e2)], expected: abstractInt(expected) });
  }
  return cases;
}

function makeShiftLeftAbstractCases() {
  const cases = [
  {
    input: /*  */[
    abstractInt(0b0000000000000000000000000000000000000000000000000000000000000001n),
    u32(1)],

    expected:
    /**/abstractInt(0b0000000000000000000000000000000000000000000000000000000000000010n)
  },
  {
    input: /*  */[
    abstractInt(0b0000000000000000000000000000000000000000000000000000000000000011n),
    u32(1)],

    expected:
    /**/abstractInt(0b0000000000000000000000000000000000000000000000000000000000000110n)
  },
  // 0 should always return 0
  { input: [abstractInt(0n), u32(0)], expected: abstractInt(0n) },
  { input: [abstractInt(0n), u32(1)], expected: abstractInt(0n) },
  { input: [abstractInt(0n), u32(16)], expected: abstractInt(0n) },
  { input: [abstractInt(0n), u32(32)], expected: abstractInt(0n) },
  { input: [abstractInt(0n), u32(64)], expected: abstractInt(0n) },
  { input: [abstractInt(0n), u32(65)], expected: abstractInt(0n) },
  { input: [abstractInt(0n), u32(128)], expected: abstractInt(0n) },
  { input: [abstractInt(0n), u32(256)], expected: abstractInt(0n) }];


  cases.push(
    ...generateShiftLeftAbstractCases(
      abstractInt(0b0000000000000000000000000000000000000000000000000000000000000000n)
    )
  );
  cases.push(
    ...generateShiftLeftAbstractCases(
      abstractInt(0b0000000000000000000000000000000000000000000000000000000000000001n)
    )
  );
  cases.push(
    ...generateShiftLeftAbstractCases(
      abstractInt(0b0000000000000000000000000000000000000000000000000000000000000010n)
    )
  );
  cases.push(
    ...generateShiftLeftAbstractCases(
      abstractInt(0b0000000000000000000000000000000000000000000000000000000000000011n)
    )
  );
  cases.push(
    ...generateShiftLeftAbstractCases(
      abstractInt(0b1000000000000000000000000000000000000000000000000000000000000000n)
    )
  );
  cases.push(
    ...generateShiftLeftAbstractCases(
      abstractInt(0b0100000000000000000000000000000000000000000000000000000000000000n)
    )
  );
  cases.push(
    ...generateShiftLeftAbstractCases(
      abstractInt(0b1100000000000000000000000000000000000000000000000000000000000000n)
    )
  );
  cases.push(
    ...generateShiftLeftAbstractCases(
      abstractInt(0b0001000000100000100001000101010100010000001000001000010001010101n)
    )
  );
  cases.push(
    ...generateShiftLeftAbstractCases(
      abstractInt(0b1110111111011111011110111010101011101111110111110111101110101010n)
    )
  );
  return cases;
}

g.test('shift_left_abstract').
specURL('https://www.w3.org/TR/WGSL/#bit-expr').
desc(
  `
e1 << e2

Shift left (shifted value is abstract)
`
).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = makeShiftLeftAbstractCases();
  await run(
    t,
    abstractIntBinary('<<'),
    [Type.abstractInt, Type.u32],
    Type.abstractInt,
    t.params,
    cases
  );
});

function makeShiftLeftConcreteCases(
isConst,
isUnsigned,
B)
{
  const cases = [
  {
    input: /*  */[B(0b00000000000000000000000000000001), u32(1)],
    expected: /**/B(0b00000000000000000000000000000010)
  },
  {
    input: /*  */[B(0b00000000000000000000000000000011), u32(1)],
    expected: /**/B(0b00000000000000000000000000000110)
  }];


  const add_unsigned_overflow_cases = !isConst || isUnsigned;
  const add_signed_overflow_cases = !isConst || !isUnsigned;

  if (add_unsigned_overflow_cases) {
    // Cases that are fine for unsigned values, but would overflow (sign change) signed
    // values when const evaluated.
    cases.push(
      ...[
      {
        input: [/*  */B(0b01000000000000000000000000000000), u32(1)],
        expected: /**/B(0b10000000000000000000000000000000)
      },
      {
        input: [/*  */B(0b01111111111111111111111111111111), u32(1)],
        expected: /**/B(0b11111111111111111111111111111110)
      },
      {
        input: [/*  */B(0b00000000000000000000000000000001), u32(31)],
        expected: /**/B(0b10000000000000000000000000000000)
      }]

    );
  }
  if (add_signed_overflow_cases) {
    // Cases that are fine for signed values (no sign change), but would overflow
    // unsigned values when const evaluated.
    cases.push(
      ...[
      {
        input: [/*  */B(0b11000000000000000000000000000000), u32(1)],
        expected: /**/B(0b10000000000000000000000000000000)
      },
      {
        input: [/*  */B(0b11111111111111111111111111111111), u32(1)],
        expected: /**/B(0b11111111111111111111111111111110)
      },
      {
        input: [/*  */B(0b11111111111111111111111111111111), u32(31)],
        expected: /**/B(0b10000000000000000000000000000000)
      }]

    );
  }

  // Generate cases that shift input value by [0,63] (invalid const eval cases are not returned).
  cases.push(...generateShiftLeftConcreteCases(B(0b00000000000000000000000000000000), isConst));
  cases.push(...generateShiftLeftConcreteCases(B(0b00000000000000000000000000000001), isConst));
  cases.push(...generateShiftLeftConcreteCases(B(0b00000000000000000000000000000010), isConst));
  cases.push(...generateShiftLeftConcreteCases(B(0b00000000000000000000000000000011), isConst));
  cases.push(...generateShiftLeftConcreteCases(B(0b10000000000000000000000000000000), isConst));
  cases.push(...generateShiftLeftConcreteCases(B(0b01000000000000000000000000000000), isConst));
  cases.push(...generateShiftLeftConcreteCases(B(0b11000000000000000000000000000000), isConst));
  cases.push(...generateShiftLeftConcreteCases(B(0b00010000001000001000010001010101), isConst));
  cases.push(...generateShiftLeftConcreteCases(B(0b11101111110111110111101110101010), isConst));
  return cases;
}

g.test('shift_left_concrete').
specURL('https://www.w3.org/TR/WGSL/#bit-expr').
desc(
  `
e1 << e2

Shift left (shifted value is concrete)
`
).
params((u) =>
u.
combine('type', ['i32', 'u32']).
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const type = Type[t.params.type];
  const builder = type.create.bind(type);

  const cases = makeShiftLeftConcreteCases(
    t.params.inputSource === 'const',
    !type.signed,
    builder
  );
  await run(t, binary('<<'), [type, Type.u32], type, t.params, cases);
});

g.test('shift_left_concrete_compound').
specURL('https://www.w3.org/TR/WGSL/#bit-expr').
desc(
  `
e1 <<= e2

Shift left (shifted value is concrete)
`
).
params((u) =>
u.
combine('type', ['i32', 'u32']).
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const type = Type[t.params.type];
  const builder = type.create.bind(type);

  const cases = makeShiftLeftConcreteCases(
    t.params.inputSource === 'const',
    !type.signed,
    builder
  );
  await run(t, compoundBinary('<<='), [type, Type.u32], type, t.params, cases);
});

function makeShiftRightAbstractCases() {
  const cases = [
  {
    input: /*  */[
    abstractInt(0b0000000000000000000000000000000000000000000000000000000000000001n),
    u32(1)],

    expected:
    /**/abstractInt(0b0000000000000000000000000000000000000000000000000000000000000000n)
  },
  {
    input: /*  */[
    abstractInt(0b0000000000000000000000000000000000000000000000000000000000000011n),
    u32(1)],

    expected:
    /**/abstractInt(0b0000000000000000000000000000000000000000000000000000000000000001n)
  },
  {
    input: /*  */[
    abstractInt(0b0100000000000000000000000000000000000000000000000000000000000000n),
    u32(1)],

    expected:
    /**/abstractInt(0b0010000000000000000000000000000000000000000000000000000000000000n)
  },
  {
    input: /*  */[
    abstractInt(0b0110000000000000000000000000000000000000000000000000000000000000n),
    u32(1)],

    expected:
    /**/abstractInt(0b0011000000000000000000000000000000000000000000000000000000000000n)
  },
  // Sign extension if msb is 1
  {
    input: /*  */[
    abstractInt(0b1000000000000000000000000000000000000000000000000000000000000000n),
    u32(1)],

    expected:
    /**/abstractInt(0b1100000000000000000000000000000000000000000000000000000000000000n)
  },
  {
    input: /*  */[
    abstractInt(0b1100000000000000000000000000000000000000000000000000000000000000n),
    u32(1)],

    expected:
    /**/abstractInt(0b1110000000000000000000000000000000000000000000000000000000000000n)
  }];


  cases.push(
    ...generateShiftRightAbstractCases(
      abstractInt(0b0000000000000000000000000000000000000000000000000000000000000000n)
    )
  );
  cases.push(
    ...generateShiftRightAbstractCases(
      abstractInt(0b0000000000000000000000000000000000000000000000000000000000000001n)
    )
  );
  cases.push(
    ...generateShiftRightAbstractCases(
      abstractInt(0b0000000000000000000000000000000000000000000000000000000000000010n)
    )
  );
  cases.push(
    ...generateShiftRightAbstractCases(
      abstractInt(0b0000000000000000000000000000000000000000000000000000000000000011n)
    )
  );
  cases.push(
    ...generateShiftRightAbstractCases(
      abstractInt(0b1000000000000000000000000000000000000000000000000000000000000000n)
    )
  );
  cases.push(
    ...generateShiftRightAbstractCases(
      abstractInt(0b0100000000000000000000000000000000000000000000000000000000000000n)
    )
  );
  cases.push(
    ...generateShiftRightAbstractCases(
      abstractInt(0b1100000000000000000000000000000000000000000000000000000000000000n)
    )
  );
  cases.push(
    ...generateShiftRightAbstractCases(
      abstractInt(0b0001000000100000100001000101010100010000001000001000010001010101n)
    )
  );
  cases.push(
    ...generateShiftRightAbstractCases(
      abstractInt(0b1110111111011111011110111010101011101111110111110111101110101010n)
    )
  );
  return cases;
}

g.test('shift_right_abstract').
specURL('https://www.w3.org/TR/WGSL/#bit-expr').
desc(
  `
  e1 >> e2

  Shift right (shifted value is abstract)
  `
).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = makeShiftRightAbstractCases();
  await run(
    t,
    abstractIntBinary('>>'),
    [Type.abstractInt, Type.u32],
    Type.abstractInt,
    t.params,
    cases
  );
});

function makeShiftRightConcreteCases(
isConst,
isUnsigned,
B)
{
  const cases = [
  {
    input: /*  */[B(0b00000000000000000000000000000001), u32(1)],
    expected: /**/B(0b00000000000000000000000000000000)
  },
  {
    input: /*  */[B(0b00000000000000000000000000000011), u32(1)],
    expected: /**/B(0b00000000000000000000000000000001)
  },
  {
    input: /*  */[B(0b01000000000000000000000000000000), u32(1)],
    expected: /**/B(0b00100000000000000000000000000000)
  },
  {
    input: /*  */[B(0b01100000000000000000000000000000), u32(1)],
    expected: /**/B(0b00110000000000000000000000000000)
  }];

  if (isUnsigned) {
    // No sign extension
    cases.push(
      ...[
      {
        input: /*  */[B(0b10000000000000000000000000000000), u32(1)],
        expected: /**/B(0b01000000000000000000000000000000)
      },
      {
        input: /*  */[B(0b11000000000000000000000000000000), u32(1)],
        expected: /**/B(0b01100000000000000000000000000000)
      }]

    );
  } else {
    cases.push(
      // Sign extension if msb is 1
      ...[
      {
        input: /*  */[B(0b10000000000000000000000000000000), u32(1)],
        expected: /**/B(0b11000000000000000000000000000000)
      },
      {
        input: /*  */[B(0b11000000000000000000000000000000), u32(1)],
        expected: /**/B(0b11100000000000000000000000000000)
      }]

    );
  }

  // Generate cases that shift input value by [0,63] (invalid const eval cases are not returned).
  cases.push(...generateShiftRightConcreteCases(B(0b00000000000000000000000000000000), isConst));
  cases.push(...generateShiftRightConcreteCases(B(0b00000000000000000000000000000001), isConst));
  cases.push(...generateShiftRightConcreteCases(B(0b00000000000000000000000000000010), isConst));
  cases.push(...generateShiftRightConcreteCases(B(0b00000000000000000000000000000011), isConst));
  cases.push(...generateShiftRightConcreteCases(B(0b10000000000000000000000000000000), isConst));
  cases.push(...generateShiftRightConcreteCases(B(0b01000000000000000000000000000000), isConst));
  cases.push(...generateShiftRightConcreteCases(B(0b11000000000000000000000000000000), isConst));
  cases.push(...generateShiftRightConcreteCases(B(0b00010000001000001000010001010101), isConst));
  cases.push(...generateShiftRightConcreteCases(B(0b11101111110111110111101110101010), isConst));
  return cases;
}

g.test('shift_right_concrete').
specURL('https://www.w3.org/TR/WGSL/#bit-expr').
desc(
  `
e1 >> e2

Shift right (shifted value is concrete)
`
).
params((u) =>
u.
combine('type', ['i32', 'u32']).
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const type = Type[t.params.type];
  const builder = type.create.bind(type);

  const cases = makeShiftRightConcreteCases(
    t.params.inputSource === 'const',
    !type.signed,
    builder
  );
  await run(t, binary('>>'), [type, Type.u32], type, t.params, cases);
});

g.test('shift_right_concrete_compound').
specURL('https://www.w3.org/TR/WGSL/#bit-expr').
desc(
  `
e1 >>= e2

Shift right (shifted value is concrete)
`
).
params((u) =>
u.
combine('type', ['i32', 'u32']).
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const type = Type[t.params.type];
  const builder = type.create.bind(type);

  const cases = makeShiftRightConcreteCases(
    t.params.inputSource === 'const',
    !type.signed,
    builder
  );
  await run(t, compoundBinary('>>='), [type, Type.u32], type, t.params, cases);
});