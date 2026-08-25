/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'textureGather';export const description = `
Validation tests for the ${builtin}() builtin.

* test textureGather component parameter must be correct type
* test textureGather component parameter must be between 0 and 3 inclusive
* test textureGather component parameter must be a const expression
* test textureGather coords parameter must be correct type
* test textureGather array_index parameter must be correct type
* test textureGather offset parameter must be correct type
* test textureGather offset parameter must be a const-expression
* test textureGather offset parameter must be between -8 and +7 inclusive
* test textureGather returns the correct type
* test textureGather doesn't work with texture types it's not supposed to
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  kAllScalarsAndVectors,
  isConvertible,


  isUnsignedType,
  scalarTypeOf,
  stringToType } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  getSampleAndBaseTextureTypeForTextureType,
  kTestTextureTypes } from
'./shader_builtin_utils.js';

const kColorSampleTypes = [Type.vec4f, Type.vec4i, Type.vec4u];
const kDepthSampleTypes = [Type.vec4f];









const kValidTextureGatherParameterTypes = {
  texture_2d: {
    sampleTypes: kColorSampleTypes,
    hasComponentArg: true,
    coordsArgType: Type.vec2f,
    offsetArgType: Type.vec2i
  },
  texture_2d_array: {
    sampleTypes: kColorSampleTypes,
    hasComponentArg: true,
    coordsArgType: Type.vec2f,
    hasArrayIndexArg: true,
    offsetArgType: Type.vec2i
  },
  texture_cube: {
    sampleTypes: kColorSampleTypes,
    hasComponentArg: true,
    coordsArgType: Type.vec3f
  },
  texture_cube_array: {
    sampleTypes: kColorSampleTypes,
    hasComponentArg: true,
    coordsArgType: Type.vec3f,
    hasArrayIndexArg: true
  },
  texture_depth_2d: {
    sampleTypes: kDepthSampleTypes,
    coordsArgType: Type.vec2f,
    offsetArgType: Type.vec2i
  },
  texture_depth_2d_array: {
    sampleTypes: kDepthSampleTypes,
    coordsArgType: Type.vec2f,
    hasArrayIndexArg: true,
    offsetArgType: Type.vec2i
  },
  texture_depth_cube: {
    sampleTypes: kDepthSampleTypes,
    coordsArgType: Type.vec3f
  },
  texture_depth_cube_array: {
    sampleTypes: kDepthSampleTypes,
    coordsArgType: Type.vec3f,
    hasArrayIndexArg: true
  }
};

const kTextureTypes = keysOf(kValidTextureGatherParameterTypes);
const kValuesTypes = objectsToRecord(kAllScalarsAndVectors);

export const g = makeTestGroup(ShaderValidationTest);

g.test('return_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturegather').
desc(
  `
Validates the return type of ${builtin} is the expected type.
`
).
params((u) =>
u.
combine('returnType', keysOf(kValuesTypes)).
combine('textureType', keysOf(kValidTextureGatherParameterTypes)).
expand('sampleType', (t) =>
kValidTextureGatherParameterTypes[t.textureType].sampleTypes.map((v) => v.toString())
).
beginSubcases().
expand('offset', (t) =>
kValidTextureGatherParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { returnType, textureType, sampleType, offset } = t.params;
  const returnVarType = kValuesTypes[returnType];
  const sampleVarType = stringToType(sampleType);
  const { hasComponentArg, offsetArgType, coordsArgType, hasArrayIndexArg } =
  kValidTextureGatherParameterTypes[textureType];

  const sampleTypeWGSL = textureType.includes('depth') ? '' : `<${scalarTypeOf(sampleVarType)}>`;
  const componentWGSL = hasComponentArg ? '0, ' : '';
  const varWGSL = returnVarType.toString();
  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType}${sampleTypeWGSL};
@fragment fn fs() -> @location(0) vec4f {
  let v: ${varWGSL} = textureGather(${componentWGSL}t, s, ${coordWGSL}${arrayWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(sampleVarType, returnVarType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('component_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturegather').
desc(
  `
Validates that only incorrect components arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', keysOf(kValidTextureGatherParameterTypes))
// filter out types with no component argument
.filter((t) => !!kValidTextureGatherParameterTypes[t.textureType].hasComponentArg).
combine('componentType', keysOf(kValuesTypes)).
expand('sampleType', (t) =>
kValidTextureGatherParameterTypes[t.textureType].sampleTypes.map((v) => v.toString())
).
beginSubcases().
combine('value', [-1, 0, 1, 2, 3, 4])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.componentType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureGatherParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, componentType, sampleType, offset, value } = t.params;
  const sampleVarType = stringToType(sampleType);
  const componentArgType = kValuesTypes[componentType];
  const { offsetArgType, coordsArgType, hasArrayIndexArg } =
  kValidTextureGatherParameterTypes[textureType];

  const sampleTypeWGSL = textureType.includes('depth') ? '' : `<${scalarTypeOf(sampleVarType)}>`;
  const componentWGSL = componentArgType.create(value).wgsl();
  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType}${sampleTypeWGSL};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureGather(${componentWGSL}, t, s, ${coordWGSL}${arrayWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  (isConvertible(componentArgType, Type.i32) || isConvertible(componentArgType, Type.u32)) &&
  value >= 0 &&
  value <= 3;
  t.expectCompileResult(expectSuccess, code);
});

g.test('component_argument,non_const').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturegather').
desc(
  `
Validates that only non-const components arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', keysOf(kValidTextureGatherParameterTypes))
// filter out types with no component argument
.filter((t) => !!kValidTextureGatherParameterTypes[t.textureType].hasComponentArg).
expand('sampleType', (t) =>
kValidTextureGatherParameterTypes[t.textureType].sampleTypes.map((v) => v.toString())
).
combine('varType', ['c', 'u', 'l']).
beginSubcases().
expand('offset', (t) =>
kValidTextureGatherParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, sampleType, varType, offset } = t.params;
  const sampleVarType = stringToType(sampleType);
  const componentArgType = Type.u32;
  const { coordsArgType, hasArrayIndexArg, offsetArgType } =
  kValidTextureGatherParameterTypes[textureType];

  const sampleTypeWGSL = textureType.includes('depth') ? '' : `<${scalarTypeOf(sampleVarType)}>`;
  const componentWGSL = `${componentArgType}(${varType})`;
  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType}${sampleTypeWGSL};
@group(0) @binding(2) var<uniform> u: ${componentArgType};

@fragment fn fs() -> @location(0) vec4f {
  const c = 1;
  let l = 1;
  let v = textureGather(${componentWGSL}, t, s, ${coordWGSL}${arrayWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = varType === 'c';
  t.expectCompileResult(expectSuccess, code);
});

g.test('coords_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturegather').
desc(
  `
Validates that only incorrect coords arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', keysOf(kValidTextureGatherParameterTypes)).
expand('sampleType', (t) =>
kValidTextureGatherParameterTypes[t.textureType].sampleTypes.map((v) => v.toString())
).
combine('coordType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.coordType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureGatherParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, sampleType, coordType, offset, value } = t.params;
  const sampleVarType = stringToType(sampleType);
  const coordArgType = kValuesTypes[coordType];
  const {
    hasComponentArg,
    offsetArgType,
    coordsArgType: coordsRequiredType,
    hasArrayIndexArg
  } = kValidTextureGatherParameterTypes[textureType];

  const sampleTypeWGSL = textureType.includes('depth') ? '' : `<${scalarTypeOf(sampleVarType)}>`;
  const componentWGSL = hasComponentArg ? '0, ' : '';
  const coordWGSL = coordArgType.create(value).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType}${sampleTypeWGSL};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureGather(${componentWGSL}t, s, ${coordWGSL}${arrayWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(coordArgType, coordsRequiredType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('array_index_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturegather').
desc(
  `
Validates that only incorrect array_index arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes)
// filter out types with no array_index
.filter((t) => !!kValidTextureGatherParameterTypes[t.textureType].hasArrayIndexArg).
expand('sampleType', (t) =>
kValidTextureGatherParameterTypes[t.textureType].sampleTypes.map((v) => v.toString())
).
combine('arrayIndexType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-9, -8, 0, 7, 8])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.arrayIndexType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureGatherParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, sampleType, arrayIndexType, value, offset } = t.params;
  const sampleVarType = stringToType(sampleType);
  const arrayIndexArgType = kValuesTypes[arrayIndexType];
  const args = [arrayIndexArgType.create(value)];
  const { hasComponentArg, coordsArgType, offsetArgType } =
  kValidTextureGatherParameterTypes[textureType];

  const sampleTypeWGSL = textureType.includes('depth') ? '' : `<${scalarTypeOf(sampleVarType)}>`;
  const componentWGSL = hasComponentArg ? '0, ' : '';
  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = args.map((arg) => arg.wgsl()).join(', ');
  const offsetWGSL = offset ? `, ${offsetArgType.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType}${sampleTypeWGSL};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureGather(${componentWGSL}t, s, ${coordWGSL}, ${arrayWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(arrayIndexArgType, Type.i32) || isConvertible(arrayIndexArgType, Type.u32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('offset_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturegather').
desc(
  `
Validates that only incorrect offset arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes)
// filter out types with no offset
.filter((t) => !!kValidTextureGatherParameterTypes[t.textureType].offsetArgType).
expand('sampleType', (t) =>
kValidTextureGatherParameterTypes[t.textureType].sampleTypes.map((v) => v.toString())
).
combine('offsetType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-9, -8, 0, 7, 8])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.offsetType]) || t.value >= 0)
).
fn((t) => {
  const { textureType, sampleType, offsetType, value } = t.params;
  const sampleVarType = stringToType(sampleType);
  const offsetArgType = kValuesTypes[offsetType];
  const args = [offsetArgType.create(value)];
  const {
    hasComponentArg,
    coordsArgType,
    hasArrayIndexArg,
    offsetArgType: offsetRequiredType
  } = kValidTextureGatherParameterTypes[textureType];

  const sampleTypeWGSL = textureType.includes('depth') ? '' : `<${scalarTypeOf(sampleVarType)}>`;
  const componentWGSL = hasComponentArg ? '0, ' : '';
  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = args.map((arg) => arg.wgsl()).join(', ');

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType}${sampleTypeWGSL};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureGather(${componentWGSL}t, s, ${coordWGSL}${arrayWGSL}, ${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(offsetArgType, offsetRequiredType) && value >= -8 && value <= 7;
  t.expectCompileResult(expectSuccess, code);
});

g.test('offset_argument,non_const').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturegather').
desc(
  `
Validates that only non-const offset arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes).
expand('sampleType', (t) =>
kValidTextureGatherParameterTypes[t.textureType].sampleTypes.map((v) => v.toString())
).
combine('varType', ['c', 'u', 'l'])
// filter out types with no offset
.filter((t) => !!kValidTextureGatherParameterTypes[t.textureType].offsetArgType)
).
fn((t) => {
  const { textureType, sampleType, varType } = t.params;
  const sampleVarType = stringToType(sampleType);
  const { hasComponentArg, coordsArgType, hasArrayIndexArg, offsetArgType } =
  kValidTextureGatherParameterTypes[textureType];

  const sampleTypeWGSL = textureType.includes('depth') ? '' : `<${scalarTypeOf(sampleVarType)}>`;
  const componentWGSL = hasComponentArg ? '0, ' : '';
  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = `${offsetArgType}(${varType})`;

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType}${sampleTypeWGSL};
@group(0) @binding(2) var<uniform> u: ${offsetArgType};
@fragment fn fs() -> @location(0) vec4f {
  const c = 1;
  let l = ${offsetArgType.create(0).wgsl()};
  let v = textureGather(${componentWGSL}t, s, ${coordWGSL}${arrayWGSL}, ${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = varType === 'c';
  t.expectCompileResult(expectSuccess, code);
});

g.test('texture_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturegather').
desc(
  `
Validates that incompatible texture types don't work with ${builtin}
`
).
params((u) =>
u.
combine('testTextureType', kTestTextureTypes).
combine('textureType', keysOf(kValidTextureGatherParameterTypes)).
expand('offset', (t) =>
kValidTextureGatherParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { testTextureType, textureType, offset } = t.params;
  const { hasComponentArg, coordsArgType, offsetArgType, hasArrayIndexArg } =
  kValidTextureGatherParameterTypes[textureType];

  const componentWGSL = hasComponentArg ? '0, ' : '';
  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${testTextureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureGather(${componentWGSL}t, s, ${coordWGSL}${arrayWGSL}${offsetWGSL});
  return vec4f(0);
}
`;

  const [baseTestTextureType] = getSampleAndBaseTextureTypeForTextureType(testTextureType);

  const types = kValidTextureGatherParameterTypes[baseTestTextureType];
  const typesMatch = types ?
  types.hasComponentArg === hasComponentArg &&
  types.coordsArgType === coordsArgType &&
  types.hasArrayIndexArg === hasArrayIndexArg && (
  offset ? types.offsetArgType === offsetArgType : true) :
  false;

  const expectSuccess = typesMatch;
  t.expectCompileResult(expectSuccess, code);
});

g.test('must_use').
desc('Tests that the result must be used').
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const code = `
    @group(0) @binding(0) var t : texture_2d<f32>;
    @group(0) @binding(1) var s : sampler;
    fn foo() {
      ${t.params.use ? '_ =' : ''} textureGather(0, t, s, vec2(0,0));
    }`;
  t.expectCompileResult(t.params.use, code);
});