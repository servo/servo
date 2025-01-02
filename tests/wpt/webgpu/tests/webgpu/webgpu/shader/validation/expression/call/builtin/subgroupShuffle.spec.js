/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for subgroupShuffle, subgroupShuffleXor, subgroupShuffleUp, and subgroupShuffleDown.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  elementTypeOf,
  kAllScalarsAndVectors,
  isConvertible } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kOps = [
'subgroupShuffle',
'subgroupShuffleXor',
'subgroupShuffleUp',
'subgroupShuffleDown'];


g.test('requires_subgroups').
desc('Validates that the subgroups feature is required').
params((u) => u.combine('enable', [false, true]).combine('op', kOps)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn((t) => {
  const wgsl = `
${t.params.enable ? 'enable subgroups;' : ''}
fn foo() {
  _ = ${t.params.op}(0, 0);
}`;

  t.expectCompileResult(t.params.enable, wgsl);
});

g.test('requires_subgroups_f16').
desc('Validates that the subgroups feature is required').
params((u) => u.combine('enable', [false, true]).combine('op', kOps)).
beforeAllSubcases((t) => {
  const features = ['shader-f16', 'subgroups'];
  if (t.params.enable) {
    features.push('subgroups-f16');
  }
  t.selectDeviceOrSkipTestCase(features);
}).
fn((t) => {
  const wgsl = `
enable f16;
enable subgroups;
${t.params.enable ? 'enable subgroups_f16;' : ''}
fn foo() {
  _ = ${t.params.op}(0h, 0);
}`;

  t.expectCompileResult(t.params.enable, wgsl);
});

const kStages = {
  constant: (op) => {
    return `
enable subgroups;
@compute @workgroup_size(16)
fn main() {
  const x = ${op}(0, 0);
}`;
  },
  override: (op) => {
    return `
enable subgroups
override o = ${op}(0, 0);`;
  },
  runtime: (op) => {
    return `
enable subgroups;
@compute @workgroup_size(16)
fn main() {
  let x = ${op}(0, 0);
}`;
  }
};

g.test('early_eval').
desc('Ensures the builtin is not able to be compile time evaluated').
params((u) => u.combine('stage', keysOf(kStages)).combine('op', kOps)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn((t) => {
  const code = kStages[t.params.stage](t.params.op);
  t.expectCompileResult(t.params.stage === 'runtime', code);
});

g.test('must_use').
desc('Tests that the builtin has the @must_use attribute').
params((u) => u.combine('must_use', [true, false]).combine('op', kOps)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn((t) => {
  const wgsl = `
enable subgroups;
@compute @workgroup_size(16)
fn main() {
  ${t.params.must_use ? '_ = ' : ''}${t.params.op}(0, 0);
}`;

  t.expectCompileResult(t.params.must_use, wgsl);
});

const kTypes = objectsToRecord(kAllScalarsAndVectors);

g.test('data_type').
desc('Validates data parameter type').
params((u) => u.combine('type', keysOf(kTypes)).combine('op', kOps)).
beforeAllSubcases((t) => {
  const features = ['subgroups'];
  const type = kTypes[t.params.type];
  if (type.requiresF16()) {
    features.push('shader-f16');
    features.push('subgroups-f16');
  }
  t.selectDeviceOrSkipTestCase(features);
}).
fn((t) => {
  const type = kTypes[t.params.type];
  let enables = `enable subgroups;\n`;
  if (type.requiresF16()) {
    enables += `enable f16;\nenable subgroups_f16;`;
  }
  const wgsl = `
${enables}
@compute @workgroup_size(1)
fn main() {
  _ = ${t.params.op}(${type.create(0).wgsl()}, 0);
}`;

  const eleType = elementTypeOf(type);
  t.expectCompileResult(eleType !== Type.bool, wgsl);
});

g.test('return_type').
desc('Validates return type').
params((u) =>
u.
combine('retType', keysOf(kTypes)).
filter((t) => {
  const type = kTypes[t.retType];
  const eleType = elementTypeOf(type);
  return eleType !== Type.abstractInt && eleType !== Type.abstractFloat;
}).
combine('op', kOps).
combine('paramType', keysOf(kTypes))
).
beforeAllSubcases((t) => {
  const features = ['subgroups'];
  const retType = kTypes[t.params.retType];
  const paramType = kTypes[t.params.paramType];
  if (retType.requiresF16() || paramType.requiresF16()) {
    features.push('shader-f16');
    features.push('subgroups-f16');
  }
  t.selectDeviceOrSkipTestCase(features);
}).
fn((t) => {
  const retType = kTypes[t.params.retType];
  const paramType = kTypes[t.params.paramType];
  let enables = `enable subgroups;\n`;
  if (retType.requiresF16() || paramType.requiresF16()) {
    enables += `enable f16;\nenable subgroups_f16;`;
  }
  const wgsl = `
${enables}
@compute @workgroup_size(1)
fn main() {
  let res : ${retType.toString()} = ${t.params.op}(${paramType.create(0).wgsl()}, 0);
}`;

  // Can't just use isConvertible since functions must concretize the parameter
  // type before examining the whole statement.
  const eleParamType = elementTypeOf(paramType);
  const eleRetType = elementTypeOf(retType);
  let expect = paramType === retType && eleRetType !== Type.bool;
  if (eleParamType === Type.abstractInt) {
    expect = eleRetType === Type.i32 && isConvertible(paramType, retType);
  } else if (eleParamType === Type.abstractFloat) {
    expect = eleRetType === Type.f32 && isConvertible(paramType, retType);
  }
  t.expectCompileResult(expect, wgsl);
});

g.test('param2_type').
desc('Validates shuffle parameter type').
params((u) => u.combine('type', keysOf(kTypes)).combine('op', kOps)).
beforeAllSubcases((t) => {
  const features = ['subgroups'];
  const type = kTypes[t.params.type];
  if (type.requiresF16()) {
    features.push('shader-f16');
    features.push('subgroups-f16');
  }
  t.selectDeviceOrSkipTestCase(features);
}).
fn((t) => {
  const type = kTypes[t.params.type];
  let enables = `enable subgroups;\n`;
  if (type.requiresF16()) {
    enables += `enable f16;\nenable subgroups_f16;`;
  }
  const wgsl = `
${enables}
@compute @workgroup_size(1)
fn main() {
  _ = ${t.params.op}(0, ${type.create(0).wgsl()});
}`;

  const expect =
  isConvertible(type, Type.u32) || type === Type.i32 && t.params.op === 'subgroupShuffle';
  t.expectCompileResult(expect, wgsl);
});

g.test('stage').
desc('validates builtin is only usable in the correct stages').
params((u) => u.combine('stage', ['compute', 'fragment', 'vertex']).combine('op', kOps)).
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
  _ = ${t.params.op}(0, 0);
}

${entry}
`;

  t.expectCompileResult(t.params.stage !== 'vertex', wgsl);
});