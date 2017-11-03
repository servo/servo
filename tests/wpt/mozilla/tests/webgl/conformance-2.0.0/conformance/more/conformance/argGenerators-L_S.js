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

// L

  lineWidth : {
    generate : function() { return [randomLineWidth()]; },
    teardown : function() { GL.lineWidth(1); }
  },
  linkProgram : {}, // FIXME

// P
  pixelStorei : {
    generate : function() {
      return [pixelStoreiPname.random(), pixelStoreiParam.random()];
    },
    checkArgValidity : function(pname, param) {
      return pixelStoreiPname.has(pname) && pixelStoreiParam.has(param);
    },
    teardown : function() {
      GL.pixelStorei(GL.PACK_ALIGNMENT, 4);
      GL.pixelStorei(GL.UNPACK_ALIGNMENT, 4);
    }
  },
  polygonOffset : {
    generate : function() { return [randomFloat(), randomFloat()]; },
    teardown : function() { GL.polygonOffset(0,0); }
  },

// R

  readPixels : {}, // FIXME
  renderbufferStorage : {}, // FIXME

// S-1

  sampleCoverage : {
    generate : function() { return [randomFloatFromRange(0,1), randomBool()] },
    teardown : function() { GL.sampleCoverage(1, false); }
  },
  scissor : {
    generate : function() {
      return [randomInt(3000)-1500, randomInt(3000)-1500, randomIntFromRange(0,3000), randomIntFromRange(0,3000)];
    },
    checkArgValidity : function(x,y,w,h) {
      return castToInt(w) >= 0 && castToInt(h) >= 0;
    },
    teardown : function() {
      GL.scissor(0,0,GL.canvas.width, GL.canvas.height);
    }
  },
  shaderSource : {}, // FIXME
  stencilFunc : {
    generate : function(){
      return [stencilFuncFunc.random(), randomInt(MaxStencilValue), randomInt(0xffffffff)];
    },
    checkArgValidity : function(func, ref, mask) {
      return stencilFuncFunc.has(func) && castToInt(ref) >= 0 && castToInt(ref) < MaxStencilValue;
    },
    teardown : function() {
      GL.stencilFunc(GL.ALWAYS, 0, 0xffffffff);
    }
  },
  stencilFuncSeparate : {
    generate : function(){
      return [cullFace.random(), stencilFuncFunc.random(), randomInt(MaxStencilValue), randomInt(0xffffffff)];
    },
    checkArgValidity : function(face, func, ref, mask) {
      return cullFace.has(face) && stencilFuncFunc.has(func) && castToInt(ref) >= 0 && castToInt(ref) < MaxStencilValue;
    },
    teardown : function() {
      GL.stencilFunc(GL.ALWAYS, 0, 0xffffffff);
    }
  },
  stencilMask : {
    generate : function() { return [randomInt(0xffffffff)]; },
    teardown : function() { GL.stencilMask(0xffffffff); }
  }

};
