/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { Float16Array } from '../../external/petamoriken/float16/float16.js';
export const kBit = {
  // Limits of int32
  i32: {
    positive: {
      min: 0x0000_0000, // 0
      max: 0x7fff_ffff, // 2147483647
    },
    negative: {
      min: 0x8000_0000, // -2147483648
      max: 0x0000_0000, // 0
    },
  },

  // Limits of uint32
  u32: {
    min: 0x0000_0000,
    max: 0xffff_ffff,
  },

  // Limits of f64
  // Have to be stored as a BigInt hex value, since number is a f64 internally,
  // so 64-bit hex values are not guaranteed to be precisely representable.
  f64: {
    positive: {
      min: BigInt(0x0010_0000_0000_0000n),
      max: BigInt(0x7fef_ffff_ffff_ffffn),
      zero: BigInt(0x0000_0000_0000_0000n),
      nearest_max: BigInt(0x7fef_ffff_ffff_fffen),
      less_than_one: BigInt(0x3fef_ffff_ffff_ffffn),
      pi: {
        whole: BigInt(0x4009_21fb_5444_2d18n),
        three_quarters: BigInt(0x4002_d97c_7f33_21d2n),
        half: BigInt(0x3ff9_21fb_5444_2d18n),
        third: BigInt(0x3ff0_c152_382d_7366n),
        quarter: BigInt(0x3fe9_21fb_5444_2d18n),
        sixth: BigInt(0x3fe0_c152_382d_7366n),
      },
      e: BigInt(0x4005_bf0a_8b14_5769n),
    },
    negative: {
      max: BigInt(0x8010_0000_0000_0000n),
      min: BigInt(0xffef_ffff_ffff_ffffn),
      zero: BigInt(0x8000_0000_0000_0000n),
      nearest_min: BigInt(0xffef_ffff_ffff_fffen),
      less_than_one: BigInt(0xbfef_ffff_ffff_ffffn),
      pi: {
        whole: BigInt(0xc009_21fb_5444_2d18n),
        three_quarters: BigInt(0xc002_d97c_7f33_21d2n),
        half: BigInt(0xbff9_21fb_5444_2d18n),
        third: BigInt(0xbff0_c152_382d_7366n),
        quarter: BigInt(0xbfe9_21fb_5444_2d18n),
        sixth: BigInt(0xbfe0_c152_382d_7366n),
      },
    },
    subnormal: {
      positive: {
        min: BigInt(0x0000_0000_0000_0001n),
        max: BigInt(0x000f_ffff_ffff_ffffn),
      },
      negative: {
        max: BigInt(0x8000_0000_0000_0001n),
        min: BigInt(0x800f_ffff_ffff_ffffn),
      },
    },
    infinity: {
      positive: BigInt(0x7ff0_0000_0000_0000n),
      negative: BigInt(0xfff0_0000_0000_0000n),
    },
  },

  // Limits of f32
  f32: {
    positive: {
      min: 0x0080_0000,
      max: 0x7f7f_ffff,
      zero: 0x0000_0000,
      nearest_max: 0x7f7f_fffe,
      less_than_one: 0x3f7f_ffff,
      pi: {
        whole: 0x4049_0fdb,
        three_quarters: 0x4016_cbe4,
        half: 0x3fc9_0fdb,
        third: 0x3f86_0a92,
        quarter: 0x3f49_0fdb,
        sixth: 0x3f06_0a92,
      },
      e: 0x402d_f854,
    },
    negative: {
      max: 0x8080_0000,
      min: 0xff7f_ffff,
      zero: 0x8000_0000,
      nearest_min: 0xff7f_fffe,
      less_than_one: 0xbf7f_ffff,
      pi: {
        whole: 0xc04_90fdb,
        three_quarters: 0xc016_cbe4,
        half: 0xbfc9_0fdb,
        third: 0xbf86_0a92,
        quarter: 0xbf49_0fdb,
        sixth: 0xbf06_0a92,
      },
    },
    subnormal: {
      positive: {
        min: 0x0000_0001,
        max: 0x007f_ffff,
      },
      negative: {
        max: 0x8000_0001,
        min: 0x807f_ffff,
      },
    },
    infinity: {
      positive: 0x7f80_0000,
      negative: 0xff80_0000,
    },
  },

  // Limits of f16
  f16: {
    positive: {
      min: 0x0400,
      max: 0x7bff,
      zero: 0x0000,
      nearest_max: 0x7bfe,
      less_than_one: 0x3bff,
      pi: {
        whole: 0x4248,
        three_quarters: 0x40b6,
        half: 0x3e48,
        third: 0x3c30,
        quarter: 0x3a48,
        sixth: 0x3830,
      },
      e: 0x416f,
    },
    negative: {
      max: 0x8400,
      min: 0xfbff,
      zero: 0x8000,
      nearest_min: 0xfbfe,
      less_than_one: 0xbbff,
      pi: {
        whole: 0xc248,
        three_quarters: 0xc0b6,
        half: 0xbe48,
        third: 0xbc30,
        quarter: 0xba48,
        sixth: 0xb830,
      },
    },
    subnormal: {
      positive: {
        min: 0x0001,
        max: 0x03ff,
      },
      negative: {
        max: 0x8001,
        min: 0x83ff,
      },
    },
    infinity: {
      positive: 0x7c00,
      negative: 0xfc00,
    },
  },

  // 32-bit representation of power(2, n) n = {-31, ..., 31}
  // A uint32 representation as a JS `number`
  // {toMinus31, ..., to31} ie. {-31, ..., 31}
  powTwo: {
    toMinus1: 0x3f00_0000,
    toMinus2: 0x3e80_0000,
    toMinus3: 0x3e00_0000,
    toMinus4: 0x3d80_0000,
    toMinus5: 0x3d00_0000,
    toMinus6: 0x3c80_0000,
    toMinus7: 0x3c00_0000,
    toMinus8: 0x3b80_0000,
    toMinus9: 0x3b00_0000,
    toMinus10: 0x3a80_0000,
    toMinus11: 0x3a00_0000,
    toMinus12: 0x3980_0000,
    toMinus13: 0x3900_0000,
    toMinus14: 0x3880_0000,
    toMinus15: 0x3800_0000,
    toMinus16: 0x3780_0000,
    toMinus17: 0x3700_0000,
    toMinus18: 0x3680_0000,
    toMinus19: 0x3600_0000,
    toMinus20: 0x3580_0000,
    toMinus21: 0x3500_0000,
    toMinus22: 0x3480_0000,
    toMinus23: 0x3400_0000,
    toMinus24: 0x3380_0000,
    toMinus25: 0x3300_0000,
    toMinus26: 0x3280_0000,
    toMinus27: 0x3200_0000,
    toMinus28: 0x3180_0000,
    toMinus29: 0x3100_0000,
    toMinus30: 0x3080_0000,
    toMinus31: 0x3000_0000,

    to0: 0x0000_0001,
    to1: 0x0000_0002,
    to2: 0x0000_0004,
    to3: 0x0000_0008,
    to4: 0x0000_0010,
    to5: 0x0000_0020,
    to6: 0x0000_0040,
    to7: 0x0000_0080,
    to8: 0x0000_0100,
    to9: 0x0000_0200,
    to10: 0x0000_0400,
    to11: 0x0000_0800,
    to12: 0x0000_1000,
    to13: 0x0000_2000,
    to14: 0x0000_4000,
    to15: 0x0000_8000,
    to16: 0x0001_0000,
    to17: 0x0002_0000,
    to18: 0x0004_0000,
    to19: 0x0008_0000,
    to20: 0x0010_0000,
    to21: 0x0020_0000,
    to22: 0x0040_0000,
    to23: 0x0080_0000,
    to24: 0x0100_0000,
    to25: 0x0200_0000,
    to26: 0x0400_0000,
    to27: 0x0800_0000,
    to28: 0x1000_0000,
    to29: 0x2000_0000,
    to30: 0x4000_0000,
    to31: 0x8000_0000,
  },

  // 32-bit representation of  of -1 * power(2, n) n = {-31, ..., 31}
  // An int32 represented as a JS `number`
  // {toMinus31, ..., to31} ie. {-31, ..., 31}
  negPowTwo: {
    toMinus1: 0xbf00_0000,
    toMinus2: 0xbe80_0000,
    toMinus3: 0xbe00_0000,
    toMinus4: 0xbd80_0000,
    toMinus5: 0xbd00_0000,
    toMinus6: 0xbc80_0000,
    toMinus7: 0xbc00_0000,
    toMinus8: 0xbb80_0000,
    toMinus9: 0xbb00_0000,
    toMinus10: 0xba80_0000,
    toMinus11: 0xba00_0000,
    toMinus12: 0xb980_0000,
    toMinus13: 0xb900_0000,
    toMinus14: 0xb880_0000,
    toMinus15: 0xb800_0000,
    toMinus16: 0xb780_0000,
    toMinus17: 0xb700_0000,
    toMinus18: 0xb680_0000,
    toMinus19: 0xb600_0000,
    toMinus20: 0xb580_0000,
    toMinus21: 0xb500_0000,
    toMinus22: 0xb480_0000,
    toMinus23: 0xb400_0000,
    toMinus24: 0xb380_0000,
    toMinus25: 0xb300_0000,
    toMinus26: 0xb280_0000,
    toMinus27: 0xb200_0000,
    toMinus28: 0xb180_0000,
    toMinus29: 0xb100_0000,
    toMinus30: 0xb080_0000,
    toMinus31: 0xb000_0000,

    to0: 0xffff_ffff,
    to1: 0xffff_fffe,
    to2: 0xffff_fffc,
    to3: 0xffff_fff8,
    to4: 0xffff_fff0,
    to5: 0xffff_ffe0,
    to6: 0xffff_ffc0,
    to7: 0xffff_ff80,
    to8: 0xffff_ff00,
    to9: 0xffff_fe00,
    to10: 0xffff_fc00,
    to11: 0xffff_f800,
    to12: 0xffff_f000,
    to13: 0xffff_e000,
    to14: 0xffff_c000,
    to15: 0xffff_8000,
    to16: 0xffff_0000,
    to17: 0xfffe_0000,
    to18: 0xfffc_0000,
    to19: 0xfff8_0000,
    to20: 0xfff0_0000,
    to21: 0xffe0_0000,
    to22: 0xffc0_0000,
    to23: 0xff80_0000,
    to24: 0xff00_0000,
    to25: 0xfe00_0000,
    to26: 0xfc00_0000,
    to27: 0xf800_0000,
    to28: 0xf000_0000,
    to29: 0xe000_0000,
    to30: 0xc000_0000,
    to31: 0x8000_0000,
  },
};

/**
 * @returns a 64-bit float value via interpreting the input as the bit
 * representation as a 64-bit integer
 *
 * Using a locally defined function here to avoid compile time dependency
 * issues.
 */
function reinterpretU64AsF64(input) {
  return new Float64Array(new BigUint64Array([input]).buffer)[0];
}

/**
 * @returns the 64-bit integer bit representation of 64-bit float value
 *
 * Using a locally defined function here to avoid compile time dependency
 * issues.
 */
function reinterpretF64AsU64(input) {
  return new BigUint64Array(new Float64Array([input]).buffer)[0];
}

/**
 * @returns a 32-bit float value via interpreting the input as the bit
 * representation as a 32-bit integer
 *
 * Using a locally defined function here to avoid compile time dependency
 * issues.
 */
function reinterpretU32AsF32(input) {
  return new Float32Array(new Uint32Array([input]).buffer)[0];
}

/**
 * @returns a 16-bit float value via interpreting the input as the bit
 * representation as a 64-bit integer
 *
 * Using a locally defined function here to avoid compile time dependency
 * issues.
 */
function reinterpretU16AsF16(input) {
  return new Float16Array(new Uint16Array([input]).buffer)[0];
}

export const kValue = {
  // Limits of i32
  i32: {
    positive: {
      min: 0,
      max: 2147483647,
    },
    negative: {
      min: -2147483648,
      max: 0,
    },
  },

  // Limits of u32
  u32: {
    min: 0,
    max: 4294967295,
  },

  // Limits of f64
  f64: {
    positive: {
      min: reinterpretU64AsF64(kBit.f64.positive.min),
      max: reinterpretU64AsF64(kBit.f64.positive.max),
      nearest_max: reinterpretU64AsF64(kBit.f64.positive.nearest_max),
      less_than_one: reinterpretU64AsF64(kBit.f64.positive.less_than_one),
      pi: {
        whole: reinterpretU64AsF64(kBit.f64.positive.pi.whole),
        three_quarters: reinterpretU64AsF64(kBit.f64.positive.pi.three_quarters),
        half: reinterpretU64AsF64(kBit.f64.positive.pi.half),
        third: reinterpretU64AsF64(kBit.f64.positive.pi.third),
        quarter: reinterpretU64AsF64(kBit.f64.positive.pi.quarter),
        sixth: reinterpretU64AsF64(kBit.f64.positive.pi.sixth),
      },
      e: reinterpretU64AsF64(kBit.f64.positive.e),
    },
    negative: {
      max: reinterpretU64AsF64(kBit.f64.negative.max),
      min: reinterpretU64AsF64(kBit.f64.negative.min),
      nearest_min: reinterpretU64AsF64(kBit.f64.negative.nearest_min),
      less_than_one: reinterpretU64AsF64(kBit.f64.negative.less_than_one), // -0.999999940395
      pi: {
        whole: reinterpretU64AsF64(kBit.f64.negative.pi.whole),
        three_quarters: reinterpretU64AsF64(kBit.f64.negative.pi.three_quarters),
        half: reinterpretU64AsF64(kBit.f64.negative.pi.half),
        third: reinterpretU64AsF64(kBit.f64.negative.pi.third),
        quarter: reinterpretU64AsF64(kBit.f64.negative.pi.quarter),
        sixth: reinterpretU64AsF64(kBit.f64.negative.pi.sixth),
      },
    },
    subnormal: {
      positive: {
        min: reinterpretU64AsF64(kBit.f64.subnormal.positive.min),
        max: reinterpretU64AsF64(kBit.f64.subnormal.positive.max),
      },
      negative: {
        max: reinterpretU64AsF64(kBit.f64.subnormal.negative.max),
        min: reinterpretU64AsF64(kBit.f64.subnormal.negative.min),
      },
    },
    infinity: {
      positive: reinterpretU64AsF64(kBit.f64.infinity.positive),
      negative: reinterpretU64AsF64(kBit.f64.infinity.negative),
    },
  },

  // Limits of f32
  f32: {
    positive: {
      min: reinterpretU32AsF32(kBit.f32.positive.min),
      max: reinterpretU32AsF32(kBit.f32.positive.max),
      nearest_max: reinterpretU32AsF32(kBit.f32.positive.nearest_max),
      less_than_one: reinterpretU32AsF32(kBit.f32.positive.less_than_one),
      pi: {
        whole: reinterpretU32AsF32(kBit.f32.positive.pi.whole),
        three_quarters: reinterpretU32AsF32(kBit.f32.positive.pi.three_quarters),
        half: reinterpretU32AsF32(kBit.f32.positive.pi.half),
        third: reinterpretU32AsF32(kBit.f32.positive.pi.third),
        quarter: reinterpretU32AsF32(kBit.f32.positive.pi.quarter),
        sixth: reinterpretU32AsF32(kBit.f32.positive.pi.sixth),
      },
      e: reinterpretU32AsF32(kBit.f32.positive.e),
      // The positive pipeline-overridable constant with the smallest magnitude
      // which when cast to f32 will produce infinity. This comes from WGSL
      // conversion rules and the rounding rules of WebIDL.
      first_non_castable_pipeline_override:
        reinterpretU32AsF32(kBit.f32.positive.max) / 2 + 2 ** 127,
      // The positive pipeline-overridable constant with the largest magnitude
      // which when cast to f32 will not produce infinity. This comes from WGSL
      // conversion rules and the rounding rules of WebIDL
      last_castable_pipeline_override: reinterpretU64AsF64(
        reinterpretF64AsU64(reinterpretU32AsF32(kBit.f32.positive.max) / 2 + 2 ** 127) - BigInt(1)
      ),
    },
    negative: {
      max: reinterpretU32AsF32(kBit.f32.negative.max),
      min: reinterpretU32AsF32(kBit.f32.negative.min),
      nearest_min: reinterpretU32AsF32(kBit.f32.negative.nearest_min),
      less_than_one: reinterpretU32AsF32(kBit.f32.negative.less_than_one), // -0.999999940395
      pi: {
        whole: reinterpretU32AsF32(kBit.f32.negative.pi.whole),
        three_quarters: reinterpretU32AsF32(kBit.f32.negative.pi.three_quarters),
        half: reinterpretU32AsF32(kBit.f32.negative.pi.half),
        third: reinterpretU32AsF32(kBit.f32.negative.pi.third),
        quarter: reinterpretU32AsF32(kBit.f32.negative.pi.quarter),
        sixth: reinterpretU32AsF32(kBit.f32.negative.pi.sixth),
      },
      // The negative pipeline-overridable constant with the smallest magnitude
      // which when cast to f32 will produce infinity. This comes from WGSL
      // conversion rules and the rounding rules of WebIDL.
      first_non_castable_pipeline_override: -(
        reinterpretU32AsF32(kBit.f32.positive.max) / 2 +
        2 ** 127
      ),

      // The negative pipeline-overridable constant with the largest magnitude
      // which when cast to f32 will not produce infinity. This comes from WGSL
      // conversion rules and the rounding rules of WebIDL.
      last_castable_pipeline_override: -reinterpretU64AsF64(
        reinterpretF64AsU64(reinterpretU32AsF32(kBit.f32.positive.max) / 2 + 2 ** 127) - BigInt(1)
      ),
    },
    subnormal: {
      positive: {
        min: reinterpretU32AsF32(kBit.f32.subnormal.positive.min),
        max: reinterpretU32AsF32(kBit.f32.subnormal.positive.max),
      },
      negative: {
        max: reinterpretU32AsF32(kBit.f32.subnormal.negative.max),
        min: reinterpretU32AsF32(kBit.f32.subnormal.negative.min),
      },
    },
    infinity: {
      positive: reinterpretU32AsF32(kBit.f32.infinity.positive),
      negative: reinterpretU32AsF32(kBit.f32.infinity.negative),
    },
  },

  // Limits of i16
  i16: {
    positive: {
      min: 0,
      max: 32767,
    },
    negative: {
      min: -32768,
      max: 0,
    },
  },

  // Limits of u16
  u16: {
    min: 0,
    max: 65535,
  },

  // Limits of f16
  f16: {
    positive: {
      min: reinterpretU16AsF16(kBit.f16.positive.min),
      max: reinterpretU16AsF16(kBit.f16.positive.max),
      zero: reinterpretU16AsF16(kBit.f16.positive.zero),
      nearest_max: reinterpretU16AsF16(kBit.f16.positive.nearest_max),
      less_than_one: reinterpretU16AsF16(kBit.f16.positive.less_than_one),
      pi: {
        whole: reinterpretU16AsF16(kBit.f16.positive.pi.whole),
        three_quarters: reinterpretU16AsF16(kBit.f16.positive.pi.three_quarters),
        half: reinterpretU16AsF16(kBit.f16.positive.pi.half),
        third: reinterpretU16AsF16(kBit.f16.positive.pi.third),
        quarter: reinterpretU16AsF16(kBit.f16.positive.pi.quarter),
        sixth: reinterpretU16AsF16(kBit.f16.positive.pi.sixth),
      },
      e: reinterpretU16AsF16(kBit.f16.positive.e),
      // The positive pipeline-overridable constant with the smallest magnitude
      // which when cast to f16 will produce infinity. This comes from WGSL
      // conversion rules and the rounding rules of WebIDL.
      first_non_castable_pipeline_override:
        reinterpretU16AsF16(kBit.f16.positive.max) / 2 + 2 ** 15,
      // The positive pipeline-overridable constant with the largest magnitude
      // which when cast to f16 will not produce infinity. This comes from WGSL
      // conversion rules and the rounding rules of WebIDL
      last_castable_pipeline_override: reinterpretU64AsF64(
        reinterpretF64AsU64(reinterpretU16AsF16(kBit.f16.positive.max) / 2 + 2 ** 15) - BigInt(1)
      ),
    },
    negative: {
      max: reinterpretU16AsF16(kBit.f16.negative.max),
      min: reinterpretU16AsF16(kBit.f16.negative.min),
      zero: reinterpretU16AsF16(kBit.f16.negative.zero),
      nearest_min: reinterpretU16AsF16(kBit.f16.negative.nearest_min),
      less_than_one: reinterpretU16AsF16(kBit.f16.negative.less_than_one), // -0.9996
      pi: {
        whole: reinterpretU16AsF16(kBit.f16.negative.pi.whole),
        three_quarters: reinterpretU16AsF16(kBit.f16.negative.pi.three_quarters),
        half: reinterpretU16AsF16(kBit.f16.negative.pi.half),
        third: reinterpretU16AsF16(kBit.f16.negative.pi.third),
        quarter: reinterpretU16AsF16(kBit.f16.negative.pi.quarter),
        sixth: reinterpretU16AsF16(kBit.f16.negative.pi.sixth),
      },
      // The negative pipeline-overridable constant with the smallest magnitude
      // which when cast to f16 will produce infinity. This comes from WGSL
      // conversion rules and the rounding rules of WebIDL.
      first_non_castable_pipeline_override: -(
        reinterpretU16AsF16(kBit.f16.positive.max) / 2 +
        2 ** 15
      ),

      // The negative pipeline-overridable constant with the largest magnitude
      // which when cast to f16 will not produce infinity. This comes from WGSL
      // conversion rules and the rounding rules of WebIDL.
      last_castable_pipeline_override: -reinterpretU64AsF64(
        reinterpretF64AsU64(reinterpretU16AsF16(kBit.f16.positive.max) / 2 + 2 ** 15) - BigInt(1)
      ),
    },
    subnormal: {
      positive: {
        min: reinterpretU16AsF16(kBit.f16.subnormal.positive.min),
        max: reinterpretU16AsF16(kBit.f16.subnormal.positive.max),
      },
      negative: {
        max: reinterpretU16AsF16(kBit.f16.subnormal.negative.max),
        min: reinterpretU16AsF16(kBit.f16.subnormal.negative.min),
      },
    },
    infinity: {
      positive: reinterpretU16AsF16(kBit.f16.infinity.positive),
      negative: reinterpretU16AsF16(kBit.f16.infinity.negative),
    },
  },

  powTwo: {
    to0: Math.pow(2, 0),
    to1: Math.pow(2, 1),
    to2: Math.pow(2, 2),
    to3: Math.pow(2, 3),
    to4: Math.pow(2, 4),
    to5: Math.pow(2, 5),
    to6: Math.pow(2, 6),
    to7: Math.pow(2, 7),
    to8: Math.pow(2, 8),
    to9: Math.pow(2, 9),
    to10: Math.pow(2, 10),
    to11: Math.pow(2, 11),
    to12: Math.pow(2, 12),
    to13: Math.pow(2, 13),
    to14: Math.pow(2, 14),
    to15: Math.pow(2, 15),
    to16: Math.pow(2, 16),
    to17: Math.pow(2, 17),
    to18: Math.pow(2, 18),
    to19: Math.pow(2, 19),
    to20: Math.pow(2, 20),
    to21: Math.pow(2, 21),
    to22: Math.pow(2, 22),
    to23: Math.pow(2, 23),
    to24: Math.pow(2, 24),
    to25: Math.pow(2, 25),
    to26: Math.pow(2, 26),
    to27: Math.pow(2, 27),
    to28: Math.pow(2, 28),
    to29: Math.pow(2, 29),
    to30: Math.pow(2, 30),
    to31: Math.pow(2, 31),
    to32: Math.pow(2, 32),

    toMinus1: Math.pow(2, -1),
    toMinus2: Math.pow(2, -2),
    toMinus3: Math.pow(2, -3),
    toMinus4: Math.pow(2, -4),
    toMinus5: Math.pow(2, -5),
    toMinus6: Math.pow(2, -6),
    toMinus7: Math.pow(2, -7),
    toMinus8: Math.pow(2, -8),
    toMinus9: Math.pow(2, -9),
    toMinus10: Math.pow(2, -10),
    toMinus11: Math.pow(2, -11),
    toMinus12: Math.pow(2, -12),
    toMinus13: Math.pow(2, -13),
    toMinus14: Math.pow(2, -14),
    toMinus15: Math.pow(2, -15),
    toMinus16: Math.pow(2, -16),
    toMinus17: Math.pow(2, -17),
    toMinus18: Math.pow(2, -18),
    toMinus19: Math.pow(2, -19),
    toMinus20: Math.pow(2, -20),
    toMinus21: Math.pow(2, -21),
    toMinus22: Math.pow(2, -22),
    toMinus23: Math.pow(2, -23),
    toMinus24: Math.pow(2, -24),
    toMinus25: Math.pow(2, -25),
    toMinus26: Math.pow(2, -26),
    toMinus27: Math.pow(2, -27),
    toMinus28: Math.pow(2, -28),
    toMinus29: Math.pow(2, -29),
    toMinus30: Math.pow(2, -30),
    toMinus31: Math.pow(2, -31),
    toMinus32: Math.pow(2, -32),
  },
  negPowTwo: {
    to0: -Math.pow(2, 0),
    to1: -Math.pow(2, 1),
    to2: -Math.pow(2, 2),
    to3: -Math.pow(2, 3),
    to4: -Math.pow(2, 4),
    to5: -Math.pow(2, 5),
    to6: -Math.pow(2, 6),
    to7: -Math.pow(2, 7),
    to8: -Math.pow(2, 8),
    to9: -Math.pow(2, 9),
    to10: -Math.pow(2, 10),
    to11: -Math.pow(2, 11),
    to12: -Math.pow(2, 12),
    to13: -Math.pow(2, 13),
    to14: -Math.pow(2, 14),
    to15: -Math.pow(2, 15),
    to16: -Math.pow(2, 16),
    to17: -Math.pow(2, 17),
    to18: -Math.pow(2, 18),
    to19: -Math.pow(2, 19),
    to20: -Math.pow(2, 20),
    to21: -Math.pow(2, 21),
    to22: -Math.pow(2, 22),
    to23: -Math.pow(2, 23),
    to24: -Math.pow(2, 24),
    to25: -Math.pow(2, 25),
    to26: -Math.pow(2, 26),
    to27: -Math.pow(2, 27),
    to28: -Math.pow(2, 28),
    to29: -Math.pow(2, 29),
    to30: -Math.pow(2, 30),
    to31: -Math.pow(2, 31),
    to32: -Math.pow(2, 32),

    toMinus1: -Math.pow(2, -1),
    toMinus2: -Math.pow(2, -2),
    toMinus3: -Math.pow(2, -3),
    toMinus4: -Math.pow(2, -4),
    toMinus5: -Math.pow(2, -5),
    toMinus6: -Math.pow(2, -6),
    toMinus7: -Math.pow(2, -7),
    toMinus8: -Math.pow(2, -8),
    toMinus9: -Math.pow(2, -9),
    toMinus10: -Math.pow(2, -10),
    toMinus11: -Math.pow(2, -11),
    toMinus12: -Math.pow(2, -12),
    toMinus13: -Math.pow(2, -13),
    toMinus14: -Math.pow(2, -14),
    toMinus15: -Math.pow(2, -15),
    toMinus16: -Math.pow(2, -16),
    toMinus17: -Math.pow(2, -17),
    toMinus18: -Math.pow(2, -18),
    toMinus19: -Math.pow(2, -19),
    toMinus20: -Math.pow(2, -20),
    toMinus21: -Math.pow(2, -21),
    toMinus22: -Math.pow(2, -22),
    toMinus23: -Math.pow(2, -23),
    toMinus24: -Math.pow(2, -24),
    toMinus25: -Math.pow(2, -25),
    toMinus26: -Math.pow(2, -26),
    toMinus27: -Math.pow(2, -27),
    toMinus28: -Math.pow(2, -28),
    toMinus29: -Math.pow(2, -29),
    toMinus30: -Math.pow(2, -30),
    toMinus31: -Math.pow(2, -31),
    toMinus32: -Math.pow(2, -32),
  },

  // Limits of i8
  i8: {
    positive: {
      min: 0,
      max: 127,
    },
    negative: {
      min: -128,
      max: 0,
    },
  },

  // Limits of u8
  u8: {
    min: 0,
    max: 255,
  },
};
