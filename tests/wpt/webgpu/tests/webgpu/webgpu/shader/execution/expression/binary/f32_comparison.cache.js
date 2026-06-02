/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { anyOf } from '../../../../util/compare.js';import { bool, f32 } from '../../../../util/conversion.js';import { flushSubnormalNumberF32, vectorF32Range } from '../../../../util/math.js';

import { makeCaseCache } from '../case_cache.js';

/**
 * @returns a test case for the provided left hand & right hand values and truth function.
 * Handles quantization and subnormals.
 */
function makeCase(
lhs,
rhs,
truthFunc)
{
  // Subnormal float values may be flushed at any time.
  // https://www.w3.org/TR/WGSL/#floating-point-evaluation
  const f32_lhs = f32(lhs);
  const f32_rhs = f32(rhs);
  const lhs_options = new Set([f32_lhs, f32(flushSubnormalNumberF32(lhs))]);
  const rhs_options = new Set([f32_rhs, f32(flushSubnormalNumberF32(rhs))]);
  const expected = [];
  lhs_options.forEach((l) => {
    rhs_options.forEach((r) => {
      const result = bool(truthFunc(l, r));
      if (!expected.includes(result)) {
        expected.push(result);
      }
    });
  });

  return { input: [f32_lhs, f32_rhs], expected: anyOf(...expected) };
}

export const d = makeCaseCache('binary/f32_logical', {
  equals_non_const: () => {
    const truthFunc = (lhs, rhs) => {
      return lhs.value === rhs.value;
    };

    return vectorF32Range(2).map((v) => {
      return makeCase(v[0], v[1], truthFunc);
    });
  },
  equals_const: () => {
    const truthFunc = (lhs, rhs) => {
      return lhs.value === rhs.value;
    };

    return vectorF32Range(2).map((v) => {
      return makeCase(v[0], v[1], truthFunc);
    });
  },
  not_equals_non_const: () => {
    const truthFunc = (lhs, rhs) => {
      return lhs.value !== rhs.value;
    };

    return vectorF32Range(2).map((v) => {
      return makeCase(v[0], v[1], truthFunc);
    });
  },
  not_equals_const: () => {
    const truthFunc = (lhs, rhs) => {
      return lhs.value !== rhs.value;
    };

    return vectorF32Range(2).map((v) => {
      return makeCase(v[0], v[1], truthFunc);
    });
  },
  less_than_non_const: () => {
    const truthFunc = (lhs, rhs) => {
      return lhs.value < rhs.value;
    };

    return vectorF32Range(2).map((v) => {
      return makeCase(v[0], v[1], truthFunc);
    });
  },
  less_than_const: () => {
    const truthFunc = (lhs, rhs) => {
      return lhs.value < rhs.value;
    };

    return vectorF32Range(2).map((v) => {
      return makeCase(v[0], v[1], truthFunc);
    });
  },
  less_equals_non_const: () => {
    const truthFunc = (lhs, rhs) => {
      return lhs.value <= rhs.value;
    };

    return vectorF32Range(2).map((v) => {
      return makeCase(v[0], v[1], truthFunc);
    });
  },
  less_equals_const: () => {
    const truthFunc = (lhs, rhs) => {
      return lhs.value <= rhs.value;
    };

    return vectorF32Range(2).map((v) => {
      return makeCase(v[0], v[1], truthFunc);
    });
  },
  greater_than_non_const: () => {
    const truthFunc = (lhs, rhs) => {
      return lhs.value > rhs.value;
    };

    return vectorF32Range(2).map((v) => {
      return makeCase(v[0], v[1], truthFunc);
    });
  },
  greater_than_const: () => {
    const truthFunc = (lhs, rhs) => {
      return lhs.value > rhs.value;
    };

    return vectorF32Range(2).map((v) => {
      return makeCase(v[0], v[1], truthFunc);
    });
  },
  greater_equals_non_const: () => {
    const truthFunc = (lhs, rhs) => {
      return lhs.value >= rhs.value;
    };

    return vectorF32Range(2).map((v) => {
      return makeCase(v[0], v[1], truthFunc);
    });
  },
  greater_equals_const: () => {
    const truthFunc = (lhs, rhs) => {
      return lhs.value >= rhs.value;
    };

    return vectorF32Range(2).map((v) => {
      return makeCase(v[0], v[1], truthFunc);
    });
  }
});