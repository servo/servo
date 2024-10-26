/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for subgroupAdd and subgroupExclusiveAdd
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import { Type, elementTypeOf, kAllScalarsAndVectors } from '../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kBuiltins = ['subgroupAdd', 'subgroupExclusiveAdd', 'subgroupInclusiveAdd'];

const kStages = {
  constant: (builtin) => {
    return `
enable subgroups;
@compute @workgroup_size(16)
fn main() {
  const x = ${builtin}(0);
}`;
  },
  override: (builtin) => {
    return `
enable subgroups;
override o = ${builtin}(0);`;
  },
  runtime: (builtin) => {
    return `
enable subgroups;
@compute @workgroup_size(16)
fn main() {
  let x = ${builtin}(0);
}`;
  }
};

g.test('early_eval').
desc('Ensures the builtin is not able to be compile time evaluated').
params((u) => u.combine('stage', keysOf(kStages)).beginSubcases().combine('builtin', kBuiltins)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn((t) => {
  const code = kStages[t.params.stage](t.params.builtin);
  t.expectCompileResult(t.params.stage === 'runtime', code);
});

g.test('must_use').
desc('Tests that the builtin has the @must_use attribute').
params((u) =>
u.
combine('must_use', [true, false]).
beginSubcases().
combine('builtin', kBuiltins)
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn((t) => {
  const wgsl = `
enable subgroups;
@compute @workgroup_size(16)
fn main() {
  ${t.params.must_use ? '_ = ' : ''}${t.params.builtin}(0);
}`;

  t.expectCompileResult(t.params.must_use, wgsl);
});

const kArgumentTypes = objectsToRecord(kAllScalarsAndVectors);

g.test('data_type').
desc('Validates data parameter type').
params((u) =>
u.combine('type', keysOf(kArgumentTypes)).beginSubcases().combine('builtin', kBuiltins)
).
beforeAllSubcases((t) => {
  const features = ['subgroups'];
  const type = kArgumentTypes[t.params.type];
  if (type.requiresF16()) {
    features.push('subgroups-f16');
    features.push('shader-f16');
  }
  t.selectDeviceOrSkipTestCase(features);
}).
fn((t) => {
  const type = kArgumentTypes[t.params.type];
  let enables = `enable subgroups;\n`;
  if (type.requiresF16()) {
    enables += `enable subgroups_f16;\nenable f16;`;
  }
  const wgsl = `
${enables}
@compute @workgroup_size(1)
fn main() {
  _ = ${t.params.builtin}(${type.create(0).wgsl()});
}`;

  t.expectCompileResult(elementTypeOf(type) !== Type.bool, wgsl);
});

g.test('return_type').
desc('Validates data parameter type').
params((u) =>
u.
combine('dataType', keysOf(kArgumentTypes)).
combine('retType', keysOf(kArgumentTypes)).
filter((t) => {
  const retType = kArgumentTypes[t.retType];
  const retEleTy = elementTypeOf(retType);
  const dataType = kArgumentTypes[t.dataType];
  const dataEleTy = elementTypeOf(dataType);
  return (
    retEleTy !== Type.abstractInt &&
    retEleTy !== Type.abstractFloat &&
    dataEleTy !== Type.abstractInt &&
    dataEleTy !== Type.abstractFloat);

}).
beginSubcases().
combine('builtin', kBuiltins)
).
beforeAllSubcases((t) => {
  const features = ['subgroups'];
  const dataType = kArgumentTypes[t.params.dataType];
  const retType = kArgumentTypes[t.params.retType];
  if (dataType.requiresF16() || retType.requiresF16()) {
    features.push('subgroups-f16');
    features.push('shader-f16');
  }
  t.selectDeviceOrSkipTestCase(features);
}).
fn((t) => {
  const dataType = kArgumentTypes[t.params.dataType];
  const retType = kArgumentTypes[t.params.retType];
  let enables = `enable subgroups;\n`;
  if (dataType.requiresF16() || retType.requiresF16()) {
    enables += `enable subgroups_f16;\nenable f16;`;
  }
  const wgsl = `
${enables}
@compute @workgroup_size(1)
fn main() {
  let res : ${retType.toString()} = ${t.params.builtin}(${dataType.create(0).wgsl()});
}`;

  const expect = elementTypeOf(dataType) !== Type.bool && dataType === retType;
  t.expectCompileResult(expect, wgsl);
});

g.test('stage').
desc('Validates it is only usable in correct stage').
params((u) =>
u.
combine('stage', ['compute', 'fragment', 'vertex']).
beginSubcases().
combine('builtin', kBuiltins)
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn((t) => {
  const compute = `
@compute @workgroup_size(1)
fn main() {
  foo();
}`;

  const fragment = `
@fragment
fn main() {
  foo();
}`;

  const vertex = `
@vertex
fn main() -> @builtin(position) vec4f {
  foo();
  return vec4f();
}`;

  const entry = { compute, fragment, vertex }[t.params.stage];
  const wgsl = `
enable subgroups;
fn foo() {
  _ = ${t.params.builtin}(0);
}

${entry}
`;

  t.expectCompileResult(t.params.stage !== 'vertex', wgsl);
});

const kInvalidTypeCases = {
  array_u32: `array(1u,2u,3u)`,
  array_f32: `array<f32, 4>()`,
  struct_s: `S()`,
  struct_t: `T(1, 1)`,
  ptr_func: `&func_var`,
  ptr_priv: `&priv_var`,
  frexp_ret: `frexp(0)`
};

g.test('invalid_types').
desc('Tests that invalid non-plain types are rejected').
params((u) =>
u.combine('case', keysOf(kInvalidTypeCases)).beginSubcases().combine('builtin', kBuiltins)
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn((t) => {
  const val = kInvalidTypeCases[t.params.case];
  const wgsl = `
enable subgroups;

struct S {
  x : u32
}

struct T {
  a : f32,
  b : u32,
}

var<private> priv_var : f32;
fn foo() {
  var func_var : vec4u;
  _ = ${t.params.builtin}(${val});
}`;

  t.expectCompileResult(false, wgsl);
});