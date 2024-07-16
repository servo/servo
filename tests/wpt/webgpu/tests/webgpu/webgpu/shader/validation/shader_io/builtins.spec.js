/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for entry point built-in variables`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
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
beforeAllSubcases((t) => {
  t.skipIf(
    t.isCompatibility && ['sample_index', 'sample_mask'].includes(t.params.name),
    'compatibility mode does not support sample_index or sample_mask'
  );
}).
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
beginSubcases().
combine('target_type', kTestTypes).
combine('use_struct', [true, false])
).
beforeAllSubcases((t) => {
  t.skipIf(
    t.isCompatibility && ['sample_index', 'sample_mask'].includes(t.params.name),
    'compatibility mode does not support sample_index or sample_mask'
  );
}).
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
  // Generate a struct that contains a frag_depth builtin, nested inside another struct.
  let code = `
    struct Inner {
      @builtin(frag_depth) value : f32
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
beforeAllSubcases((t) => {
  t.skipIf(t.isCompatibility, 'compatibility mode does not support sample_mask');
}).
fn((t) => {
  const p1 =
  t.params.first === 'p1' ? '@builtin(sample_mask)' : '@location(1) @interpolate(flat, either)';
  const p2 =
  t.params.second === 'p2' ?
  '@builtin(sample_mask)' :
  '@location(2) @interpolate(flat, either)';
  const s1a =
  t.params.first === 's1a' ?
  '@builtin(sample_mask)' :
  '@location(3) @interpolate(flat, either)';
  const s1b =
  t.params.second === 's1b' ?
  '@builtin(sample_mask)' :
  '@location(4) @interpolate(flat, either)';
  const s2a =
  t.params.first === 's2a' ?
  '@builtin(sample_mask)' :
  '@location(5) @interpolate(flat, either)';
  const s2b =
  t.params.second === 's2b' ?
  '@builtin(sample_mask)' :
  '@location(6) @interpolate(flat, either)';
  const ra =
  t.params.first === 'ra' ? '@builtin(sample_mask)' : '@location(1) @interpolate(flat, either)';
  const rb =
  t.params.second === 'rb' ?
  '@builtin(sample_mask)' :
  '@location(2) @interpolate(flat, either)';
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

const kTests = {
  pos: {
    src: `@builtin(position)`,
    pass: true
  },
  trailing_comma: {
    src: `@builtin(position,)`,
    pass: true
  },
  newline_in_attr: {
    src: `@ \n builtin(position)`,
    pass: true
  },
  whitespace_in_attr: {
    src: `@/* comment */builtin/* comment */\n\n(\t/*comment*/position/*comment*/)`,
    pass: true
  },
  invalid_name: {
    src: `@abuiltin(position)`,
    pass: false
  },
  no_params: {
    src: `@builtin`,
    pass: false
  },
  missing_param: {
    src: `@builtin()`,
    pass: false
  },
  missing_parens: {
    src: `@builtin position`,
    pass: false
  },
  missing_lparen: {
    src: `@builtin position)`,
    pass: false
  },
  missing_rparen: {
    src: `@builtin(position`,
    pass: false
  },
  multiple_params: {
    src: `@builtin(position, frag_depth)`,
    pass: false
  },
  ident_param: {
    src: `@builtin(identifier)`,
    pass: false
  },
  number_param: {
    src: `@builtin(2)`,
    pass: false
  },
  duplicate: {
    src: `@builtin(position) @builtin(position)`,
    pass: false
  }
};

g.test('parse').
desc(`Test that @builtin is parsed correctly.`).
params((u) => u.combine('builtin', keysOf(kTests))).
fn((t) => {
  const src = kTests[t.params.builtin].src;
  const code = `
@vertex
fn main() -> ${src} vec4<f32> {
  return vec4<f32>(.4, .2, .3, .1);
}`;
  t.expectCompileResult(kTests[t.params.builtin].pass, code);
});

g.test('placement').
desc('Tests the locations @builtin is allowed to appear').
params((u) =>
u.
combine('scope', [
// The fn-param and fn-ret are part of the shader_io/builtins tests
'private-var',
'storage-var',
'struct-member',
'non-ep-param',
'non-ep-ret',
'fn-decl',
'fn-var',
'while-stmt',
undefined]
).
combine('attribute', [
{
  'private-var': false,
  'storage-var': false,
  'struct-member': true,
  'non-ep-param': false,
  'non-ep-ret': false,
  'fn-decl': false,
  'fn-var': false,
  'fn-return': false,
  'while-stmt': false
}]
).
beginSubcases()
).
fn((t) => {
  const scope = t.params.scope;

  const attr = '@builtin(vertex_index)';
  const code = `
      ${scope === 'private-var' ? attr : ''}
      var<private> priv_var : u32;

      ${scope === 'storage-var' ? attr : ''}
      @group(0) @binding(0)
      var<storage> stor_var : u32;

      struct A {
        ${scope === 'struct-member' ? attr : ''}
        a : u32,
      }

      fn v(${scope === 'non-ep-param' ? attr : ''} i : u32) ->
            ${scope === 'non-ep-ret' ? attr : ''} u32 { return 1; }

      @vertex
      ${scope === 'fn-decl' ? attr : ''}
      fn f(
        @location(0) b : u32,
      ) -> @builtin(position) vec4f {
        ${scope === 'fn-var' ? attr : ''}
        var<function> func_v : u32;

        ${scope === 'while-stmt' ? attr : ''}
        while false {}

        return vec4(1, 1, 1, 1);
      }
    `;

  t.expectCompileResult(scope === undefined || t.params.attribute[scope], code);
});