/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for entry point built-in variables`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

import { generateShader } from './util.js';

export const g = makeTestGroup(ShaderValidationTest);

// List of all built-in variables and their stage, in|out usage, and type.
// Taken from table in Section 15:
// https://www.w3.org/TR/2021/WD-WGSL-20211013/#builtin-variables
export const kBuiltins = [
{ name: 'vertex_index', stage: 'vertex', io: 'in', type: 'u32' },
{ name: 'instance_index', stage: 'vertex', io: 'in', type: 'u32' },
{ name: 'position', stage: 'vertex', io: 'out', type: 'vec4<f32>' },
{ name: 'position', stage: 'fragment', io: 'in', type: 'vec4<f32>' },
{ name: 'front_facing', stage: 'fragment', io: 'in', type: 'bool' },
{ name: 'frag_depth', stage: 'fragment', io: 'out', type: 'f32' },
{ name: 'local_invocation_id', stage: 'compute', io: 'in', type: 'vec3<u32>' },
{ name: 'local_invocation_index', stage: 'compute', io: 'in', type: 'u32' },
{ name: 'global_invocation_id', stage: 'compute', io: 'in', type: 'vec3<u32>' },
{ name: 'workgroup_id', stage: 'compute', io: 'in', type: 'vec3<u32>' },
{ name: 'num_workgroups', stage: 'compute', io: 'in', type: 'vec3<u32>' },
{ name: 'sample_index', stage: 'fragment', io: 'in', type: 'u32' },
{ name: 'sample_mask', stage: 'fragment', io: 'in', type: 'u32' },
{ name: 'sample_mask', stage: 'fragment', io: 'out', type: 'u32' }];


// List of types to test against.
const kTestTypes = [
'bool',
'u32',
'i32',
'f32',
'vec2<bool>',
'vec2<u32>',
'vec2<i32>',
'vec2<f32>',
'vec3<bool>',
'vec3<u32>',
'vec3<i32>',
'vec3<f32>',
'vec4<bool>',
'vec4<u32>',
'vec4<i32>',
'vec4<f32>',
'mat2x2<f32>',
'mat2x3<f32>',
'mat2x4<f32>',
'mat3x2<f32>',
'mat3x3<f32>',
'mat3x4<f32>',
'mat4x2<f32>',
'mat4x3<f32>',
'mat4x4<f32>',
'atomic<u32>',
'atomic<i32>',
'array<bool,4>',
'array<u32,4>',
'array<i32,4>',
'array<f32,4>',
'MyStruct'];


g.test('stage_inout').
desc(
  `Test that each @builtin attribute is validated against the required stage and in/out usage for that built-in variable.`
).
params((u) =>
u.
combineWithParams(kBuiltins).
combine('use_struct', [true, false]).
combine('target_stage', ['', 'vertex', 'fragment', 'compute']).
combine('target_io', ['in', 'out']).
beginSubcases()
).
fn((t) => {
  const code = generateShader({
    attribute: `@builtin(${t.params.name})`,
    type: t.params.type,
    stage: t.params.target_stage,
    io: t.params.target_io,
    use_struct: t.params.use_struct
  });

  // Expect to pass iff the built-in table contains an entry that matches.
  const expectation = kBuiltins.some(
    (x) =>
    x.name === t.params.name && (
    x.stage === t.params.target_stage ||
    t.params.use_struct && t.params.target_stage === '') && (
    x.io === t.params.target_io || t.params.target_stage === '') &&
    x.type === t.params.type
  );
  t.expectCompileResult(expectation, code);
});

g.test('type').
desc(
  `Test that each @builtin attribute is validated against the required type of that built-in variable.`
).
params((u) =>
u.
combineWithParams(kBuiltins).
combine('use_struct', [true, false]).
combine('target_type', kTestTypes).
beginSubcases()
).
fn((t) => {
  let code = '';

  if (t.params.target_type === 'MyStruct') {
    // Generate a struct that contains the correct built-in type.
    code += 'struct MyStruct {\n';
    code += `  value : ${t.params.type}\n`;
    code += '};\n\n';
  }

  code += generateShader({
    attribute: `@builtin(${t.params.name})`,
    type: t.params.target_type,
    stage: t.params.stage,
    io: t.params.io,
    use_struct: t.params.use_struct
  });

  // Expect to pass iff the built-in table contains an entry that matches.
  const expectation = kBuiltins.some(
    (x) =>
    x.name === t.params.name &&
    x.stage === t.params.stage &&
    x.io === t.params.io &&
    x.type === t.params.target_type
  );
  t.expectCompileResult(expectation, code);
});

g.test('nesting').
desc(`Test validation of nested built-in variables`).
params((u) =>
u.
combine('target_stage', ['fragment', '']).
combine('target_io', ['in', 'out']).
beginSubcases()
).
fn((t) => {
  // Generate a struct that contains a sample_mask builtin, nested inside another struct.
  let code = `
    struct Inner {
      @builtin(sample_mask) value : u32
    };
    struct Outer {
      inner : Inner
    };`;

  code += generateShader({
    attribute: '',
    type: 'Outer',
    stage: t.params.target_stage,
    io: t.params.target_io,
    use_struct: false
  });

  // Expect to pass only if the struct is not used for entry point IO.
  t.expectCompileResult(t.params.target_stage === '', code);
});

g.test('duplicates').
desc(`Test that duplicated built-in variables are validated.`).
params((u) =>
u
// Place two @builtin(sample_mask) attributes onto the entry point function.
// We use `sample_mask` as it is valid as both an input and output for the same entry point.
// The function:
// - has two non-struct parameters (`p1` and `p2`)
// - has two struct parameters each with two members (`s1{a,b}` and `s2{a,b}`)
// - returns a struct with two members (`ra` and `rb`)
// By default, all of these variables will have unique @location() attributes.
.combine('first', ['p1', 's1a', 's2a', 'ra']).
combine('second', ['p2', 's1b', 's2b', 'rb']).
beginSubcases()
).
fn((t) => {
  const p1 =
  t.params.first === 'p1' ? '@builtin(sample_mask)' : '@location(1) @interpolate(flat)';
  const p2 =
  t.params.second === 'p2' ? '@builtin(sample_mask)' : '@location(2) @interpolate(flat)';
  const s1a =
  t.params.first === 's1a' ? '@builtin(sample_mask)' : '@location(3) @interpolate(flat)';
  const s1b =
  t.params.second === 's1b' ? '@builtin(sample_mask)' : '@location(4) @interpolate(flat)';
  const s2a =
  t.params.first === 's2a' ? '@builtin(sample_mask)' : '@location(5) @interpolate(flat)';
  const s2b =
  t.params.second === 's2b' ? '@builtin(sample_mask)' : '@location(6) @interpolate(flat)';
  const ra =
  t.params.first === 'ra' ? '@builtin(sample_mask)' : '@location(1) @interpolate(flat)';
  const rb =
  t.params.second === 'rb' ? '@builtin(sample_mask)' : '@location(2) @interpolate(flat)';
  const code = `
    struct S1 {
      ${s1a} a : u32,
      ${s1b} b : u32,
    };
    struct S2 {
      ${s2a} a : u32,
      ${s2b} b : u32,
    };
    struct R {
      ${ra} a : u32,
      ${rb} b : u32,
    };
    @fragment
    fn main(${p1} p1 : u32,
            ${p2} p2 : u32,
            s1 : S1,
            s2 : S2,
            ) -> R {
      return R();
    }
    `;

  // The test should fail if both @builtin(sample_mask) attributes are on the input parameters
  // or structures, or it they are both on the output struct. Otherwise it should pass.
  const firstIsRet = t.params.first === 'ra';
  const secondIsRet = t.params.second === 'rb';
  const expectation = firstIsRet !== secondIsRet;
  t.expectCompileResult(expectation, code);
});

g.test('missing_vertex_position').
desc(`Test that vertex shaders are required to output @builtin(position).`).
params((u) =>
u.
combine('use_struct', [true, false]).
combine('attribute', ['@builtin(position)', '@location(0)']).
beginSubcases()
).
fn((t) => {
  const code = `
    struct S {
      ${t.params.attribute} value : vec4<f32>
    };

    @vertex
    fn main() -> ${t.params.use_struct ? 'S' : `${t.params.attribute} vec4<f32>`} {
      return ${t.params.use_struct ? 'S' : 'vec4<f32>'}();
    }
    `;

  // Expect to pass only when using @builtin(position).
  t.expectCompileResult(t.params.attribute === '@builtin(position)', code);
});

g.test('reuse_builtin_name').
desc(`Test that a builtin name can be used in different contexts`).
params((u) =>
u.
combineWithParams(kBuiltins).
combine('use', ['alias', 'struct', 'function', 'module-var', 'function-var'])
).
fn((t) => {
  let code = '';
  if (t.params.use === 'alias') {
    code += `alias ${t.params.name} = i32;`;
  } else if (t.params.use === `struct`) {
    code += `struct ${t.params.name} { i: f32, }`;
  } else if (t.params.use === `function`) {
    code += `fn ${t.params.name}() {}`;
  } else if (t.params.use === `module-var`) {
    code += `const ${t.params.name} = 1;`;
  } else if (t.params.use === `function-var`) {
    code += `fn test() { let ${t.params.name} = 1; }`;
  }
  t.expectCompileResult(true, code);
});