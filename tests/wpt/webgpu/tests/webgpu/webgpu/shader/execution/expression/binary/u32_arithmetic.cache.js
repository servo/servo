/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { sparseU32Range, vectorU32Range } from '../../../../util/math.js';import { generateBinaryToU32Cases,
generateU32VectorBinaryToVectorCases,
generateVectorU32BinaryToVectorCases } from
'../case.js';
import { makeCaseCache } from '../case_cache.js';

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
  }
});