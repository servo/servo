/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { reinterpretU64AsF64, reinterpretF64AsU64,
  reinterpretU32AsF32,
  reinterpretU16AsF16 } from
'./reinterpret.js';

export const kBit = {
  // Limits of int32
  i32: {
    positive: {
      min: 0x0000_0000, // 0
      max: 0x7fff_ffff // 2147483647
    },
    negative: {
      min: 0x8000_0000, // -2147483648
      max: 0x0000_0000 // 0
    }
  },

  // Limits of uint32
  u32: {
    min: 0x0000_0000,
    max: 0xffff_ffff
  },

  // Limits of f64
  // Have to be stored as a BigInt hex value, since number is a f64 internally,
  // so 64-bit hex values are not guaranteed to be precisely representable.
  f64: {
    positive: {
      min: BigInt(0x0010_0000_0000_0000n),
      max: BigInt(0x7fef_ffff_ffff_ffffn),
      zero: BigInt(0x0000_0000_0000_0000n),
      subnormal: {
        min: BigInt(0x0000_0000_0000_0001n),
        max: BigInt(0x000f_ffff_ffff_ffffn)
      },
      infinity: BigInt(0x7ff0_0000_0000_0000n),
      nearest_max: BigInt(0x7fef_ffff_ffff_fffen),
      less_than_one: BigInt(0x3fef_ffff_ffff_ffffn),
      pi: {
        whole: BigInt(0x4009_21fb_5444_2d18n),
        three_quarters: BigInt(0x4002_d97c_7f33_21d2n),
        half: BigInt(0x3ff9_21fb_5444_2d18n),
        third: BigInt(0x3ff0_c152_382d_7365n),
        quarter: BigInt(0x3fe9_21fb_5444_2d18n),
        sixth: BigInt(0x3fe0_c152_382d_7365n)
      },
      e: BigInt(0x4005_bf0a_8b14_5769n)
    },
    negative: {
      max: BigInt(0x8010_0000_0000_0000n),
      min: BigInt(0xffef_ffff_ffff_ffffn),
      zero: BigInt(0x8000_0000_0000_0000n),
      subnormal: {
        max: BigInt(0x8000_0000_0000_0001n),
        min: BigInt(0x800f_ffff_ffff_ffffn)
      },
      infinity: BigInt(0xfff0_0000_0000_0000n),
      nearest_min: BigInt(0xffef_ffff_ffff_fffen),
      less_than_one: BigInt(0xbfef_ffff_ffff_ffffn),
      pi: {
        whole: BigInt(0xc009_21fb_5444_2d18n),
        three_quarters: BigInt(0xc002_d97c_7f33_21d2n),
        half: BigInt(0xbff9_21fb_5444_2d18n),
        third: BigInt(0xbff0_c152_382d_7365n),
        quarter: BigInt(0xbfe9_21fb_5444_2d18n),
        sixth: BigInt(0xbfe0_c152_382d_7365n)
      }
    },
    max_ulp: BigInt(0x7ca0_0000_0000_0000n)
  },

  // Limits of f32
  f32: {
    positive: {
      min: 0x0080_0000,
      max: 0x7f7f_ffff,
      zero: 0x0000_0000,
      subnormal: {
        min: 0x0000_0001,
        max: 0x007f_ffff
      },
      infinity: 0x7f80_0000,
      nearest_max: 0x7f7f_fffe,
      less_than_one: 0x3f7f_ffff,
      pi: {
        whole: 0x4049_0fdb,
        three_quarters: 0x4016_cbe4,
        half: 0x3fc9_0fdb,
        third: 0x3f86_0a92,
        quarter: 0x3f49_0fdb,
        sixth: 0x3f06_0a92
      },
      e: 0x402d_f854
    },
    negative: {
      max: 0x8080_0000,
      min: 0xff7f_ffff,
      zero: 0x8000_0000,
      subnormal: {
        max: 0x8000_0001,
        min: 0x807f_ffff
      },
      infinity: 0xff80_0000,
      nearest_min: 0xff7f_fffe,
      less_than_one: 0xbf7f_ffff,
      pi: {
        whole: 0xc04_90fdb,
        three_quarters: 0xc016_cbe4,
        half: 0xbfc9_0fdb,
        third: 0xbf86_0a92,
        quarter: 0xbf49_0fdb,
        sixth: 0xbf06_0a92
      }
    },
    max_ulp: 0x7380_0000
  },

  // Limits of f16
  f16: {
    positive: {
      min: 0x0400,
      max: 0x7bff,
      zero: 0x0000,
      subnormal: {
        min: 0x0001,
        max: 0x03ff
      },
      infinity: 0x7c00,
      nearest_max: 0x7bfe,
      less_than_one: 0x3bff,
      pi: {
        whole: 0x4248,
        three_quarters: 0x40b6,
        half: 0x3e48,
        third: 0x3c30,
        quarter: 0x3a48,
        sixth: 0x3830
      },
      e: 0x416f
    },
    negative: {
      max: 0x8400,
      min: 0xfbff,
      zero: 0x8000,
      subnormal: {
        max: 0x8001,
        min: 0x83ff
      },
      infinity: 0xfc00,
      nearest_min: 0xfbfe,
      less_than_one: 0xbbff,
      pi: {
        whole: 0xc248,
        three_quarters: 0xc0b6,
        half: 0xbe48,
        third: 0xbc30,
        quarter: 0xba48,
        sixth: 0xb830
      }
    },
    max_ulp: 0x5000
  },

  // Uint32 representation of power(2, n) n = {0, ..., 31}
  // Stored as a JS `number`
  // {to0, ..., to31} ie. {0, ..., 31}
  powTwo: {
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
    to31: 0x8000_0000
  },

  // Int32 representation of  of -1 * power(2, n) n = {0, ..., 31}
  // Stored as a JS `number`
  // {to0, ..., to31} ie. {0, ..., 31}
  negPowTwo: {
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
    to31: 0x8000_0000
  }
};

export const kValue = {
  // Limits of i32
  i32: {
    positive: {
      min: 0,
      max: 2147483647
    },
    negative: {
      min: -2147483648,
      max: 0
    }
  },

  // Limits of u32
  u32: {
    min: 0,
    max: 4294967295
  },

  // Limits of f64
  f64: {
    positive: {
      min: reinterpretU64AsF64(kBit.f64.positive.min),
      max: reinterpretU64AsF64(kBit.f64.positive.max),
      zero: reinterpretU64AsF64(kBit.f64.positive.zero),
      subnormal: {
        min: reinterpretU64AsF64(kBit.f64.positive.subnormal.min),
        max: reinterpretU64AsF64(kBit.f64.positive.subnormal.max)
      },
      infinity: reinterpretU64AsF64(kBit.f64.positive.infinity),
      nearest_max: reinterpretU64AsF64(kBit.f64.positive.nearest_max),
      less_than_one: reinterpretU64AsF64(kBit.f64.positive.less_than_one),
      pi: {
        whole: reinterpretU64AsF64(kBit.f64.positive.pi.whole),
        three_quarters: reinterpretU64AsF64(kBit.f64.positive.pi.three_quarters),
        half: reinterpretU64AsF64(kBit.f64.positive.pi.half),
        third: reinterpretU64AsF64(kBit.f64.positive.pi.third),
        quarter: reinterpretU64AsF64(kBit.f64.positive.pi.quarter),
        sixth: reinterpretU64AsF64(kBit.f64.positive.pi.sixth)
      },
      e: reinterpretU64AsF64(kBit.f64.positive.e)
    },
    negative: {
      max: reinterpretU64AsF64(kBit.f64.negative.max),
      min: reinterpretU64AsF64(kBit.f64.negative.min),
      zero: reinterpretU64AsF64(kBit.f64.negative.zero),
      subnormal: {
        max: reinterpretU64AsF64(kBit.f64.negative.subnormal.max),
        min: reinterpretU64AsF64(kBit.f64.negative.subnormal.min)
      },
      infinity: reinterpretU64AsF64(kBit.f64.negative.infinity),
      nearest_min: reinterpretU64AsF64(kBit.f64.negative.nearest_min),
      less_than_one: reinterpretU64AsF64(kBit.f64.negative.less_than_one), // -0.999999940395
      pi: {
        whole: reinterpretU64AsF64(kBit.f64.negative.pi.whole),
        three_quarters: reinterpretU64AsF64(kBit.f64.negative.pi.three_quarters),
        half: reinterpretU64AsF64(kBit.f64.negative.pi.half),
        third: reinterpretU64AsF64(kBit.f64.negative.pi.third),
        quarter: reinterpretU64AsF64(kBit.f64.negative.pi.quarter),
        sixth: reinterpretU64AsF64(kBit.f64.negative.pi.sixth)
      }
    },
    max_ulp: reinterpretU64AsF64(kBit.f64.max_ulp)
  },

  // Limits of f32
  f32: {
    positive: {
      min: reinterpretU32AsF32(kBit.f32.positive.min),
      max: reinterpretU32AsF32(kBit.f32.positive.max),
      zero: reinterpretU32AsF32(kBit.f32.positive.zero),
      subnormal: {
        min: reinterpretU32AsF32(kBit.f32.positive.subnormal.min),
        max: reinterpretU32AsF32(kBit.f32.positive.subnormal.max)
      },
      infinity: reinterpretU32AsF32(kBit.f32.positive.infinity),

      nearest_max: reinterpretU32AsF32(kBit.f32.positive.nearest_max),
      less_than_one: reinterpretU32AsF32(kBit.f32.positive.less_than_one),
      pi: {
        whole: reinterpretU32AsF32(kBit.f32.positive.pi.whole),
        three_quarters: reinterpretU32AsF32(kBit.f32.positive.pi.three_quarters),
        half: reinterpretU32AsF32(kBit.f32.positive.pi.half),
        third: reinterpretU32AsF32(kBit.f32.positive.pi.third),
        quarter: reinterpretU32AsF32(kBit.f32.positive.pi.quarter),
        sixth: reinterpretU32AsF32(kBit.f32.positive.pi.sixth)
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
      )
    },
    negative: {
      max: reinterpretU32AsF32(kBit.f32.negative.max),
      min: reinterpretU32AsF32(kBit.f32.negative.min),
      zero: reinterpretU32AsF32(kBit.f32.negative.zero),
      subnormal: {
        max: reinterpretU32AsF32(kBit.f32.negative.subnormal.max),
        min: reinterpretU32AsF32(kBit.f32.negative.subnormal.min)
      },
      infinity: reinterpretU32AsF32(kBit.f32.negative.infinity),
      nearest_min: reinterpretU32AsF32(kBit.f32.negative.nearest_min),
      less_than_one: reinterpretU32AsF32(kBit.f32.negative.less_than_one), // -0.999999940395
      pi: {
        whole: reinterpretU32AsF32(kBit.f32.negative.pi.whole),
        three_quarters: reinterpretU32AsF32(kBit.f32.negative.pi.three_quarters),
        half: reinterpretU32AsF32(kBit.f32.negative.pi.half),
        third: reinterpretU32AsF32(kBit.f32.negative.pi.third),
        quarter: reinterpretU32AsF32(kBit.f32.negative.pi.quarter),
        sixth: reinterpretU32AsF32(kBit.f32.negative.pi.sixth)
      },
      // The negative pipeline-overridable constant with the smallest magnitude
      // which when cast to f32 will produce infinity. This comes from WGSL
      // conversion rules and the rounding rules of WebIDL.
      first_non_castable_pipeline_override: -(
      reinterpretU32AsF32(kBit.f32.positive.max) / 2 +
      2 ** 127),

      // The negative pipeline-overridable constant with the largest magnitude
      // which when cast to f32 will not produce infinity. This comes from WGSL
      // conversion rules and the rounding rules of WebIDL.
      last_castable_pipeline_override: -reinterpretU64AsF64(
        reinterpretF64AsU64(reinterpretU32AsF32(kBit.f32.positive.max) / 2 + 2 ** 127) - BigInt(1)
      )
    },
    max_ulp: reinterpretU32AsF32(kBit.f32.max_ulp),
    emax: 127
  },

  // Limits of i16
  i16: {
    positive: {
      min: 0,
      max: 32767
    },
    negative: {
      min: -32768,
      max: 0
    }
  },

  // Limits of u16
  u16: {
    min: 0,
    max: 65535
  },

  // Limits of f16
  f16: {
    positive: {
      min: reinterpretU16AsF16(kBit.f16.positive.min),
      max: reinterpretU16AsF16(kBit.f16.positive.max),
      zero: reinterpretU16AsF16(kBit.f16.positive.zero),
      subnormal: {
        min: reinterpretU16AsF16(kBit.f16.positive.subnormal.min),
        max: reinterpretU16AsF16(kBit.f16.positive.subnormal.max)
      },
      infinity: reinterpretU16AsF16(kBit.f16.positive.infinity),
      nearest_max: reinterpretU16AsF16(kBit.f16.positive.nearest_max),
      less_than_one: reinterpretU16AsF16(kBit.f16.positive.less_than_one),
      pi: {
        whole: reinterpretU16AsF16(kBit.f16.positive.pi.whole),
        three_quarters: reinterpretU16AsF16(kBit.f16.positive.pi.three_quarters),
        half: reinterpretU16AsF16(kBit.f16.positive.pi.half),
        third: reinterpretU16AsF16(kBit.f16.positive.pi.third),
        quarter: reinterpretU16AsF16(kBit.f16.positive.pi.quarter),
        sixth: reinterpretU16AsF16(kBit.f16.positive.pi.sixth)
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
      )
    },
    negative: {
      max: reinterpretU16AsF16(kBit.f16.negative.max),
      min: reinterpretU16AsF16(kBit.f16.negative.min),
      zero: reinterpretU16AsF16(kBit.f16.negative.zero),
      subnormal: {
        max: reinterpretU16AsF16(kBit.f16.negative.subnormal.max),
        min: reinterpretU16AsF16(kBit.f16.negative.subnormal.min)
      },
      infinity: reinterpretU16AsF16(kBit.f16.negative.infinity),
      nearest_min: reinterpretU16AsF16(kBit.f16.negative.nearest_min),
      less_than_one: reinterpretU16AsF16(kBit.f16.negative.less_than_one), // -0.9996
      pi: {
        whole: reinterpretU16AsF16(kBit.f16.negative.pi.whole),
        three_quarters: reinterpretU16AsF16(kBit.f16.negative.pi.three_quarters),
        half: reinterpretU16AsF16(kBit.f16.negative.pi.half),
        third: reinterpretU16AsF16(kBit.f16.negative.pi.third),
        quarter: reinterpretU16AsF16(kBit.f16.negative.pi.quarter),
        sixth: reinterpretU16AsF16(kBit.f16.negative.pi.sixth)
      },
      // The negative pipeline-overridable constant with the smallest magnitude
      // which when cast to f16 will produce infinity. This comes from WGSL
      // conversion rules and the rounding rules of WebIDL.
      first_non_castable_pipeline_override: -(
      reinterpretU16AsF16(kBit.f16.positive.max) / 2 +
      2 ** 15),

      // The negative pipeline-overridable constant with the largest magnitude
      // which when cast to f16 will not produce infinity. This comes from WGSL
      // conversion rules and the rounding rules of WebIDL.
      last_castable_pipeline_override: -reinterpretU64AsF64(
        reinterpretF64AsU64(reinterpretU16AsF16(kBit.f16.positive.max) / 2 + 2 ** 15) - BigInt(1)
      )
    },
    max_ulp: reinterpretU16AsF16(kBit.f16.max_ulp),
    emax: 15
  },

  // Limits of i8
  i8: {
    positive: {
      min: 0,
      max: 127
    },
    negative: {
      min: -128,
      max: 0
    }
  },

  // Limits of u8
  u8: {
    min: 0,
    max: 255
  }
};