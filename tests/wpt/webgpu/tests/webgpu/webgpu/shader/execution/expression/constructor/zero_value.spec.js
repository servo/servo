/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for zero value constructors
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import { basicExpressionBuilder, run } from '../expression.js';

export const g = makeTestGroup(GPUTest);

g.test('scalar').
specURL('https://www.w3.org/TR/WGSL/#zero-value-builtin-function').
desc(`Test that a zero value scalar constructor produces the expected zero value`).
params((u) => u.combine('type', ['bool', 'i32', 'u32', 'f32', 'f16'])).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const type = Type[t.params.type];
  await run(
    t,
    basicExpressionBuilder((ops) => `${type}()`),
    [],
    type,
    { inputSource: 'const' },
    [{ input: [], expected: type.create(0) }]
  );
});

g.test('vector').
specURL('https://www.w3.org/TR/WGSL/#zero-value-builtin-function').
desc(`Test that a zero value vector constructor produces the expected zero value`).
params((u) =>
u.
combine('type', ['bool', 'i32', 'u32', 'f32', 'f16']).
combine('width', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const type = Type.vec(t.params.width, Type[t.params.type]);
  await run(
    t,
    basicExpressionBuilder((ops) => `${type}()`),
    [],
    type,
    { inputSource: 'const' },
    [{ input: [], expected: type.create(0) }]
  );
});

g.test('vector_prefix').
desc(`Test that a zero value vector constructor produces the expected zero value`).
params((u) =>
u.combine('type', ['i32', 'u32', 'f32', 'f16']).combine('width', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const type = Type.vec(t.params.width, Type[t.params.type]);
  await run(
    t,
    basicExpressionBuilder((ops) => `vec${t.params.width}()`),
    [],
    type,
    { inputSource: 'const', constEvaluationMode: 'direct' },
    [{ input: [], expected: type.create(0) }]
  );
});

g.test('matrix').
specURL('https://www.w3.org/TR/WGSL/#zero-value-builtin-function').
desc(`Test that a zero value matrix constructor produces the expected zero value`).
params((u) =>
u.
combine('type', ['f32', 'f16']).
combine('columns', [2, 3, 4]).
combine('rows', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const type = Type.mat(t.params.columns, t.params.rows, Type[t.params.type]);
  await run(
    t,
    basicExpressionBuilder((ops) => `${type}()`),
    [],
    type,
    { inputSource: 'const' },
    [{ input: [], expected: type.create(0) }]
  );
});

g.test('array').
specURL('https://www.w3.org/TR/WGSL/#zero-value-builtin-function').
desc(`Test that a zero value matrix constructor produces the expected zero value`).
params((u) =>
u.
combine('type', ['bool', 'i32', 'u32', 'f32', 'f16', 'vec3f', 'vec4i']).
combine('length', [1, 5, 10])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const type = Type.array(t.params.length, Type[t.params.type]);
  await run(
    t,
    basicExpressionBuilder((ops) => `${type}()`),
    [],
    type,
    { inputSource: 'const' },
    [{ input: [], expected: type.create(0) }]
  );
});

g.test('structure').
specURL('https://www.w3.org/TR/WGSL/#zero-value-builtin-function').
desc(`Test that an structure constructed from element values produces the expected value`).
params((u) =>
u.
combine('member_types', [
['bool'],
['u32'],
['vec3f'],
['i32', 'u32'],
['i32', 'f16', 'vec4i', 'mat3x2f'],
['bool', 'u32', 'f16', 'vec3f', 'vec2i'],
['i32', 'u32', 'f32', 'f16', 'vec3f', 'vec4i']]
).
combine('nested', [false, true]).
beginSubcases().
expand('member_index', (t) => t.member_types.map((_, i) => i))
).
beforeAllSubcases((t) => {
  if (t.params.member_types.includes('f16')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const memberType = Type[t.params.member_types[t.params.member_index]];
  const builder = basicExpressionBuilder((_) =>
  t.params.nested ?
  `OuterStruct().inner.member_${t.params.member_index}` :
  `MyStruct().member_${t.params.member_index}`
  );
  await run(
    t,
    (params) => {
      return `
${t.params.member_types.includes('f16') ? 'enable f16;' : ''}

${builder(params)}

struct MyStruct {
${t.params.member_types.map((ty, i) => `  member_${i} : ${ty},`).join('\n')}
};
struct OuterStruct {
  pre : i32,
  inner : MyStruct,
  post : i32,
};
`;
    },
    [],
    memberType,
    { inputSource: 'const' },
    [{ input: [], expected: memberType.create(0) }]
  );
});