/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for identifiers`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValidIdentifiers = new Set([
  'foo',
  'Foo',
  'FOO',
  '_0',
  '_foo0',
  '_0foo',
  'foo__0',
  'Δέλτα',
  'réflexion',
  'Кызыл',
  '𐰓𐰏𐰇',
  '朝焼け',
  'سلام',
  '검정',
  'שָׁלוֹם',
  'गुलाबी',
  'փիրուզ',
  // Builtin type identifiers:
  'array',
  'atomic',
  'bool',
  'bf16',
  'bitcast',
  'f32',
  'f16',
  'f64',
  'i32',
  'i16',
  'i64',
  'i8',
  'mat2x2',
  'mat2x3',
  'mat2x4',
  'mat3x2',
  'mat3x3',
  'mat3x4',
  'mat4x2',
  'mat4x3',
  'mat4x4',
  'ptr',
  'quat',
  'sampler',
  'sampler_comparison',
  'signed',
  'texture_1d',
  'texture_2d',
  'texture_2d_array',
  'texture_3d',
  'texture_cube',
  'texture_cube_array',
  'texture_multisampled_2d',
  'texture_storage_1d',
  'texture_storage_2d',
  'texture_storage_2d_array',
  'texture_storage_3d',
  'texture_depth_2d',
  'texture_depth_2d_array',
  'texture_depth_cube',
  'texture_depth_cube_array',
  'texture_depth_multisampled_2d',
  'u32',
  'u16',
  'u64',
  'u8',
  'unsigned',
  'vec2',
  'vec3',
  'vec4',
]);

const kInvalidIdentifiers = new Set([
  '_', // Single underscore is a syntactic token for phony assignment.
  '__', // Leading double underscore is reserved.
  '__foo', // Leading double underscore is reserved.
  '0foo', // Must start with single underscore or a letter.
  // No punctuation:
  'foo.bar',
  'foo-bar',
  'foo+bar',
  'foo#bar',
  'foo!bar',
  'foo\\bar',
  'foo/bar',
  'foo,bar',
  'foo@bar',
  'foo::bar',
  // Keywords:
  'alias',
  'break',
  'case',
  'const',
  'const_assert',
  'continue',
  'continuing',
  'default',
  'diagnostic',
  'discard',
  'else',
  'enable',
  'false',
  'fn',
  'for',
  'if',
  'let',
  'loop',
  'override',
  'requires',
  'return',
  'struct',
  'switch',
  'true',
  'var',
  'while',
  // Reserved Words
  'NULL',
  'Self',
  'abstract',
  'active',
  'alignas',
  'alignof',
  'as',
  'asm',
  'asm_fragment',
  'async',
  'attribute',
  'auto',
  'await',
  'become',
  'binding_array',
  'cast',
  'catch',
  'class',
  'co_await',
  'co_return',
  'co_yield',
  'coherent',
  'column_major',
  'common',
  'compile',
  'compile_fragment',
  'concept',
  'const_cast',
  'consteval',
  'constexpr',
  'constinit',
  'crate',
  'debugger',
  'decltype',
  'delete',
  'demote',
  'demote_to_helper',
  'do',
  'dynamic_cast',
  'enum',
  'explicit',
  'export',
  'extends',
  'extern',
  'external',
  'fallthrough',
  'filter',
  'final',
  'finally',
  'friend',
  'from',
  'fxgroup',
  'get',
  'goto',
  'groupshared',
  'highp',
  'impl',
  'implements',
  'import',
  'inline',
  'instanceof',
  'interface',
  'layout',
  'lowp',
  'macro',
  'macro_rules',
  'match',
  'mediump',
  'meta',
  'mod',
  'module',
  'move',
  'mut',
  'mutable',
  'namespace',
  'new',
  'nil',
  'noexcept',
  'noinline',
  'nointerpolation',
  'noperspective',
  'null',
  'nullptr',
  'of',
  'operator',
  'package',
  'packoffset',
  'partition',
  'pass',
  'patch',
  'pixelfragment',
  'precise',
  'precision',
  'premerge',
  'priv',
  'protected',
  'pub',
  'public',
  'readonly',
  'ref',
  'regardless',
  'register',
  'reinterpret_cast',
  'require',
  'resource',
  'restrict',
  'self',
  'set',
  'shared',
  'sizeof',
  'smooth',
  'snorm',
  'static',
  'static_assert',
  'static_cast',
  'std',
  'subroutine',
  'super',
  'target',
  'template',
  'this',
  'thread_local',
  'throw',
  'trait',
  'try',
  'type',
  'typedef',
  'typeid',
  'typename',
  'typeof',
  'union',
  'unless',
  'unorm',
  'unsafe',
  'unsized',
  'use',
  'using',
  'varying',
  'virtual',
  'volatile',
  'wgsl',
  'where',
  'with',
  'writeonly',
  'yield',
]);

g.test('module_var_name')
  .desc(
    `Test that valid identifiers are accepted for names of module-scope 'var's, and invalid identifiers are rejected.`
  )
  .params(u =>
    u.combine('ident', new Set([...kValidIdentifiers, ...kInvalidIdentifiers])).beginSubcases()
  )
  .fn(t => {
    const type = t.params.ident === 'i32' ? 'u32' : 'i32';
    const code = `var<private> ${t.params.ident} : ${type};`;
    t.expectCompileResult(kValidIdentifiers.has(t.params.ident), code);
  });

g.test('module_const_name')
  .desc(
    `Test that valid identifiers are accepted for names of module-scope 'const's, and invalid identifiers are rejected.`
  )
  .params(u =>
    u.combine('ident', new Set([...kValidIdentifiers, ...kInvalidIdentifiers])).beginSubcases()
  )
  .fn(t => {
    const type = t.params.ident === 'i32' ? 'u32' : 'i32';
    const code = `const ${t.params.ident} : ${type} = 0;`;
    t.expectCompileResult(kValidIdentifiers.has(t.params.ident), code);
  });

g.test('override_name')
  .desc(
    `Test that valid identifiers are accepted for names of 'override's, and invalid identifiers are rejected.`
  )
  .params(u =>
    u.combine('ident', new Set([...kValidIdentifiers, ...kInvalidIdentifiers])).beginSubcases()
  )
  .fn(t => {
    const type = t.params.ident === 'i32' ? 'u32' : 'i32';
    const code = `override ${t.params.ident} : ${type} = 0;`;
    t.expectCompileResult(kValidIdentifiers.has(t.params.ident), code);
  });

g.test('function_name')
  .desc(
    `Test that valid identifiers are accepted for names of functions, and invalid identifiers are rejected.`
  )
  .params(u =>
    u.combine('ident', new Set([...kValidIdentifiers, ...kInvalidIdentifiers])).beginSubcases()
  )
  .fn(t => {
    const code = `fn ${t.params.ident}() {}`;
    t.expectCompileResult(kValidIdentifiers.has(t.params.ident), code);
  });

g.test('struct_name')
  .desc(
    `Test that valid identifiers are accepted for names of structs, and invalid identifiers are rejected.`
  )
  .params(u =>
    u.combine('ident', new Set([...kValidIdentifiers, ...kInvalidIdentifiers])).beginSubcases()
  )
  .fn(t => {
    const type = t.params.ident === 'i32' ? 'u32' : 'i32';
    const code = `struct ${t.params.ident} { i : ${type} }`;
    t.expectCompileResult(kValidIdentifiers.has(t.params.ident), code);
  });

g.test('alias_name')
  .desc(
    `Test that valid identifiers are accepted for names of aliases, and invalid identifiers are rejected.`
  )
  .params(u =>
    u.combine('ident', new Set([...kValidIdentifiers, ...kInvalidIdentifiers])).beginSubcases()
  )
  .fn(t => {
    const type = t.params.ident === 'i32' ? 'u32' : 'i32';
    const code = `alias ${t.params.ident} = ${type};`;
    t.expectCompileResult(kValidIdentifiers.has(t.params.ident), code);
  });

g.test('function_param_name')
  .desc(
    `Test that valid identifiers are accepted for names of function parameters, and invalid identifiers are rejected.`
  )
  .params(u =>
    u.combine('ident', new Set([...kValidIdentifiers, ...kInvalidIdentifiers])).beginSubcases()
  )
  .fn(t => {
    const type = t.params.ident === 'i32' ? 'u32' : 'i32';
    const code = `fn F(${t.params.ident} : ${type}) {}`;
    t.expectCompileResult(kValidIdentifiers.has(t.params.ident), code);
  });

g.test('function_const_name')
  .desc(
    `Test that valid identifiers are accepted for names of function-scoped 'const's, and invalid identifiers are rejected.`
  )
  .params(u =>
    u.combine('ident', new Set([...kValidIdentifiers, ...kInvalidIdentifiers])).beginSubcases()
  )
  .fn(t => {
    const code = `fn F() {
  const ${t.params.ident} = 1;
}`;
    t.expectCompileResult(kValidIdentifiers.has(t.params.ident), code);
  });

g.test('function_let_name')
  .desc(
    `Test that valid identifiers are accepted for names of function-scoped 'let's, and invalid identifiers are rejected.`
  )
  .params(u =>
    u.combine('ident', new Set([...kValidIdentifiers, ...kInvalidIdentifiers])).beginSubcases()
  )
  .fn(t => {
    const code = `fn F() {
  let ${t.params.ident} = 1;
}`;
    t.expectCompileResult(kValidIdentifiers.has(t.params.ident), code);
  });

g.test('function_var_name')
  .desc(
    `Test that valid identifiers are accepted for names of function-scoped 'var's, and invalid identifiers are rejected.`
  )
  .params(u =>
    u.combine('ident', new Set([...kValidIdentifiers, ...kInvalidIdentifiers])).beginSubcases()
  )
  .fn(t => {
    const code = `fn F() {
  var ${t.params.ident} = 1;
}`;
    t.expectCompileResult(kValidIdentifiers.has(t.params.ident), code);
  });

g.test('non_normalized')
  .desc(`Test that identifiers are not unicode normalized`)
  .fn(t => {
    const code = `var<private> \u212b : i32;  // \u212b normalizes with NFC to \u00c5
var<private> \u00c5 : i32;`;
    t.expectCompileResult(true, code);
  });
