/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for literals`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('bools').
desc(`Test that valid bools are accepted.`).
params((u) => u.combine('val', ['true', 'false']).beginSubcases()).
fn((t) => {
  const code = `var test = ${t.params.val};`;
  t.expectCompileResult(true, t.wrapInEntryPoint(code));
});

const kAbstractIntNonNegative = new Set([
'0x123', // hex number
'123', // signed number, no suffix
'0', // zero
'0x3f', // hex with 'f' as last character
'2147483647' // max signed int
]);

const kAbstractIntNegative = new Set([
'-0x123', // hex number
'-123', // signed number, no suffix
'-0x3f', // hex with 'f' as last character
'-2147483647', // nagative of max signed int
'-2147483648' // min signed int
]);

const kI32 = new Set([
'94i', // signed number
'2147483647i', // max signed int
'-2147483647i', // min parsable signed int
'i32(-2147483648)' // min signed int
]);

const kU32 = new Set([
'42u', // unsigned number
'0u', // min unsigned int
'4294967295u' // max unsigned int
]);

{
  const kValidIntegers = new Set([
  ...kAbstractIntNonNegative,
  ...kAbstractIntNegative,
  ...kI32,
  ...kU32]
  );
  const kInvalidIntegers = new Set([
  '0123', // Integer does not start with zero
  '2147483648i', // max signed int + 1
  '-2147483649i', // min signed int - 1
  '4294967295', // a untyped lhs will be i32, so this is too big
  '4294967295i', // max unsigned int with i suffix
  '4294967296u', // max unsigned int + 1
  '-1u' // negative unsigned
  ]);
  g.test('abstract_int').
  desc(`Test that valid integers are accepted, and invalid integers are rejected.`).
  params((u) =>
  u.combine('val', new Set([...kValidIntegers, ...kInvalidIntegers])).beginSubcases()
  ).
  fn((t) => {
    const code = `var test = ${t.params.val};`;
    t.expectCompileResult(kValidIntegers.has(t.params.val), t.wrapInEntryPoint(code));
  });
}

{
  const kValidI32 = new Set([...kAbstractIntNonNegative, ...kAbstractIntNegative, ...kI32]);
  const kInvalidI32 = new Set([
  ...kU32,
  '2147483648', // max signed int + 1
  '2147483648i', // max signed int + 1
  '-2147483649', // min signed int - 1
  '-2147483649i', // min signed int - 1
  '1.0', // no conversion from float
  '1.0f', // no conversion from float
  '1.0h' // no conversion from float
  ]);
  g.test('i32').
  desc(`Test that valid signed integers are accepted, and invalid signed integers are rejected.`).
  params((u) => u.combine('val', new Set([...kValidI32, ...kInvalidI32])).beginSubcases()).
  beforeAllSubcases((t) => {
    if (t.params.val.includes('h')) {
      t.selectDeviceOrSkipTestCase('shader-f16');
    }
  }).
  fn((t) => {
    const { val } = t.params;
    const code = `var test: i32 = ${val};`;
    const extensionList = val.includes('h') ? ['f16'] : [];
    t.expectCompileResult(kValidI32.has(val), t.wrapInEntryPoint(code, extensionList));
  });
}

{
  const kValidU32 = new Set([
  ...kAbstractIntNonNegative,
  ...kU32,
  '4294967295' // max unsigned
  ]);
  const kInvalidU32 = new Set([
  ...kAbstractIntNegative,
  ...kI32,
  '4294967296', // max unsigned int + 1
  '4294967296u', // min unsigned int + 1
  '-1', // min unsigned int - 1
  '1.0', // no conversion from float
  '1.0f', // no conversion from float
  '1.0h' // no conversion from float
  ]);
  g.test('u32').
  desc(
    `Test that valid unsigned integers are accepted, and invalid unsigned integers are rejected.`
  ).
  params((u) => u.combine('val', new Set([...kValidU32, ...kInvalidU32])).beginSubcases()).
  beforeAllSubcases((t) => {
    if (t.params.val.includes('h')) {
      t.selectDeviceOrSkipTestCase('shader-f16');
    }
  }).
  fn((t) => {
    const { val } = t.params;
    const code = `var test: u32 = ${val};`;
    const extensionList = val.includes('h') ? ['f16'] : [];
    t.expectCompileResult(kValidU32.has(val), t.wrapInEntryPoint(code, extensionList));
  });
}

const kF32 = new Set([
'0f', // Zero float
'0.0f', // Zero float
'12.223f', // float value
'12.f', // .f
'.12f', // No leading number with a f
'2.4e+4f', // Positive exponent with f suffix
'2.4e-2f', // Negative exponent with f suffix
'2.e+4f', // Exponent without decimals
'1e-4f', // Exponennt without decimal point
'0x1P+4f' // Hex float no decimal
]);

const kF16 = new Set([
'0h', // Zero half
'1h', // Half no decimal
'.1h', // Half no leading value
'1.1e2h', // Exponent half no sign
'1.1E+2h', // Exponent half, plus (uppercase E)
'2.4e-2h', // Exponent half, negative
'0xep2h', // Hexfloat half lower case p
'0xEp-2h', // Hexfloat uppcase hex value
'0x3p+2h', // Hex float half positive exponent
'0x3.2p+2h' // Hex float with decimal half
]);

const kAbstractFloat = new Set([
'0.0', // Zero float without suffix
'.0', // Zero float without leading value
'12.', // No decimal points
'00012.', // Leading zeros allowed
'.12', // No leading digits
'1.2e2', // Exponent without sign (lowercase e)
'1.2E2', // Exponent without sign (uppercase e)
'1.2e+2', // positive exponent
'2.4e-2', // Negative exponent
'.1e-2', // Exponent without leading number
'0x.3', // Hex float, lowercase X
'0X.3', // Hex float, uppercase X
'0xa.fp+2', // Hex float, lowercase p
'0xa.fP+2', // Hex float, uppercase p
'0xE.fp+2', // Uppercase E (as hex, but matches non hex exponent char)
'0X1.fp-4' // Hex float negative exponent
]);

{
  const kValidFloats = new Set([...kF32, ...kF16, ...kAbstractFloat]);
  const kInvalidFloats = new Set([
  '.f', // Must have a number
  '.e-2', // Exponent without leading values
  '1.e&2f', // Exponent invalid sign
  '1.ef', // Exponent without value
  '1.e+f', // Exponent sign no value
  '0x.p2', // Hex float no value
  '0x1p', // Hex float missing exponent
  '0x1p^', // Hex float invalid exponent
  '1.0e+999999999999f', // Too big
  '0x1.0p+999999999999f', // Too big hex
  '0x1.00000001pf0' // Mantissa too big
  ]);
  const kInvalidF16s = new Set([
  '1.1eh', // Missing exponent value
  '1.1e!2h', // Invalid exponent sign
  '1.1e+h', // Missing exponent with sign
  '1.0e+999999h', // Too large
  '0x1.0p+999999h', // Too large hex
  '0xf.h', // Having suffix "h" without "p" or "P"
  '0x3h' // Having suffix "h" without "p" or "P"
  ]);

  g.test('abstract_float').
  desc(`Test that valid floats are accepted, and invalid floats are rejected`).
  params((u) =>
  u.
  combine('val', new Set([...kValidFloats, ...kInvalidFloats, ...kInvalidF16s])).
  beginSubcases()
  ).
  beforeAllSubcases((t) => {
    if (kF16.has(t.params.val) || kInvalidF16s.has(t.params.val)) {
      t.selectDeviceOrSkipTestCase('shader-f16');
    }
  }).
  fn((t) => {
    const code = `var test = ${t.params.val};`;
    const extensionList = kF16.has(t.params.val) || kInvalidF16s.has(t.params.val) ? ['f16'] : [];
    t.expectCompileResult(
      kValidFloats.has(t.params.val),
      t.wrapInEntryPoint(code, extensionList)
    );
  });
}

{
  const kValidF32 = new Set([
  ...kF32,
  ...kAbstractFloat,
  '1', // AbstractInt
  '-1' // AbstractInt
  ]);
  const kInvalidF32 = new Set([
  ...kF16, // no conversion
  '1u', // unsigned
  '1i', // signed
  '1h', // half float
  '.f', // Must have a number
  '.e-2', // Exponent without leading values
  '1.e&2f', // Exponent invalid sign
  '1.ef', // Exponent without value
  '1.e+f', // Exponent sign no value
  '0x.p2', // Hex float no value
  '0x1p', // Hex float missing exponent
  '0x1p^', // Hex float invalid exponent
  '1.0e+999999999999f', // Too big
  '0x1.0p+999999999999f', // Too big hex
  '0x1.00000001pf0' // Mantissa too big
  ]);

  g.test('f32').
  desc(`Test that valid floats are accepted, and invalid floats are rejected`).
  params((u) => u.combine('val', new Set([...kValidF32, ...kInvalidF32])).beginSubcases()).
  beforeAllSubcases((t) => {
    if (kF16.has(t.params.val)) {
      t.selectDeviceOrSkipTestCase('shader-f16');
    }
  }).
  fn((t) => {
    const { val } = t.params;
    const code = `var test: f32 = ${val};`;
    const extensionList = kF16.has(val) ? ['f16'] : [];
    t.expectCompileResult(kValidF32.has(val), t.wrapInEntryPoint(code, extensionList));
  });
}

{
  const kValidF16 = new Set([
  ...kF16,
  ...kAbstractFloat,
  '1', // AbstractInt
  '-1' // AbstractInt
  ]);
  const kInvalidF16 = new Set([
  ...kF32,
  '1i', // signed int
  '1u', // unsigned int
  '1f', // no conversion from f32 to f16
  '1.1eh', // Missing exponent value
  '1.1e!2h', // Invalid exponent sign
  '1.1e+h', // Missing exponent with sign
  '1.0e+999999h', // Too large
  '0x1.0p+999999h' // Too large hex
  ]);

  g.test('f16').
  desc(
    `
Test that valid half floats are accepted, and invalid half floats are rejected
`
  ).
  params((u) => u.combine('val', new Set([...kValidF16, ...kInvalidF16])).beginSubcases()).
  beforeAllSubcases((t) => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }).
  fn((t) => {
    const { val } = t.params;
    const code = `var test: f16 = ${val};`;
    const extensionList = ['f16'];
    t.expectCompileResult(kValidF16.has(val), t.wrapInEntryPoint(code, extensionList));
  });
}