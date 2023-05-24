/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for the u32 arithmetic binary expression operations
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeU32, TypeVec } from '../../../../util/conversion.js';
import { sparseU32Range, vectorU32Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import {
  allInputSources,
  generateBinaryToU32Cases,
  generateU32VectorBinaryToVectorCases,
  generateVectorU32BinaryToVectorCases,
  run,
} from '../expression.js';

import { binary, compoundBinary } from './binary.js';

function u32_add(x, y) {
  return x + y;
}

function u32_subtract(x, y) {
  return x - y;
}

function u32_multiply(x, y) {
  return Math.imul(x, y);
}

function u32_divide_non_const(x, y) {
  if (y === 0) {
    return x;
  }
  return x / y;
}

function u32_divide_const(x, y) {
  if (y === 0) {
    return undefined;
  }
  return x / y;
}

function u32_remainder_non_const(x, y) {
  if (y === 0) {
    return 0;
  }
  return x % y;
}

function u32_remainder_const(x, y) {
  if (y === 0) {
    return undefined;
  }
  return x % y;
}

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('binary/u32_arithmetic', {
  addition: () => {
    return generateBinaryToU32Cases(sparseU32Range(), sparseU32Range(), u32_add);
  },
  subtraction: () => {
    return generateBinaryToU32Cases(sparseU32Range(), sparseU32Range(), u32_subtract);
  },
  multiplication: () => {
    return generateBinaryToU32Cases(sparseU32Range(), sparseU32Range(), u32_multiply);
  },
  division_non_const: () => {
    return generateBinaryToU32Cases(sparseU32Range(), sparseU32Range(), u32_divide_non_const);
  },
  division_const: () => {
    return generateBinaryToU32Cases(sparseU32Range(), sparseU32Range(), u32_divide_const);
  },
  remainder_non_const: () => {
    return generateBinaryToU32Cases(sparseU32Range(), sparseU32Range(), u32_remainder_non_const);
  },
  remainder_const: () => {
    return generateBinaryToU32Cases(sparseU32Range(), sparseU32Range(), u32_remainder_const);
  },
  addition_scalar_vector2: () => {
    return generateU32VectorBinaryToVectorCases(sparseU32Range(), vectorU32Range(2), u32_add);
  },
  addition_scalar_vector3: () => {
    return generateU32VectorBinaryToVectorCases(sparseU32Range(), vectorU32Range(3), u32_add);
  },
  addition_scalar_vector4: () => {
    return generateU32VectorBinaryToVectorCases(sparseU32Range(), vectorU32Range(4), u32_add);
  },
  addition_vector2_scalar: () => {
    return generateVectorU32BinaryToVectorCases(vectorU32Range(2), sparseU32Range(), u32_add);
  },
  addition_vector3_scalar: () => {
    return generateVectorU32BinaryToVectorCases(vectorU32Range(3), sparseU32Range(), u32_add);
  },
  addition_vector4_scalar: () => {
    return generateVectorU32BinaryToVectorCases(vectorU32Range(4), sparseU32Range(), u32_add);
  },
  subtraction_scalar_vector2: () => {
    return generateU32VectorBinaryToVectorCases(sparseU32Range(), vectorU32Range(2), u32_subtract);
  },
  subtraction_scalar_vector3: () => {
    return generateU32VectorBinaryToVectorCases(sparseU32Range(), vectorU32Range(3), u32_subtract);
  },
  subtraction_scalar_vector4: () => {
    return generateU32VectorBinaryToVectorCases(sparseU32Range(), vectorU32Range(4), u32_subtract);
  },
  subtraction_vector2_scalar: () => {
    return generateVectorU32BinaryToVectorCases(vectorU32Range(2), sparseU32Range(), u32_subtract);
  },
  subtraction_vector3_scalar: () => {
    return generateVectorU32BinaryToVectorCases(vectorU32Range(3), sparseU32Range(), u32_subtract);
  },
  subtraction_vector4_scalar: () => {
    return generateVectorU32BinaryToVectorCases(vectorU32Range(4), sparseU32Range(), u32_subtract);
  },
  multiplication_scalar_vector2: () => {
    return generateU32VectorBinaryToVectorCases(sparseU32Range(), vectorU32Range(2), u32_multiply);
  },
  multiplication_scalar_vector3: () => {
    return generateU32VectorBinaryToVectorCases(sparseU32Range(), vectorU32Range(3), u32_multiply);
  },
  multiplication_scalar_vector4: () => {
    return generateU32VectorBinaryToVectorCases(sparseU32Range(), vectorU32Range(4), u32_multiply);
  },
  multiplication_vector2_scalar: () => {
    return generateVectorU32BinaryToVectorCases(vectorU32Range(2), sparseU32Range(), u32_multiply);
  },
  multiplication_vector3_scalar: () => {
    return generateVectorU32BinaryToVectorCases(vectorU32Range(3), sparseU32Range(), u32_multiply);
  },
  multiplication_vector4_scalar: () => {
    return generateVectorU32BinaryToVectorCases(vectorU32Range(4), sparseU32Range(), u32_multiply);
  },
  division_scalar_vector2_non_const: () => {
    return generateU32VectorBinaryToVectorCases(
      sparseU32Range(),
      vectorU32Range(2),
      u32_divide_non_const
    );
  },
  division_scalar_vector3_non_const: () => {
    return generateU32VectorBinaryToVectorCases(
      sparseU32Range(),
      vectorU32Range(3),
      u32_divide_non_const
    );
  },
  division_scalar_vector4_non_const: () => {
    return generateU32VectorBinaryToVectorCases(
      sparseU32Range(),
      vectorU32Range(4),
      u32_divide_non_const
    );
  },
  division_vector2_scalar_non_const: () => {
    return generateVectorU32BinaryToVectorCases(
      vectorU32Range(2),
      sparseU32Range(),
      u32_divide_non_const
    );
  },
  division_vector3_scalar_non_const: () => {
    return generateVectorU32BinaryToVectorCases(
      vectorU32Range(3),
      sparseU32Range(),
      u32_divide_non_const
    );
  },
  division_vector4_scalar_non_const: () => {
    return generateVectorU32BinaryToVectorCases(
      vectorU32Range(4),
      sparseU32Range(),
      u32_divide_non_const
    );
  },
  division_scalar_vector2_const: () => {
    return generateU32VectorBinaryToVectorCases(
      sparseU32Range(),
      vectorU32Range(2),
      u32_divide_const
    );
  },
  division_scalar_vector3_const: () => {
    return generateU32VectorBinaryToVectorCases(
      sparseU32Range(),
      vectorU32Range(3),
      u32_divide_const
    );
  },
  division_scalar_vector4_const: () => {
    return generateU32VectorBinaryToVectorCases(
      sparseU32Range(),
      vectorU32Range(4),
      u32_divide_const
    );
  },
  division_vector2_scalar_const: () => {
    return generateVectorU32BinaryToVectorCases(
      vectorU32Range(2),
      sparseU32Range(),
      u32_divide_const
    );
  },
  division_vector3_scalar_const: () => {
    return generateVectorU32BinaryToVectorCases(
      vectorU32Range(3),
      sparseU32Range(),
      u32_divide_const
    );
  },
  division_vector4_scalar_const: () => {
    return generateVectorU32BinaryToVectorCases(
      vectorU32Range(4),
      sparseU32Range(),
      u32_divide_const
    );
  },
  remainder_scalar_vector2_non_const: () => {
    return generateU32VectorBinaryToVectorCases(
      sparseU32Range(),
      vectorU32Range(2),
      u32_remainder_non_const
    );
  },
  remainder_scalar_vector3_non_const: () => {
    return generateU32VectorBinaryToVectorCases(
      sparseU32Range(),
      vectorU32Range(3),
      u32_remainder_non_const
    );
  },
  remainder_scalar_vector4_non_const: () => {
    return generateU32VectorBinaryToVectorCases(
      sparseU32Range(),
      vectorU32Range(4),
      u32_remainder_non_const
    );
  },
  remainder_vector2_scalar_non_const: () => {
    return generateVectorU32BinaryToVectorCases(
      vectorU32Range(2),
      sparseU32Range(),
      u32_remainder_non_const
    );
  },
  remainder_vector3_scalar_non_const: () => {
    return generateVectorU32BinaryToVectorCases(
      vectorU32Range(3),
      sparseU32Range(),
      u32_remainder_non_const
    );
  },
  remainder_vector4_scalar_non_const: () => {
    return generateVectorU32BinaryToVectorCases(
      vectorU32Range(4),
      sparseU32Range(),
      u32_remainder_non_const
    );
  },
  remainder_scalar_vector2_const: () => {
    return generateU32VectorBinaryToVectorCases(
      sparseU32Range(),
      vectorU32Range(2),
      u32_remainder_const
    );
  },
  remainder_scalar_vector3_const: () => {
    return generateU32VectorBinaryToVectorCases(
      sparseU32Range(),
      vectorU32Range(3),
      u32_remainder_const
    );
  },
  remainder_scalar_vector4_const: () => {
    return generateU32VectorBinaryToVectorCases(
      sparseU32Range(),
      vectorU32Range(4),
      u32_remainder_const
    );
  },
  remainder_vector2_scalar_const: () => {
    return generateVectorU32BinaryToVectorCases(
      vectorU32Range(2),
      sparseU32Range(),
      u32_remainder_const
    );
  },
  remainder_vector3_scalar_const: () => {
    return generateVectorU32BinaryToVectorCases(
      vectorU32Range(3),
      sparseU32Range(),
      u32_remainder_const
    );
  },
  remainder_vector4_scalar_const: () => {
    return generateVectorU32BinaryToVectorCases(
      vectorU32Range(4),
      sparseU32Range(),
      u32_remainder_const
    );
  },
});

g.test('addition')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x + y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('addition');
    await run(t, binary('+'), [TypeU32, TypeU32], TypeU32, t.params, cases);
  });

g.test('addition_compound')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x += y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('addition');
    await run(t, compoundBinary('+='), [TypeU32, TypeU32], TypeU32, t.params, cases);
  });

g.test('subtraction')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x - y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('subtraction');
    await run(t, binary('-'), [TypeU32, TypeU32], TypeU32, t.params, cases);
  });

g.test('subtraction_compound')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x -= y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('subtraction');
    await run(t, compoundBinary('-='), [TypeU32, TypeU32], TypeU32, t.params, cases);
  });

g.test('multiplication')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x * y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('multiplication');
    await run(t, binary('*'), [TypeU32, TypeU32], TypeU32, t.params, cases);
  });

g.test('multiplication_compound')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x *= y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('multiplication');
    await run(t, compoundBinary('*='), [TypeU32, TypeU32], TypeU32, t.params, cases);
  });

g.test('division')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x / y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'division_const' : 'division_non_const'
    );

    await run(t, binary('/'), [TypeU32, TypeU32], TypeU32, t.params, cases);
  });

g.test('division_compound')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x /= y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'division_const' : 'division_non_const'
    );

    await run(t, compoundBinary('/='), [TypeU32, TypeU32], TypeU32, t.params, cases);
  });

g.test('remainder')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x % y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'remainder_const' : 'remainder_non_const'
    );

    await run(t, binary('%'), [TypeU32, TypeU32], TypeU32, t.params, cases);
  });

g.test('remainder_compound')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x %= y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'remainder_const' : 'remainder_non_const'
    );

    await run(t, compoundBinary('%='), [TypeU32, TypeU32], TypeU32, t.params, cases);
  });

g.test('addition_scalar_vector')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x + y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_rhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_rhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const cases = await d.get(`addition_scalar_vector${vec_size}`);
    await run(t, binary('+'), [TypeU32, vec_type], vec_type, t.params, cases);
  });

g.test('addition_vector_scalar')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x + y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_lhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const cases = await d.get(`addition_vector${vec_size}_scalar`);
    await run(t, binary('+'), [vec_type, TypeU32], vec_type, t.params, cases);
  });

g.test('addition_vector_scalar_compound')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x += y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_lhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const cases = await d.get(`addition_vector${vec_size}_scalar`);
    await run(t, compoundBinary('+='), [vec_type, TypeU32], vec_type, t.params, cases);
  });

g.test('subtraction_scalar_vector')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x - y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_rhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_rhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const cases = await d.get(`subtraction_scalar_vector${vec_size}`);
    await run(t, binary('-'), [TypeU32, vec_type], vec_type, t.params, cases);
  });

g.test('subtraction_vector_scalar')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x - y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_lhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const cases = await d.get(`subtraction_vector${vec_size}_scalar`);
    await run(t, binary('-'), [vec_type, TypeU32], vec_type, t.params, cases);
  });

g.test('subtraction_vector_scalar_compound')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x -= y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_lhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const cases = await d.get(`subtraction_vector${vec_size}_scalar`);
    await run(t, compoundBinary('-='), [vec_type, TypeU32], vec_type, t.params, cases);
  });

g.test('multiplication_scalar_vector')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x * y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_rhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_rhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const cases = await d.get(`multiplication_scalar_vector${vec_size}`);
    await run(t, binary('*'), [TypeU32, vec_type], vec_type, t.params, cases);
  });

g.test('multiplication_vector_scalar')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x * y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_lhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const cases = await d.get(`multiplication_vector${vec_size}_scalar`);
    await run(t, binary('*'), [vec_type, TypeU32], vec_type, t.params, cases);
  });

g.test('multiplication_vector_scalar_compound')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x *= y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_lhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const cases = await d.get(`multiplication_vector${vec_size}_scalar`);
    await run(t, compoundBinary('*='), [vec_type, TypeU32], vec_type, t.params, cases);
  });

g.test('division_scalar_vector')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x / y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_rhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_rhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const source = t.params.inputSource === 'const' ? 'const' : 'non_const';
    const cases = await d.get(`division_scalar_vector${vec_size}_${source}`);
    await run(t, binary('/'), [TypeU32, vec_type], vec_type, t.params, cases);
  });

g.test('division_vector_scalar')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x / y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_lhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const source = t.params.inputSource === 'const' ? 'const' : 'non_const';
    const cases = await d.get(`division_vector${vec_size}_scalar_${source}`);
    await run(t, binary('/'), [vec_type, TypeU32], vec_type, t.params, cases);
  });

g.test('division_vector_scalar_compound')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x /= y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_lhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const source = t.params.inputSource === 'const' ? 'const' : 'non_const';
    const cases = await d.get(`division_vector${vec_size}_scalar_${source}`);
    await run(t, compoundBinary('/='), [vec_type, TypeU32], vec_type, t.params, cases);
  });

g.test('remainder_scalar_vector')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x % y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_rhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_rhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const source = t.params.inputSource === 'const' ? 'const' : 'non_const';
    const cases = await d.get(`remainder_scalar_vector${vec_size}_${source}`);
    await run(t, binary('%'), [TypeU32, vec_type], vec_type, t.params, cases);
  });

g.test('remainder_vector_scalar')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x % y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_lhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const source = t.params.inputSource === 'const' ? 'const' : 'non_const';
    const cases = await d.get(`remainder_vector${vec_size}_scalar_${source}`);
    await run(t, binary('%'), [vec_type, TypeU32], vec_type, t.params, cases);
  });

g.test('remainder_vector_scalar_compound')
  .specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr')
  .desc(
    `
Expression: x %= y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4]))
  .fn(async t => {
    const vec_size = t.params.vectorize_lhs;
    const vec_type = TypeVec(vec_size, TypeU32);
    const source = t.params.inputSource === 'const' ? 'const' : 'non_const';
    const cases = await d.get(`remainder_vector${vec_size}_scalar_${source}`);
    await run(t, compoundBinary('%='), [vec_type, TypeU32], vec_type, t.params, cases);
  });
