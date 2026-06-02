"use strict";
description("This test ensures WebGL implementations correctly implement querying for compressed textures when extensions are disabled.");

debug("");

const wtu = WebGLTestUtils;
const gl = wtu.create3DContext(null, undefined, contextVersion);

const COMPRESSED_RGB_PVRTC_4BPPV1_IMG     = 0x8C00;
const COMPRESSED_RGBA_PVRTC_4BPPV1_IMG    = 0x8C02;

let formats = null;
let ext;

if (!gl) {
  testFailed("context does not exist");
} else {
  testPassed("context exists");

  var tex = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, tex);
  wtu.shouldGenerateGLError(gl, [gl.INVALID_ENUM, gl.INVALID_OPERATION],
                            "gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 10, 10, COMPRESSED_RGB_PVRTC_4BPPV1_IMG, new Uint8Array(8));");

  wtu.shouldGenerateGLError(gl, gl.INVALID_ENUM, "gl.compressedTexImage2D(gl.TEXTURE_2D, 0, COMPRESSED_RGB_PVRTC_4BPPV1_IMG, 8, 8, 0, new Uint8Array(8))");
  wtu.shouldGenerateGLError(gl, gl.INVALID_ENUM, "gl.compressedTexImage2D(gl.TEXTURE_2D, 0, COMPRESSED_RGBA_PVRTC_4BPPV1_IMG, 8, 8, 0, new Uint8Array(8))");

  wtu.shouldGenerateGLError(gl, gl.NO_ERROR, "formats = gl.getParameter(gl.COMPRESSED_TEXTURE_FORMATS)");
  shouldBeNonNull("formats");
  shouldBe("formats.length", "0");

  wtu.shouldGenerateGLError(gl, gl.NO_ERROR, "gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(4*4*4));");
  wtu.shouldGenerateGLError(gl, gl.INVALID_ENUM,
                            "gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 4, 4, COMPRESSED_RGB_PVRTC_4BPPV1_IMG, new Uint8Array(8));");

  // Check too-many and too-few args.

  wtu.shouldThrow(gl, false, "too many args", function() {
    gl.compressedTexImage2D(gl.TEXTURE_2D, 0, COMPRESSED_RGB_PVRTC_4BPPV1_IMG, 4, 4, 0, new Uint8Array(8), null);
  });
  wtu.shouldThrow(gl, TypeError, "too few args", function() {
    gl.compressedTexImage2D(gl.TEXTURE_2D, 0, COMPRESSED_RGB_PVRTC_4BPPV1_IMG, 4, 4, 0);
  });

  wtu.shouldThrow(gl, false, "too many args", function() {
    gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 4, 4, COMPRESSED_RGB_PVRTC_4BPPV1_IMG, new Uint8Array(8), null);
  });
  wtu.shouldThrow(gl, TypeError, "too few args", function() {
    gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 4, 4, COMPRESSED_RGB_PVRTC_4BPPV1_IMG);
  });

  // -

  let pbo;
  // WebGL 2.0 specific
  if (gl.PIXEL_UNPACK_BUFFER) {
    pbo = gl.createBuffer();
  }

  gl.bindTexture(gl.TEXTURE_2D, tex);

  function validateExt(extName, enumName, blockSize, blockByteSize, expectedSubImageError) {
    debug('\n---------------------------');
    debug('\n' + extName);
    ext = gl.getExtension(extName);
    if (!ext) {
      testPassed(`Optional ext ${extName} MAY be unsupported.`);
      return;
    }
    testPassed(`Optional ext ${extName} is supported.`);

    const data = new Uint8Array(blockByteSize);

    const views = [
      data,
      new Uint8ClampedArray(data.buffer),
      new Int8Array(data.buffer),
      new Uint16Array(data.buffer),
      new Int16Array(data.buffer),
      new Uint32Array(data.buffer),
      new Int32Array(data.buffer),
      new Float32Array(data.buffer),
      new DataView(data.buffer),
    ];
    if (window.SharedArrayBuffer) {
      const sharedBuffer = new SharedArrayBuffer(blockByteSize);
      views.push(
        new Uint8Array(sharedBuffer),
        new Uint8ClampedArray(sharedBuffer),
        new DataView(sharedBuffer)
      );
    }

    for (const view of views) {
      window.g_view = view;
      debug(`\nfrom ${view.constructor.name} of ${view.buffer.constructor.name}`);
      wtu.shouldGenerateGLError(gl, gl.NO_ERROR,
          `gl.compressedTexImage2D(gl.TEXTURE_2D, 0, ext.${enumName}, ${blockSize},${blockSize}, 0, g_view)`);

      wtu.shouldGenerateGLError(gl, expectedSubImageError,
          `gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0,0, ${blockSize},${blockSize}, ext.${enumName}, g_view)`);
    }

    if (pbo) {
      debug('\nfrom PBO');
      gl.bindBuffer(gl.PIXEL_UNPACK_BUFFER, pbo);
      wtu.shouldGenerateGLError(gl, gl.NO_ERROR,
          `gl.bufferData(gl.PIXEL_UNPACK_BUFFER, ${blockByteSize}*2, gl.STATIC_DRAW)`);

      wtu.shouldGenerateGLError(gl, gl.NO_ERROR,
          `gl.compressedTexImage2D(gl.TEXTURE_2D, 0, ext.${enumName}, ${blockSize},${blockSize}, 0, ${blockByteSize}, 0)`);
      wtu.shouldGenerateGLError(gl, gl.NO_ERROR,
          `gl.compressedTexImage2D(gl.TEXTURE_2D, 0, ext.${enumName}, ${blockSize},${blockSize}, 0, ${blockByteSize}, 1)`);
      wtu.shouldGenerateGLError(gl, gl.NO_ERROR,
          `gl.compressedTexImage2D(gl.TEXTURE_2D, 0, ext.${enumName}, ${blockSize},${blockSize}, 0, ${blockByteSize}, ${blockByteSize})`);
      wtu.shouldGenerateGLError(gl, gl.INVALID_OPERATION,
          `gl.compressedTexImage2D(gl.TEXTURE_2D, 0, ext.${enumName}, ${blockSize},${blockSize}, 0, ${blockByteSize}, ${blockByteSize+1})`);

      wtu.shouldGenerateGLError(gl, expectedSubImageError,
          `gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0,0, ${blockSize},${blockSize}, ext.${enumName}, ${blockByteSize}, 0)`);
      wtu.shouldGenerateGLError(gl, expectedSubImageError,
          `gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0,0, ${blockSize},${blockSize}, ext.${enumName}, ${blockByteSize}, 1)`);
      wtu.shouldGenerateGLError(gl, expectedSubImageError,
          `gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0,0, ${blockSize},${blockSize}, ext.${enumName}, ${blockByteSize}, ${blockByteSize})`);
      wtu.shouldGenerateGLError(gl, gl.INVALID_OPERATION,
          `gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0,0, ${blockSize},${blockSize}, ext.${enumName}, ${blockByteSize}, ${blockByteSize+1})`);

      gl.bindBuffer(gl.PIXEL_UNPACK_BUFFER, null);
    }
  }

  validateExt('WEBGL_compressed_texture_s3tc', 'COMPRESSED_RGBA_S3TC_DXT5_EXT', 4, 16, gl.NO_ERROR);
  validateExt('WEBGL_compressed_texture_etc1', 'COMPRESSED_RGB_ETC1_WEBGL', 4, 8, gl.INVALID_OPERATION);
  validateExt('WEBGL_compressed_texture_etc', 'COMPRESSED_RGBA8_ETC2_EAC', 4, 16, gl.NO_ERROR);
  validateExt('WEBGL_compressed_texture_astc', 'COMPRESSED_RGBA_ASTC_4x4_KHR', 4, 16, gl.NO_ERROR);
}

var successfullyParsed = true;
