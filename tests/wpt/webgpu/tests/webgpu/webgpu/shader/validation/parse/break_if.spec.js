/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for break if`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTests = {
  compound_break: {
    src: '{ break if true; }',
    pass: false
  },
  loop_break: {
    src: 'loop { break if true; }',
    pass: false
  },
  loop_if_break: {
    src: 'loop { if true { break if false; } }',
    pass: false
  },
  continuing_break_if: {
    src: 'loop { continuing { break if true; } }',
    pass: true
  },
  continuing_break_if_parens: {
    src: 'loop { continuing { break if (true); } }',
    pass: true
  },
  continuing_break_if_not_last: {
    src: 'loop { continuing { break if (true); let a = 4;} }',
    pass: false
  },
  while_break: {
    src: 'while true { break if true; }',
    pass: false
  },
  while_if_break: {
    src: 'while true { if true { break if true; } }',
    pass: false
  },
  for_break: {
    src: 'for (;;) { break if true; }',
    pass: false
  },
  for_if_break: {
    src: 'for (;;) { if true { break if true; } }',
    pass: false
  },
  switch_case_break: {
    src: 'switch(1) { default: { break if true; } }',
    pass: false
  },
  switch_case_if_break: {
    src: 'switch(1) { default: { if true { break if true; } } }',
    pass: false
  },
  break: {
    src: 'break if true;',
    pass: false
  },
  return_break: {
    src: 'return break if true;',
    pass: false
  },
  if_break: {
    src: 'if true { break if true; }',
    pass: false
  },
  continuing_if_break: {
    src: 'loop { continuing { if (true) { break if true; } } }',
    pass: false
  },
  switch_break: {
    src: 'switch(1) { break if true; }',
    pass: false
  }
};

g.test('placement').
desc('Test that break if placement is validated correctly').
params((u) => u.combine('stmt', keysOf(kTests))).
fn((t) => {
  const code = `
@vertex
fn vtx() -> @builtin(position) vec4f {
  ${kTests[t.params.stmt].src}
  return vec4f(1);
}
    `;
  t.expectCompileResult(kTests[t.params.stmt].pass, code);
});

const vec_types = [2, 3, 4].
map((i) => ['i32', 'u32', 'f32', 'f16'].map((j) => `vec${i}<${j}>`)).
reduce((a, c) => a.concat(c), []);
const f32_matrix_types = [2, 3, 4].
map((i) => [2, 3, 4].map((j) => `mat${i}x${j}f`)).
reduce((a, c) => a.concat(c), []);
const f16_matrix_types = [2, 3, 4].
map((i) => [2, 3, 4].map((j) => `mat${i}x${j}<f16>`)).
reduce((a, c) => a.concat(c), []);

g.test('non_bool_param').
desc('Test that break if fails with a non-bool parameter').
params((u) =>
u.combine('type', [
'f32',
'f16',
'i32',
'u32',
'S',
...vec_types,
...f32_matrix_types,
...f16_matrix_types]
)
).
beforeAllSubcases((t) => {
  if (t.params.type.includes('f16')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const code = `
struct S {
  a: i32,
}

@vertex
fn vtx() -> @builtin(position) vec4f {
  var v: ${t.params.type};

  loop {
    continuing {
      break if v;
    }
  }
  return vec4f(1);
}`;
  t.expectCompileResult(t.params.type === 'bool', code);
});