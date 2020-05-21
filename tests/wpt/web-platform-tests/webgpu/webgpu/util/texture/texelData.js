/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { assert, unreachable } from '../../../common/framework/util/util.js';
import { kTextureFormatInfo } from '../../capability_info.js';
import { assertInIntegerRange, float32ToFloatBits, floatAsNormalizedInteger, gammaCompress } from '../conversion.js';
export let TexelComponent;

(function (TexelComponent) {
  TexelComponent["R"] = "R";
  TexelComponent["G"] = "G";
  TexelComponent["B"] = "B";
  TexelComponent["A"] = "A";
  TexelComponent["Depth"] = "Depth";
  TexelComponent["Stencil"] = "Stencil";
})(TexelComponent || (TexelComponent = {}));

var TexelWriteType; // Function to convert a value into a texel value. It returns the converted value
// and the type of the converted value. For example, conversion may convert:
//  - floats to unsigned normalized integers
//  - floats to half floats, interpreted as uint16 bits

(function (TexelWriteType) {
  TexelWriteType[TexelWriteType["Sint"] = 0] = "Sint";
  TexelWriteType[TexelWriteType["Uint"] = 1] = "Uint";
  TexelWriteType[TexelWriteType["Float"] = 2] = "Float";
})(TexelWriteType || (TexelWriteType = {}));

const kR = [TexelComponent.R];
const kRG = [TexelComponent.R, TexelComponent.G];
const kRGB = [TexelComponent.R, TexelComponent.G, TexelComponent.B];
const kRGBA = [TexelComponent.R, TexelComponent.G, TexelComponent.B, TexelComponent.A];
const kBGRA = [TexelComponent.B, TexelComponent.G, TexelComponent.R, TexelComponent.A];

const unorm = bitLength => n => ({
  value: floatAsNormalizedInteger(n, bitLength, false),
  type: TexelWriteType.Uint
});

const snorm = bitLength => n => ({
  value: floatAsNormalizedInteger(n, bitLength, true),
  type: TexelWriteType.Sint
});

const uint = bitLength => n => ({
  value: (assertInIntegerRange(n, bitLength, false), n),
  type: TexelWriteType.Uint
});

const sint = bitLength => n => ({
  value: (assertInIntegerRange(n, bitLength, true), n),
  type: TexelWriteType.Sint
});

const unorm2 = {
  write: unorm(2),
  bitLength: 2
};
const unorm8 = {
  write: unorm(8),
  bitLength: 8
};
const unorm10 = {
  write: unorm(10),
  bitLength: 10
};
const snorm8 = {
  write: snorm(8),
  bitLength: 8
};
const uint8 = {
  write: uint(8),
  bitLength: 8
};
const uint16 = {
  write: uint(16),
  bitLength: 16
};
const uint32 = {
  write: uint(32),
  bitLength: 32
};
const sint8 = {
  write: sint(8),
  bitLength: 8
};
const sint16 = {
  write: sint(16),
  bitLength: 16
};
const sint32 = {
  write: sint(32),
  bitLength: 32
};
const float10 = {
  write: n => ({
    value: float32ToFloatBits(n, 0, 5, 5, 15),
    type: TexelWriteType.Uint
  }),
  bitLength: 10
};
const float11 = {
  write: n => ({
    value: float32ToFloatBits(n, 0, 5, 6, 15),
    type: TexelWriteType.Uint
  }),
  bitLength: 11
};
const float16 = {
  write: n => ({
    value: float32ToFloatBits(n, 1, 5, 10, 15),
    type: TexelWriteType.Uint
  }),
  bitLength: 16
};
const float32 = {
  write: n => ({
    value: Math.fround(n),
    type: TexelWriteType.Float
  }),
  bitLength: 32
};

const repeatComponents = (componentOrder, perComponentInfo) => {
  const componentInfo = componentOrder.reduce((acc, curr) => {
    return Object.assign(acc, {
      [curr]: perComponentInfo
    });
  }, {});
  return {
    componentOrder,
    componentInfo
  };
};

const kRepresentationInfo =
/* prettier-ignore */
{
  'r8unorm': { ...repeatComponents(kR, unorm8),
    sRGB: false
  },
  'r8snorm': { ...repeatComponents(kR, snorm8),
    sRGB: false
  },
  'r8uint': { ...repeatComponents(kR, uint8),
    sRGB: false
  },
  'r8sint': { ...repeatComponents(kR, sint8),
    sRGB: false
  },
  'r16uint': { ...repeatComponents(kR, uint16),
    sRGB: false
  },
  'r16sint': { ...repeatComponents(kR, sint16),
    sRGB: false
  },
  'r16float': { ...repeatComponents(kR, float16),
    sRGB: false
  },
  'rg8unorm': { ...repeatComponents(kRG, unorm8),
    sRGB: false
  },
  'rg8snorm': { ...repeatComponents(kRG, snorm8),
    sRGB: false
  },
  'rg8uint': { ...repeatComponents(kRG, uint8),
    sRGB: false
  },
  'rg8sint': { ...repeatComponents(kRG, sint8),
    sRGB: false
  },
  'r32uint': { ...repeatComponents(kR, uint32),
    sRGB: false
  },
  'r32sint': { ...repeatComponents(kR, sint32),
    sRGB: false
  },
  'r32float': { ...repeatComponents(kR, float32),
    sRGB: false
  },
  'rg16uint': { ...repeatComponents(kRG, uint16),
    sRGB: false
  },
  'rg16sint': { ...repeatComponents(kRG, sint16),
    sRGB: false
  },
  'rg16float': { ...repeatComponents(kRG, float16),
    sRGB: false
  },
  'rgba8unorm': { ...repeatComponents(kRGBA, unorm8),
    sRGB: false
  },
  'rgba8unorm-srgb': { ...repeatComponents(kRGBA, unorm8),
    sRGB: true
  },
  'rgba8snorm': { ...repeatComponents(kRGBA, snorm8),
    sRGB: false
  },
  'rgba8uint': { ...repeatComponents(kRGBA, uint8),
    sRGB: false
  },
  'rgba8sint': { ...repeatComponents(kRGBA, sint8),
    sRGB: false
  },
  'bgra8unorm': { ...repeatComponents(kBGRA, unorm8),
    sRGB: false
  },
  'bgra8unorm-srgb': { ...repeatComponents(kBGRA, unorm8),
    sRGB: true
  },
  'rg32uint': { ...repeatComponents(kRG, uint32),
    sRGB: false
  },
  'rg32sint': { ...repeatComponents(kRG, sint32),
    sRGB: false
  },
  'rg32float': { ...repeatComponents(kRG, float32),
    sRGB: false
  },
  'rgba16uint': { ...repeatComponents(kRGBA, uint16),
    sRGB: false
  },
  'rgba16sint': { ...repeatComponents(kRGBA, sint16),
    sRGB: false
  },
  'rgba16float': { ...repeatComponents(kRGBA, float16),
    sRGB: false
  },
  'rgba32uint': { ...repeatComponents(kRGBA, uint32),
    sRGB: false
  },
  'rgba32sint': { ...repeatComponents(kRGBA, sint32),
    sRGB: false
  },
  'rgba32float': { ...repeatComponents(kRGBA, float32),
    sRGB: false
  },
  'rgb10a2unorm': {
    componentOrder: kRGBA,
    componentInfo: {
      R: unorm10,
      G: unorm10,
      B: unorm10,
      A: unorm2
    },
    sRGB: false
  },
  'rg11b10float': {
    componentOrder: kRGB,
    componentInfo: {
      R: float11,
      G: float11,
      B: float10
    },
    sRGB: false
  },
  'depth32float': {
    componentOrder: [TexelComponent.Depth],
    componentInfo: {
      Depth: float32
    },
    sRGB: false
  },
  'depth24plus': {
    componentOrder: [TexelComponent.Depth],
    componentInfo: {
      Depth: null
    },
    sRGB: false
  },
  'depth24plus-stencil8': {
    componentOrder: [TexelComponent.Depth, TexelComponent.Stencil],
    componentInfo: {
      Depth: null,
      Stencil: null
    },
    sRGB: false
  }
};

class TexelDataRepresentationImpl {
  // TODO: Determine endianness of the GPU data?
  constructor(format, componentOrder, componentInfo, sRGB) {
    this.format = format;
    this.componentOrder = componentOrder;
    this.componentInfo = componentInfo;
    this.sRGB = sRGB;

    _defineProperty(this, "isGPULittleEndian", true);
  }

  totalBitLength() {
    return this.componentOrder.reduce((acc, curr) => {
      return acc + this.componentInfo[curr].bitLength;
    }, 0);
  }

  setComponent(data, component, n) {
    const componentIndex = this.componentOrder.indexOf(component);
    assert(componentIndex !== -1);
    const bitOffset = this.componentOrder.slice(0, componentIndex).reduce((acc, curr) => {
      const componentInfo = this.componentInfo[curr];
      assert(!!componentInfo);
      return acc + componentInfo.bitLength;
    }, 0);
    const componentInfo = this.componentInfo[component];
    assert(!!componentInfo);
    const {
      write,
      bitLength
    } = componentInfo;
    const {
      value,
      type
    } = write(n);

    switch (type) {
      case TexelWriteType.Float:
        {
          const byteOffset = Math.floor(bitOffset / 8);
          const byteLength = Math.ceil(bitLength / 8);
          assert(byteOffset === bitOffset / 8 && byteLength === bitLength / 8);

          switch (byteLength) {
            case 4:
              new DataView(data, byteOffset, byteLength).setFloat32(0, value, this.isGPULittleEndian);
              break;

            default:
              unreachable();
          }

          break;
        }

      case TexelWriteType.Sint:
        {
          const byteOffset = Math.floor(bitOffset / 8);
          const byteLength = Math.ceil(bitLength / 8);
          assert(byteOffset === bitOffset / 8 && byteLength === bitLength / 8);

          switch (byteLength) {
            case 1:
              new DataView(data, byteOffset, byteLength).setInt8(0, value);
              break;

            case 2:
              new DataView(data, byteOffset, byteLength).setInt16(0, value, this.isGPULittleEndian);
              break;

            case 4:
              new DataView(data, byteOffset, byteLength).setInt32(0, value, this.isGPULittleEndian);
              break;

            default:
              unreachable();
          }

          break;
        }

      case TexelWriteType.Uint:
        {
          const byteOffset = Math.floor(bitOffset / 8);
          const byteLength = Math.ceil(bitLength / 8);

          if (byteOffset === bitOffset / 8 && byteLength === bitLength / 8) {
            switch (byteLength) {
              case 1:
                new DataView(data, byteOffset, byteLength).setUint8(0, value);
                break;

              case 2:
                new DataView(data, byteOffset, byteLength).setUint16(0, value, this.isGPULittleEndian);
                break;

              case 4:
                new DataView(data, byteOffset, byteLength).setUint32(0, value, this.isGPULittleEndian);
                break;

              default:
                unreachable();
            }
          } else {
            // Packed representations are all 32-bit and use Uint as the data type.
            // ex.) rg10b11float, rgb10a2unorm
            switch (this.totalBitLength()) {
              case 32:
                {
                  const view = new DataView(data);
                  const currentValue = view.getUint32(0, this.isGPULittleEndian);
                  let mask = 0xffffffff;
                  const bitsToClearRight = bitOffset;
                  const bitsToClearLeft = 32 - (bitLength + bitOffset);
                  mask = mask >>> bitsToClearRight << bitsToClearRight;
                  mask = mask << bitsToClearLeft >>> bitsToClearLeft;
                  const newValue = currentValue & ~mask | value << bitOffset;
                  view.setUint32(0, newValue, this.isGPULittleEndian);
                  break;
                }

              default:
                unreachable();
            }
          }

          break;
        }

      default:
        unreachable();
    }
  }

  getBytes(components) {
    if (this.sRGB) {
      components = Object.assign({}, components);
      assert(components.R !== undefined);
      assert(components.G !== undefined);
      assert(components.B !== undefined);
      [components.R, components.G, components.B] = [gammaCompress(components.R), gammaCompress(components.G), gammaCompress(components.B)];
    }

    const bytesPerBlock = kTextureFormatInfo[this.format].bytesPerBlock;
    assert(!!bytesPerBlock);
    const data = new ArrayBuffer(bytesPerBlock);

    for (const c of this.componentOrder) {
      const componentValue = components[c];
      assert(componentValue !== undefined);
      this.setComponent(data, c, componentValue);
    }

    return data;
  }

}

const kRepresentationCache = new Map();
export function getTexelDataRepresentation(format) {
  if (!kRepresentationCache.has(format)) {
    const {
      componentOrder,
      componentInfo,
      sRGB
    } = kRepresentationInfo[format];
    kRepresentationCache.set(format, new TexelDataRepresentationImpl(format, componentOrder, componentInfo, sRGB));
  }

  return kRepresentationCache.get(format);
}
//# sourceMappingURL=texelData.js.map