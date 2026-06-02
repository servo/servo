/*
Copyright (c) 2022 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

async function testNonReservedWords(part, numParts) {

  description(`test shaders using reserved words as identifiers compile ${part} of ${numParts}`);

  const DXWords = [
    "Buffer",
    "double",
    "uint",
    "half",
    "dword",
    "string",
    "texture",
    "pixelshader",
    "vertexshader",
    "switch",
    "min16float",
    "min10float",
    "min16int",
    "min12int",
    "min16uint",
    "vector",
    "matrix",
    "float2",
    "float3",
    "float4",
    "float1x1",
    "float1x2",
    "float1x3",
    "float1x4",
    "float2x1",
    "float2x2",
    "float2x3",
    "float2x4",
    "float3x1",
    "float3x2",
    "float3x3",
    "float3x4",
    "float4x1",
    "float4x2",
    "float4x3",
    "float4x4",
    "int1x1",
    "int1x2",
    "int1x3",
    "int1x4",
    "int2x1",
    "int2x2",
    "int2x3",
    "int2x4",
    "int3x1",
    "int3x2",
    "int3x3",
    "int3x4",
    "int4x1",
    "int4x2",
    "int4x3",
    "int4x4",
    "double1x1",
    "double1x2",
    "double1x3",
    "double1x4",
    "double2x1",
    "double2x2",
    "double2x3",
    "double2x4",
    "double3x1",
    "double3x2",
    "double3x3",
    "double3x4",
    "double4x1",
    "double4x2",
    "double4x3",
    "double4x4",
    "abort",
    "abs",
    "acos",
    "all",
    "AllMemoryBarrier",
    "AllMemoryBarrierWithGroupSync",
    "any",
    "asdouble",
    "asfloat",
    "asin",
    "asint",
    "asint",
    "asuint",
    "asuint",
    "atan",
    "atan2",
    "ceil",
    "clamp",
    "clip",
    "cos",
    "cosh",
    "countbits",
    "cross",
    "D3DCOLORtoUBYTE4",
    "ddx",
    "ddx_coarse",
    "ddx_fine",
    "ddy",
    "ddy_coarse",
    "ddy_fine",
    "degrees",
    "determinant",
    "DeviceMemoryBarrier",
    "DeviceMemoryBarrierWithGroupSync",
    "distance",
    "dot",
    "dst",
    "errorf",
    "EvaluateAttributeAtCentroid",
    "EvaluateAttributeAtSample",
    "EvaluateAttributeSnapped",
    "exp",
    "exp2",
    "f16tof32",
    "f32tof16",
    "faceforward",
    "firstbithigh",
    "firstbitlow",
    "floor",
    "fma",
    "fmod",
    "frac",
    "frexp",
    "fwidth",
    "GetRenderTargetSampleCount",
    "GetRenderTargetSamplePosition",
    "GroupMemoryBarrier",
    "GroupMemoryBarrierWithGroupSync",
    "InterlockedAdd",
    "InterlockedAnd",
    "InterlockedCompareExchange",
    "InterlockedCompareStore",
    "InterlockedExchange",
    "InterlockedMax",
    "InterlockedMin",
    "InterlockedOr",
    "InterlockedXor",
    "isfinite",
    "isinf",
    "isnan",
    "ldexp",
    "length",
    "lerp",
    "lit",
    "log",
    "log10",
    "log2",
    "mad",
    "max",
    "min",
    "modf",
    "msad4",
    "mul",
    "noise",
    "normalize",
    "pow",
    "printf",
    "Process2DQuadTessFactorsAvg",
    "Process2DQuadTessFactorsMax",
    "Process2DQuadTessFactorsMin",
    "ProcessIsolineTessFactors",
    "ProcessQuadTessFactorsAvg",
    "ProcessQuadTessFactorsMax",
    "ProcessQuadTessFactorsMin",
    "ProcessTriTessFactorsAvg",
    "ProcessTriTessFactorsMax",
    "ProcessTriTessFactorsMin",
    "radians",
    "rcp",
    "reflect",
    "refract",
    "reversebits",
    "round",
    "rsqrt",
    "saturate",
    "sign",
    "sin",
    "sincos",
    "sinh",
    "smoothstep",
    "sqrt",
    "step",
    "tan",
    "tanh",
    "tex1D",
    "tex1D",
    "tex1Dbias",
    "tex1Dgrad",
    "tex1Dlod",
    "tex1Dproj",
    "tex2D",
    "tex2D",
    "tex2Dbias",
    "tex2Dgrad",
    "tex2Dlod",
    "tex2Dproj",
    "tex3D",
    "tex3D",
    "tex3Dbias",
    "tex3Dgrad",
    "tex3Dlod",
    "tex3Dproj",
    "texCUBE",
    "texCUBE",
    "texCUBEbias",
    "texCUBEgrad",
    "texCUBElod",
    "texCUBEproj",
    "transpose",
    "trunc"
  ];

  const GLSL_4_20_11_words = [
    "attribute",
    "const",
    "uniform",
    "varying",
    "coherent",
    "volatile",
    "restrict",
    "readonly",
    "writeonly",
    "atomic_uint",
    "layout",
    "centroid",
    "flat",
    "smooth",
    "noperspective",
    "patch",
    "sample",
    "break",
    "continue",
    "do",
    "for",
    "while",
    "switch",
    "case",
    "default",
    "if",
    "else",
    "subroutine",
    "in",
    "out",
    "inout",
    "float",
    "double",
    "int",
    "void",
    "bool",
    "true",
    "false",
    "invariant",
    "discard",
    "return",
    "mat2",
    "mat3",
    "mat4",
    "dmat2",
    "dmat3",
    "dmat4",
    "mat2x2",
    "mat2x3",
    "mat2x4",
    "dmat2x2",
    "dmat2x3",
    "dmat2x4",
    "mat3x2",
    "mat3x3",
    "mat3x4",
    "dmat3x2",
    "dmat3x3",
    "dmat3x4",
    "mat4x2",
    "mat4x3",
    "mat4x4",
    "dmat4x2",
    "dmat4x3",
    "dmat4x4",
    "vec2",
    "vec3",
    "vec4",
    "ivec2",
    "ivec3",
    "ivec4",
    "bvec2",
    "bvec3",
    "bvec4",
    "dvec2",
    "dvec3",
    "dvec4",
    "uint",
    "uvec2",
    "uvec3",
    "uvec4",
    "lowp",
    "mediump",
    "highp",
    "precision",
    "sampler1D",
    "sampler2D",
    "sampler3D",
    "samplerCube",
    "sampler1DShadow",
    "sampler2DShadow",
    "samplerCubeShadow",
    "sampler1DArray",
    "sampler2DArray",
    "sampler1DArrayShadow",
    "sampler2DArrayShadow",
    "isampler1D",
    "isampler2D",
    "isampler3D",
    "isamplerCube",
    "isampler1DArray",
    "isampler2DArray",
    "usampler1D",
    "usampler2D",
    "usampler3D",
    "usamplerCube",
    "usampler1DArray",
    "usampler2DArray",
    "sampler2DRect",
    "sampler2DRectShadow",
    "isampler2DRect",
    "usampler2DRect",
    "samplerBuffer",
    "isamplerBuffer",
    "usamplerBuffer",
    "sampler2DMS",
    "isampler2DMS",
    "usampler2DMS",
    "sampler2DMSArray",
    "isampler2DMSArray",
    "usampler2DMSArray",
    "samplerCubeArray",
    "samplerCubeArrayShadow",
    "isamplerCubeArray",
    "usamplerCubeArray",
    "image1D",
    "iimage1D",
    "uimage1D",
    "image2D",
    "iimage2D",
    "uimage2D",
    "image3D",
    "iimage3D",
    "uimage3D",
    "image2DRect",
    "iimage2DRect",
    "uimage2DRect",
    "imageCube",
    "iimageCube",
    "uimageCube",
    "imageBuffer",
    "iimageBuffer",
    "uimageBuffer",
    "image1DArray",
    "iimage1DArray",
    "uimage1DArray",
    "image2DArray",
    "iimage2DArray",
    "uimage2DArray",
    "imageCubeArray",
    "iimageCubeArray",
    "uimageCubeArray",
    "image2DMS",
    "iimage2DMS",
    "uimage2DMS",
    "image2DMSArray",
    "iimage2DMSArray",
    "uimage2DMSArray",
    "struct"
  ];

  const GLSL_4_20_11_future_words = [
    "common",
    "partition",
    "active",
    "asm",
    "class",
    "union",
    "enum",
    "typedef",
    "template",
    "this",
    "packed",
    "resource",
    "goto",
    "inline",
    "noinline",
    "public",
    "static",
    "extern",
    "external",
    "interface",
    "long",
    "short",
    "half",
    "fixed",
    "unsigned",
    "superp",
    "input",
    "output",
    "hvec2",
    "hvec3",
    "hvec4",
    "fvec2",
    "fvec3",
    "fvec4",
    "sampler3DRect",
    "filter",
    "sizeof",
    "cast",
    "namespace",
    "using",
    "row_major"
  ];

  const GLSL_1_0_17_words = [
    "attribute",
    "const",
    "uniform",
    "varying",
    "break",
    "continue",
    "do",
    "for",
    "while",
    "if",
    "else",
    "in",
    "out",
    "inout",
    "float",
    "int",
    "void",
    "bool",
    "true",
    "false",
    "lowp",
    "mediump",
    "highp",
    "precision",
    "invariant",
    "discard",
    "return",
    "mat2",
    "mat3",
    "mat4",
    "vec2",
    "vec3",
    "vec4",
    "ivec2",
    "ivec3",
    "ivec4",
    "bvec2",
    "bvec3",
    "bvec4",
    "sampler2D",
    "samplerCube",
    "struct"
  ]

  const GLSL_1_0_17_FutureWords = [
    "asm",
    "class",
    "union",
    "enum",
    "typedef",
    "template",
    "this",
    "packed",
    "goto",
    "switch",
    "default",
    "inline",
    "noinline",
    "volatile",
    "public",
    "static",
    "extern",
    "external",
    "interface",
    "flat",
    "long",
    "short",
    "double",
    "half",
    "fixed",
    "unsigned",
    "superp",
    "input",
    "output",
    "hvec2",
    "hvec3",
    "hvec4",
    "dvec2",
    "dvec3",
    "dvec4",
    "fvec2",
    "fvec3",
    "fvec4",
    "sampler1D",
    "sampler3D",
    "sampler1DShadow",
    "sampler2DShadow",
    "sampler2DRect",
    "sampler3DRect",
    "sampler2DRectShadow",
    "sizeof",
    "cast",
    "namespace",
    "using"
  ];

  const allBadWords = [
    ...DXWords,
    ...GLSL_4_20_11_words,
    ...GLSL_4_20_11_future_words,
  ];
  const numWordsPerPart = Math.ceil(allBadWords.length / numParts);
  const firstWordNdx = numWordsPerPart * (part - 1);
  const badWords = allBadWords.slice(firstWordNdx, firstWordNdx + numWordsPerPart);
  debug(`running tests for words ${firstWordNdx} to ${firstWordNdx + badWords.length - 1} of ${allBadWords.length}`);

  const shaders = {
    vertexShader0: `
struct $replaceMe {
  vec4 $replaceMe;
};
struct Foo {
  $replaceMe $replaceMe;
};
attribute vec4 position;
void main()
{
    Foo f;
    f.$replaceMe.$replaceMe = position;
    gl_Position = f.$replaceMe.$replaceMe;
}
`,
    fragmentShader0: `
precision mediump float;
vec4 $replaceMe() {
    return vec4(0,1,0,1);
}
void main()
{
    gl_FragColor = $replaceMe();
}
`,
    vertexShader1: `
attribute vec4 $replaceMe;
void main()
{
    gl_Position = $replaceMe;
}
`,
    fragmentShader1: `
precision mediump float;
vec4 foo(vec4 $replaceMe) {
  return $replaceMe;
}
void main()
{
    gl_FragColor = foo(vec4(1,0,1,1));
}
`,
    vertexShader2: `
varying vec4 $replaceMe;
attribute vec4 position;
void main()
{
    gl_Position = position;
    $replaceMe = position;
}
`,
    fragmentShader2: `
precision mediump float;
varying vec4 $replaceMe;
void main()
{
    gl_FragColor = $replaceMe;
}
`,
    vertexShader3: `
attribute vec4 position;
void main()
{
    gl_Position = position;
}
`,
    fragmentShader3: `
precision mediump float;
uniform vec4 $replaceMe;
void main()
{
    gl_FragColor = $replaceMe;
}
`,
  };

  const wtu = WebGLTestUtils;
  const gl = wtu.create3DContext();
  const wait = ms => new Promise(resolve => setTimeout(resolve, ms));

  const reservedWords = new Set([
      ...GLSL_1_0_17_words,
      ...GLSL_1_0_17_FutureWords,
  ]);

  const checkedWords = new Set();

  const src = [];
  for (let ii = 0; ii < 4; ++ii) {
    const vSrc = shaders[`vertexShader${ii}`];
    const fSrc = shaders[`fragmentShader${ii}`];
    src.push({vSrc: vSrc, fSrc: fSrc});
  }

  for (const badWord of badWords) {
    testWord(badWord);
    await wait();
  }
  finishTest();

  function testWord(word) {
    if (reservedWords.has(word) || checkedWords.has(word)) {
      return;
    }
    checkedWords.add(word);
    debug("");
    debug(`testing: ${word}`);

    for (let ii = 0; ii < src.length; ++ii) {
      const vs = src[ii].vSrc.replace(/\$replaceMe/g, word);
      const fs = src[ii].fSrc.replace(/\$replaceMe/g, word);

      let success = true;
      const program = wtu.loadProgram(gl, vs, fs, function(msg) {
        debug(msg);
        success = false;
      }, true);
      if (success) {
        testPassed(`shader with: '${word}' compiled`);
      } else {
        testFailed(`shader with: '${word}' failed to compile`);
      }
      if (program) {
        gl.deleteProgram(program);
      }
      wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no GL errors");
    }
  }
}