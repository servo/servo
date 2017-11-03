/*
** Copyright (c) 2012 The Khronos Group Inc.
**
** Permission is hereby granted, free of charge, to any person obtaining a
** copy of this software and/or associated documentation files (the
** "Materials"), to deal in the Materials without restriction, including
** without limitation the rights to use, copy, modify, merge, publish,
** distribute, sublicense, and/or sell copies of the Materials, and to
** permit persons to whom the Materials are furnished to do so, subject to
** the following conditions:
**
** The above copyright notice and this permission notice shall be included
** in all copies or substantial portions of the Materials.
**
** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
*/

// ArgGenerators contains argument generators for WebGL functions.
// The argument generators are used for running random tests against the WebGL
// functions.
//
// ArgGenerators is an object consisting of functionName : argGen -properties.
//
// functionName is a WebGL context function name and the argGen is an argument
// generator object that encapsulates the requirements to run
// randomly generated tests on the WebGL function.
//
// An argGen object has the following methods:
//   - setup    -- set up state for testing the GL function, returns values
//                 that need cleanup in teardown. Run once before entering a
//                 test loop.
//   - teardown -- do cleanup on setup's return values after testing is complete
//   - generate -- generate a valid set of random arguments for the GL function
//   - returnValueCleanup -- do cleanup on value returned by the tested GL function
//   - cleanup  -- do cleanup on generated arguments from generate
//   - checkArgValidity -- check if passed args are valid. Has a call signature
//                         that matches generate's return value. Returns true
//                         if args are valid, false if not.
//
//   Example test loop that demonstrates how the function args and return
//   values flow together:
//
//   var setupArgs = argGen.setup();
//   for (var i=0; i<numberOfTests; i++) {
//     var generatedArgs = argGen.generate.apply(argGen, setupArgs);
//     var validArgs = argGen.checkArgValidity.apply(argGen, generatedArgs);
//     var rv = call the GL function with generatedArgs;
//     argGen.returnValueCleanup(rv);
//     argGen.cleanup.apply(argGen, generatedArgs);
//   }
//   argGen.teardown.apply(argGen, setupArgs);
//
ArgGenerators = {

// GL functions in alphabetical order

// S-2

  stencilMaskSeparate : {
    generate : function() { return [cullFace.random(), randomInt(0xffffffff)]; },
    checkArgValidity : function(face, mask) {
      return cullFace.has(face);
    },
    teardown : function() { GL.stencilMask(0xffffffff); }
  },
  stencilOp : {
    generate : function() {
      return [stencilOp.random(), stencilOp.random(), stencilOp.random()];
    },
    checkArgValidity : function(sfail, dpfail, dppass) {
      return stencilOp.has(sfail) && stencilOp.has(dpfail) && stencilOp.has(dppass);
    },
    teardown : function() { GL.stencilOp(GL.KEEP, GL.KEEP, GL.KEEP); }
  },
  stencilOpSeparate : {
    generate : function() {
      return [cullFace.random(), stencilOp.random(), stencilOp.random(), stencilOp.random()];
    },
    checkArgValidity : function(face, sfail, dpfail, dppass) {
      return cullFace.has(face) && stencilOp.has(sfail) &&
              stencilOp.has(dpfail) && stencilOp.has(dppass);
    },
    teardown : function() { GL.stencilOp(GL.KEEP, GL.KEEP, GL.KEEP); }
  },

// T
  texImage2D : {
    noAlreadyTriedCheck : true, // Object.toSource is very slow here
    setup : function() {
      var tex = GL.createTexture();
      var tex2 = GL.createTexture();
      GL.bindTexture(GL.TEXTURE_2D, tex);
      GL.bindTexture(GL.TEXTURE_CUBE_MAP, tex2);
      return [tex, tex2];
    },
    generate : function() {
      var format = texImageFormat.random();
      if (Math.random() < 0.5) {
        var img = randomImage(16,16);
        var a = [ texImageTarget.random(), 0, format, format, GL.UNSIGNED_BYTE, img ];
        return a;
      } else {
        var pix = null;
        if (Math.random > 0.5) {
          pix = new Uint8Array(16*16*4);
        }
        return [
          texImageTarget.random(), 0,
          format, 16, 16, 0,
          format, GL.UNSIGNED_BYTE, pix
        ];
      }
    },
    checkArgValidity : function(target, level, internalformat, width, height, border, format, type, data) {
               // or : function(target, level, internalformat, format, type, image)
      if (!texImageTarget.has(target) || castToInt(level) < 0)
        return false;
      if (arguments.length <= 6) {
        var xformat = width;
        var xtype = height;
        var ximage = border;
        if ((ximage instanceof HTMLImageElement ||
             ximage instanceof HTMLVideoElement ||
             ximage instanceof HTMLCanvasElement ||
             ximage instanceof ImageData) &&
            texImageInternalFormat.has(internalformat) &&
            texImageFormat.has(xformat) &&
            texImageType.has(xtype) &&
            internalformat == xformat)
          return true;
        return false;
      }
      var w = castToInt(width), h = castToInt(height), b = castToInt(border);
      return texImageInternalFormat.has(internalformat) && w >= 0 && h >= 0 &&
            b == 0 && (data == null || data.byteLength == w*h*4) &&
            texImageFormat.has(format) && texImageType.has(type)
            && internalformat == format;
    },
    teardown : function(tex, tex2) {
      GL.bindTexture(GL.TEXTURE_2D, null);
      GL.bindTexture(GL.TEXTURE_CUBE_MAP, null);
      GL.deleteTexture(tex);
      GL.deleteTexture(tex2);
    }
  },
  texParameterf : {
    generate : function() {
      var pname = texParameterPname.random();
      var param = texParameterParam[pname].random();
      return [bindTextureTarget.random(), pname, param];
    },
    checkArgValidity : function(target, pname, param) {
      if (!bindTextureTarget.has(target))
        return false;
      if (!texParameterPname.has(pname))
        return false;
      return texParameterParam[pname].has(param);
    }
  },
  texParameteri : {
    generate : function() {
      var pname = texParameterPname.random();
      var param = texParameterParam[pname].random();
      return [bindTextureTarget.random(), pname, param];
    },
    checkArgValidity : function(target, pname, param) {
      if (!bindTextureTarget.has(target))
        return false;
      if (!texParameterPname.has(pname))
        return false;
      return texParameterParam[pname].has(param);
    }
  },
  texSubImage2D : {}, // FIXME

// U

  uniform1f : {}, // FIXME
  uniform1fv : {}, // FIXME
  uniform1i : {}, // FIXME
  uniform1iv : {}, // FIXME
  uniform2f : {}, // FIXME
  uniform2fv : {}, // FIXME
  uniform2i : {}, // FIXME
  uniform2iv : {}, // FIXME
  uniform3f : {}, // FIXME
  uniform3fv : {}, // FIXME
  uniform3i : {}, // FIXME
  uniform3iv : {}, // FIXME
  uniform4f : {}, // FIXME
  uniform4fv : {}, // FIXME
  uniform4i : {}, // FIXME
  uniform4iv : {}, // FIXME
  uniformMatrix2fv : {}, // FIXME
  uniformMatrix3fv : {}, // FIXME
  uniformMatrix4fv : {}, // FIXME
  useProgram : {}, // FIXME

// V

  validateProgram : {}, // FIXME
  vertexAttrib1f : {}, // FIXME
  vertexAttrib1fv : {}, // FIXME
  vertexAttrib2f : {}, // FIXME
  vertexAttrib2fv : {}, // FIXME
  vertexAttrib3f : {}, // FIXME
  vertexAttrib3fv : {}, // FIXME
  vertexAttrib4f : {}, // FIXME
  vertexAttrib4fv : {}, // FIXME
  vertexAttribPointer : {}, // FIXME
  viewport : {
    generate : function() {
      return [randomInt(3000)-1500, randomInt(3000)-1500, randomIntFromRange(0,3000), randomIntFromRange(0,3000)];
    },
    checkArgValidity : function(x,y,w,h) {
      return castToInt(w) >= 0 && castToInt(h) >= 0;
    },
    teardown : function() {
      GL.viewport(0,0,GL.canvas.width, GL.canvas.height);
    }
  }

};
