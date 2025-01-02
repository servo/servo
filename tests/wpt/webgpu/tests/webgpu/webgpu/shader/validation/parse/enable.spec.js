/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Parser validation tests for enable`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kCases = {
  f16: { code: `enable f16;`, pass: true },
  decl_before: {
    code: `alias i = i32;
enable f16;`,
    pass: false
  },
  decl_after: {
    code: `enable f16;
alias i = i32;`,
    pass: true
  },
  requires_before: {
    code: `requires readonly_and_readwrite_storage_textures;
enable f16;`,
    pass: true
  },
  diagnostic_before: {
    code: `diagnostic(info, derivative_uniformity);
enable f16;`,
    pass: true
  },
  const_assert_before: {
    code: `const_assert 1 == 1;
enable f16;`,
    pass: false
  },
  const_assert_after: {
    code: `enable f16;
const_assert 1 == 1;`,
    pass: true
  },
  embedded_comment: {
    code: `/* comment

*/enable f16;`,
    pass: true
  },
  parens: {
    code: `enable(f16);`,
    pass: false
  },
  multi_line: {
    code: `enable
f16;`,
    pass: true
  },
  multiple_enables: {
    code: `enable f16;
enable f16;`,
    pass: true
  },
  multiple_entries: {
    code: `enable f16, f16, f16;`,
    pass: true
  },
  unknown: {
    code: `enable unknown;`,
    pass: false
  },
  subgroups: {
    code: `enable subgroups;`,
    pass: true
  },
  subgroups_f16_fail: {
    code: `enable subgroups_f16;`,
    pass: false
  },
  subgroups_f16_pass1: {
    code: `
    enable subgroups_f16;
    enable subgroups;
    enable f16;`,
    pass: true
  },
  subgroups_f16_pass2: {
    code: `
    enable f16;
    enable subgroups;
    enable subgroups_f16;`,
    pass: true
  }
};

g.test('enable').
desc(`Tests that enables are validated correctly`).
beforeAllSubcases((t) => {
  const features = [];
  const name = t.params.case;
  if (name.includes('subgroups_f16')) {
    features.push('subgroups');
    features.push('subgroups-f16');
    features.push('shader-f16');
  } else if (name.includes('subgroups')) {
    features.push('subgroups');
  } else {
    features.push('shader-f16');
  }
  t.selectDeviceOrSkipTestCase(features);
}).
params((u) => u.combine('case', keysOf(kCases))).
fn((t) => {
  if (t.params.case === 'requires_before') {
    t.skipIfLanguageFeatureNotSupported('readonly_and_readwrite_storage_textures');
  }

  const c = kCases[t.params.case];
  t.expectCompileResult(c.pass, c.code);
});