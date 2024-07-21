/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Parser validation tests for requires`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { kKnownWGSLLanguageFeatures } from '../../../capability_info.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kCases = {
  valid: { code: `requires readonly_and_readwrite_storage_textures;`, pass: true },
  decl_before: {
    code: `alias i = i32;
requires readonly_and_readwrite_storage_textures;`,
    pass: false
  },
  decl_after: {
    code: `requires readonly_and_readwrite_storage_textures;
alias i = i32;`,
    pass: true
  },
  enable_before: {
    code: `enable f16;
requires readonly_and_readwrite_storage_textures;`,
    pass: true
  },
  diagnostic_before: {
    code: `diagnostic(info, derivative_uniformity);
requires readonly_and_readwrite_storage_textures;`,
    pass: true
  },
  const_assert_before: {
    code: `const_assert 1 == 1;
requires readonly_and_readwrite_storage_textures;`,
    pass: false
  },
  const_assert_after: {
    code: `requires readonly_and_readwrite_storage_textures;
const_assert 1 == 1;`,
    pass: true
  },
  embedded_comment: {
    code: `/* comment

*/requires readonly_and_readwrite_storage_textures;`,
    pass: true
  },
  parens: {
    code: `requires(readonly_and_readwrite_storage_textures);`,
    pass: false
  },
  multi_line: {
    code: `requires
readonly_and_readwrite_storage_textures;`,
    pass: true
  },
  multiple_requires_duplicate: {
    code: `requires readonly_and_readwrite_storage_textures;
requires readonly_and_readwrite_storage_textures;`,
    pass: true
  },
  multiple_requires_different: {
    code: `requires readonly_and_readwrite_storage_textures;
requires packed_4x8_integer_dot_product;`,
    pass: true
  },
  multiple_entries_duplicate: {
    code: `requires readonly_and_readwrite_storage_textures, readonly_and_readwrite_storage_textures, readonly_and_readwrite_storage_textures;`,
    pass: true
  },
  multiple_entries_different: {
    code: `requires readonly_and_readwrite_storage_textures, packed_4x8_integer_dot_product;`,
    pass: true
  },
  unknown: {
    code: `requires unknown;`,
    pass: false
  }
};

g.test('requires').
desc(`Tests that requires are validated correctly.`).
params((u) => u.combine('case', keysOf(kCases))).
beforeAllSubcases((t) => {
  if (t.params.case === 'enable_before') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('readonly_and_readwrite_storage_textures');
  t.skipIfLanguageFeatureNotSupported('packed_4x8_integer_dot_product');

  const c = kCases[t.params.case];
  t.expectCompileResult(c.pass, c.code);
});

g.test('wgsl_matches_api').
desc(`Tests that language features are accepted iff the API reports support for them.`).
params((u) => u.combine('feature', kKnownWGSLLanguageFeatures)).
fn((t) => {
  const code = `requires ${t.params.feature};`;
  t.expectCompileResult(t.hasLanguageFeature(t.params.feature), code);
});