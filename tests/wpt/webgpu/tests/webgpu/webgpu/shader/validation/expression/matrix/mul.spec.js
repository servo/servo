/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for matrix multiplication expressions.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../common/util/data_tables.js';
import { kValue } from '../../../../util/constants.js';
import { ShaderValidationTest } from '../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);








const kTests = {
  match: {
    src: 'mat3x2f()'
  },
  bool: {
    src: 'false'
  },
  vec: {
    src: 'vec4f()'
  },
  i32: {
    src: '1i'
  },
  u32: {
    src: '1u'
  },
  texture: {
    src: 't'
  },
  sampler: {
    src: 's'
  },
  atomic: {
    src: 'a'
  },
  struct: {
    src: 'str'
  },
  array: {
    src: 'arr'
  },
  matf_no_match: {
    src: 'mat4x4f()'
  }
};

g.test('invalid').
desc(`Validates types for matrix multiplication`).
params((u) =>
u.
combine('rhs', ['ai', 'mat2x3f()', 'mat2x3h()']).
combine('test', keysOf(kTests)).
combine('swap', [true, false])
).
beforeAllSubcases((t) => {
  if (kTests[t.params.test].is_f16 === true || t.params.rhs.startsWith('mat2x3h(')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  let lhs = kTests[t.params.test].src;
  let rhs = t.params.rhs === 'ai' ? 'mat3x2(0, 0, 0, 0, 0, 0)' : t.params.rhs;

  if (t.params.swap) {
    const a = lhs;
    lhs = rhs;
    rhs = a;
  }

  const code = `
${kTests[t.params.test].is_f16 || t.params.rhs.startsWith('mat2x3h(') ? 'enable f16;' : ''}
@group(0) @binding(0) var t : texture_2d<f32>;
@group(0) @binding(1) var s : sampler;
@group(0) @binding(2) var<storage, read_write> a : atomic<i32>;

struct S { u : u32 }

var<private> arr : array<i32, 4>;
var<private> str : S;

@compute @workgroup_size(1)
fn main() {
  let foo = ${lhs} * ${rhs};
}
`;

  const pass = kTests[t.params.test].src === 'mat3x2f()' && t.params.rhs === 'mat2x3f()';
  t.expectCompileResult(pass, code);
});

g.test('f16_and_f32_matrix').
desc(`Validates that f16 multiplied by an f32 matrix is an error.`).
params((u) => u.combine('rhs', ['mat2x3f()', 'mat2x3h()']).combine('swap', [true, false])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn((t) => {
  let lhs = '1h';
  let rhs = t.params.rhs;
  if (t.params.swap) {
    const a = lhs;
    lhs = rhs;
    rhs = a;
  }

  const code = `
enable f16;

@compute @workgroup_size(1)
fn main() {
  let foo = ${lhs} * ${rhs};
}
`;

  const pass = t.params.rhs === 'mat2x3h()';
  t.expectCompileResult(pass, code);
});

g.test('f32_and_f16_matrix').
desc(`Validates that f32 multiplied by an f16 matrix is an error`).
params((u) => u.combine('rhs', ['mat2x3f()', 'mat2x3h()']).combine('swap', [true, false])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn((t) => {
  let lhs = '1f';
  let rhs = t.params.rhs;
  if (t.params.swap) {
    const a = lhs;
    lhs = rhs;
    rhs = a;
  }

  const code = `
enable f16;

@compute @workgroup_size(1)
fn main() {
  let foo = ${lhs} * ${rhs};
}
`;

  const pass = t.params.rhs === 'mat2x3f()';
  t.expectCompileResult(pass, code);
});

g.test('mat_by_mat').
desc(`Validates that mat * mat is only valid for kxR * Cxk.`).
params((u) =>
u.
combine('ty1', ['f', 'h', '']).
combine('ty2', ['f', 'h', '']).
beginSubcases().
combine('c1', [2, 3, 4]).
combine('r1', [2, 3, 4]).
combine('c2', [2, 3, 4]).
combine('r2', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.ty1 === 'h' || t.params.ty2 === 'h') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const c1 = t.params.c1;
  const c2 = t.params.c2;
  const r1 = t.params.r1;
  const r2 = t.params.r2;

  let t1_val = '';
  if (t.params.ty1 === '') {
    [...Array(c1)].map((_, i) => {
      [...Array(r1)].map((_, k) => {
        t1_val += '0,';
      });
    });
  }

  let t2_val = '';
  if (t.params.ty2 === '') {
    [...Array(c2)].map((_, i) => {
      [...Array(r2)].map((_, k) => {
        t2_val += '0,';
      });
    });
  }

  const code = `
${t.params.ty1 === 'h' || t.params.ty2 === 'h' ? 'enable f16;' : ''}
@compute @workgroup_size(1)
fn main() {
  let foo = mat${c1}x${r1}${t.params.ty1}(${t1_val}) * mat${c2}x${r2}${t.params.ty2}(${t2_val});
}
`;

  const pass =
  c1 === r2 && (t.params.ty1 === t.params.ty2 || t.params.ty1 === '' || t.params.ty2 === '');
  t.expectCompileResult(pass, code);
});

g.test('mat_by_vec').
desc(`Validates that mat * vec is only valid for CxR * C.`).
params((u) =>
u.
combine('ty1', ['f', 'h', '']).
combine('ty2', ['f', 'h', '']).
beginSubcases().
combine('c1', [2, 3, 4]).
combine('r1', [2, 3, 4]).
combine('v1', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.ty1 === 'h' || t.params.ty2 === 'h') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const c1 = t.params.c1;
  const r1 = t.params.r1;
  const v1 = t.params.v1;

  let t1_val = '';
  if (t.params.ty1 === '') {
    [...Array(c1)].map((_, i) => {
      [...Array(r1)].map((_, k) => {
        t1_val += '0,';
      });
    });
  }

  let t2_val = '';
  if (t.params.ty2 === '') {
    [...Array(v1)].map((_, i) => {
      t2_val += '0,';
    });
  }

  const code = `
${t.params.ty1 === 'h' || t.params.ty2 === 'h' ? 'enable f16;' : ''}
@compute @workgroup_size(1)
fn main() {
  let foo = mat${c1}x${r1}${t.params.ty1}(${t1_val}) * vec${v1}${t.params.ty2}(${t2_val});
}
`;

  const pass =
  c1 === v1 && (t.params.ty1 === t.params.ty2 || t.params.ty1 === '' || t.params.ty2 === '');
  t.expectCompileResult(pass, code);
});

g.test('vec_by_mat').
desc(`Validates that vec * mat is only valid for R * CxR.`).
params((u) =>
u.
combine('ty1', ['f', 'h', '']).
combine('ty2', ['f', 'h', '']).
beginSubcases().
combine('c1', [2, 3, 4]).
combine('r1', [2, 3, 4]).
combine('v1', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.ty1 === 'h' || t.params.ty2 === 'h') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const c1 = t.params.c1;
  const r1 = t.params.r1;
  const v1 = t.params.v1;

  let t1_val = '';
  if (t.params.ty1 === '') {
    [...Array(c1)].map((_, i) => {
      [...Array(r1)].map((_, k) => {
        t1_val += '0,';
      });
    });
  }

  let t2_val = '';
  if (t.params.ty2 === '') {
    [...Array(v1)].map((_, i) => {
      t2_val += '0,';
    });
  }

  const code = `
${t.params.ty1 === 'h' || t.params.ty2 === 'h' ? 'enable f16;' : ''}
@compute @workgroup_size(1)
fn main() {
  let foo = vec${v1}${t.params.ty2}(${t2_val}) * mat${c1}x${r1}${t.params.ty1}(${t1_val});
}
`;

  const pass =
  r1 === v1 && (t.params.ty1 === t.params.ty2 || t.params.ty1 === '' || t.params.ty2 === '');
  t.expectCompileResult(pass, code);
});

g.test('overflow_scalar_f32').
desc(`Validates that f32 scalar multiplication overflows in shader creation`).
params((u) =>
u.
combine('rhs', [kValue.f32.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}f(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      lhs += `${kValue.f32.positive.max},`;
    }
  }
  lhs += ')';
  const rhs = t.params.rhs;

  const code = `
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.rhs === 1, code);
});

g.test('overflow_vec_f32').
desc(
  `Validates that f32 vector multiplication overflows in shader creation. The overflow happens when multiplying the values.`
).
params((u) =>
u.
combine('rhs', [kValue.f32.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}f(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      if (i === 0) {
        lhs += `${kValue.f32.positive.max},`;
      } else {
        lhs += `0,`;
      }
    }
  }
  lhs += ')';
  const rhs = `vec${t.params.c}f(${t.params.rhs})`;

  const code = `
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.rhs === 1, code);
});

g.test('overflow_vec_f32_internal').
desc(
  `Validates that f32 vector multiplication overflows in shader creation. The overflow happens while summing the values.`
).
params((u) =>
u.
combine('lhs', [kValue.f32.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}f(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      lhs += `${t.params.lhs},`;
    }
  }
  lhs += ')';
  const rhs = `vec${t.params.c}f(1)`;

  const code = `
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.lhs === 1, code);
});

g.test('overflow_mat_f32').
desc(
  `Validates that f32 matrix multiplication overflows in shader creation. Overflows when multiplying the values`
).
params((u) =>
u.
combine('rhs', [kValue.f32.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}f(`;
  let rhs = `mat${t.params.r}x${t.params.c}f(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      if (i === 0) {
        lhs += `${kValue.f32.positive.max},`;
        rhs += `${t.params.rhs},`;
      } else {
        lhs += `0,`;
        rhs += `0,`;
      }
    }
  }
  rhs += ')';
  lhs += ')';

  const code = `
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.rhs === 1, code);
});

g.test('overflow_mat_f32_internal').
desc(
  `Validates that f32 matrix multiplication overflows in shader creation. Overflows when summing the values`
).
params((u) =>
u.
combine('lhs', [kValue.f32.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}f(`;
  let rhs = `mat${t.params.r}x${t.params.c}f(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      lhs += `${t.params.lhs},`;
      rhs += `1,`;
    }
  }
  rhs += ')';
  lhs += ')';

  const code = `
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.lhs === 1, code);
});

g.test('overflow_scalar_f16').
desc(`Validates that f16 scalar multiplication overflows in shader creation`).
params((u) =>
u.
combine('rhs', [kValue.f16.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}h(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      if (i === 0) {
        lhs += `${kValue.f16.positive.max},`;
      } else {
        lhs += `0,`;
      }
    }
  }
  lhs += ')';
  const rhs = t.params.rhs;

  const code = `
enable f16;
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.rhs === 1, code);
});

g.test('overflow_vec_f16').
desc(
  `Validates that f16 vector multiplication overflows in shader creation. Overflow occurs when multiplying.`
).
params((u) =>
u.
combine('rhs', [kValue.f16.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}h(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      lhs += `${kValue.f16.positive.max},`;
    }
  }
  lhs += ')';
  const rhs = `vec${t.params.c}h(${t.params.rhs}/${t.params.c})`;

  const code = `
enable f16;
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.rhs !== kValue.f16.positive.max, code);
});

g.test('overflow_vec_f16_internal').
desc(
  `Validates that f16 vector multiplication overflows in shader creation. Overflow occurs when summing`
).
params((u) =>
u.
combine('lhs', [kValue.f16.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}h(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      lhs += `${t.params.lhs},`;
    }
  }
  lhs += ')';
  const rhs = `vec${t.params.c}h(1)`;

  const code = `
enable f16;
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.lhs === 1, code);
});

g.test('overflow_mat_f16').
desc(
  `Validates that f16 matrix multiplication overflows in shader creation. Overflow occurs when multiplying`
).
params((u) =>
u.
combine('rhs', [kValue.f16.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}h(`;
  let rhs = `mat${t.params.r}x${t.params.c}h(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      if (i === 0) {
        lhs += `${kValue.f16.positive.max},`;
        rhs += `${t.params.rhs},`;
      } else {
        lhs += `0,`;
        rhs += `0,`;
      }
    }
  }
  rhs += ')';
  lhs += ')';

  const code = `
enable f16;
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.rhs === 1, code);
});

g.test('overflow_mat_f16_internal').
desc(
  `Validates that f16 matrix multiplication overflows in shader creation. Overflow occurs when summing`
).
params((u) =>
u.
combine('lhs', [kValue.f16.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}h(`;
  let rhs = `mat${t.params.r}x${t.params.c}h(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      lhs += `${t.params.lhs},`;
      rhs += `1,`;
    }
  }
  rhs += ')';
  lhs += ')';

  const code = `
enable f16;
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.lhs === 1, code);
});

g.test('overflow_scalar_abstract').
desc(`Validates that abstract scalar multiplication overflows in shader creation`).
params((u) =>
u.
combine('rhs', [kValue.f64.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      lhs += `${kValue.f64.positive.max},`;
    }
  }
  lhs += ')';
  const rhs = t.params.rhs;

  const code = `
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.rhs === 1, code);
});

g.test('overflow_vec_abstract').
desc(
  `Validates that abstract vector multiplication overflows in shader creation. Overflow occurs when multiplying.`
).
params((u) =>
u.
combine('rhs', [kValue.f64.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      if (i === 0) {
        lhs += `${kValue.f64.positive.max},`;
      } else {
        lhs += `0,`;
      }
    }
  }
  lhs += ')';
  const rhs = `vec${t.params.c}(${t.params.rhs})`;

  const code = `
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.rhs === 1, code);
});

g.test('overflow_vec_abstract_internal').
desc(
  `Validates that abstract vector multiplication overflows in shader creation. Overflow occurs when summing.`
).
params((u) =>
u.
combine('lhs', [kValue.f64.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      lhs += `${t.params.lhs},`;
    }
  }
  lhs += ')';
  const rhs = `vec${t.params.c}(1)`;

  const code = `
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.lhs === 1, code);
});

g.test('overflow_mat_abstract').
desc(
  `Validates that abstract matrix multiplication overflows in shader creation. Overflow occurs when multiplying.`
).
params((u) =>
u.
combine('rhs', [kValue.f64.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}(`;
  let rhs = `mat${t.params.r}x${t.params.c}(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      if (i === 0) {
        lhs += `${kValue.f64.positive.max},`;
        rhs += `${t.params.rhs},`;
      } else {
        lhs += `0,`;
        rhs += `0,`;
      }
    }
  }
  rhs += ')';
  lhs += ')';

  const code = `
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.rhs === 1, code);
});

g.test('overflow_mat_abstract_internal').
desc(
  `Validates that abstract matrix multiplication overflows in shader creation. Overflow occurs when summing.`
).
params((u) =>
u.
combine('lhs', [kValue.f64.positive.max, 1]).
combine('c', [2, 3, 4]).
combine('r', [2, 3, 4])
).
fn((t) => {
  let lhs = `mat${t.params.c}x${t.params.r}(`;
  let rhs = `mat${t.params.r}x${t.params.c}(`;
  for (let i = 0; i < t.params.c; i++) {
    for (let k = 0; k < t.params.r; k++) {
      lhs += `${t.params.lhs},`;
      rhs += `1,`;
    }
  }
  rhs += ')';
  lhs += ')';

  const code = `
@compute @workgroup_size(1)
fn main() {
  const foo = ${lhs} * ${rhs};
}
`;

  t.expectCompileResult(t.params.lhs === 1, code);
});