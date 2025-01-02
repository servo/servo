/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'determinant';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

// Generate a dictionary mapping each matrix type variation (columns,rows,
// floating point type) to a nontrivial matrix value of that type.
const kMatrixCases = [2, 3, 4].
flatMap((cols) =>
[2, 3, 4].flatMap((rows) =>
['abstract-int', 'abstract-float', 'f32', 'f16'].map((type) => ({
  [`mat${cols}x${rows}_${type}`]: (() => {
    const suffix = (() => {
      switch (type) {
        case 'abstract-int':
          return '';
        case 'abstract-float':
          return '.0';
        case 'f32':
          return 'f';
        case 'f16':
          return 'h';
      }
    })();
    return `(mat${cols}x${rows}(${[...Array(cols * rows).keys()].
    map((e) => `${e}${suffix}`).
    join(', ')}))`;
  })()
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

g.test('matrix_args').
desc(`Test compilation failure of ${builtin} with variously shaped matrices`).
params((u) =>
u.
combine('cols', [2, 3, 4]).
combine('rows', [2, 3, 4]).
combine('type', ['abstract-int', 'abstract-float', 'f32', 'f16'])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const cols = t.params.cols;
  const rows = t.params.rows;
  const type = t.params.type;
  const arg = kMatrixCases[`mat${cols}x${rows}_${type}`];
  t.expectCompileResult(
    cols === rows,
    t.wrapInEntryPoint(`const c = ${builtin}${arg};`, type === 'f16' ? ['f16'] : [])
  );
});

const kArgCases = {
  good: '(mat2x2(0.0, 2.0, 3.0, 4.0))', // Included to check test implementation
  bad_no_parens: '',
  // Bad number of args
  bad_too_few: '()',
  bad_too_many: '(mat2x2(0.0, 2.0, 3.0, 4.0), mat2x2(0.0, 2.0, 3.0, 4.0))',
  // Bad value type for arg 0
  bad_0i32: '(1i)',
  bad_0u32: '(1u)',
  bad_0bool: '(false)',
  bad_0vec2u: '(vec2u())',
  bad_0array: '(array(1.1,2.2))',
  bad_0struct: '(modf(2.2))'
};

g.test('args').
desc(`Test compilation failure of ${builtin} with variously shaped and typed arguments`).
params((u) => u.combine('arg', keysOf(kArgCases))).
fn((t) => {
  t.expectCompileResult(
    t.params.arg === 'good',
    `const c = ${builtin}${kArgCases[t.params.arg]};`
  );
});

g.test('must_use').
desc(`Result of ${builtin} must be used`).
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const use_it = t.params.use ? '_ = ' : '';
  t.expectCompileResult(t.params.use, `fn f() { ${use_it}${builtin}${kArgCases['good']}; }`);
});