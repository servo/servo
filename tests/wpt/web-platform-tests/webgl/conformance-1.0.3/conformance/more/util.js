/*
Utilities for the OpenGL ES 2.0 HTML Canvas context
*/

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

function loadTexture(gl, elem, mipmaps) {
  var tex = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, elem);
  if (mipmaps != false)
    gl.generateMipmap(gl.TEXTURE_2D);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  if (mipmaps)
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);
  else
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  return tex;
}

function getShader(gl, id) {
  var shaderScript = document.getElementById(id);
  if (!shaderScript) {
    throw(new Error("No shader element with id: "+id));
  }

  var str = "";
  var k = shaderScript.firstChild;
  while (k) {
    if (k.nodeType == 3)
      str += k.textContent;
    k = k.nextSibling;
  }

  var shader;
  if (shaderScript.type == "x-shader/x-fragment") {
    shader = gl.createShader(gl.FRAGMENT_SHADER);
  } else if (shaderScript.type == "x-shader/x-vertex") {
    shader = gl.createShader(gl.VERTEX_SHADER);
  } else {
    throw(new Error("Unknown shader type "+shaderScript.type));
  }

  gl.shaderSource(shader, str);
  gl.compileShader(shader);

  if (gl.getShaderParameter(shader, gl.COMPILE_STATUS) != 1) {
    var ilog = gl.getShaderInfoLog(shader);
    gl.deleteShader(shader);
    throw(new Error("Failed to compile shader "+shaderScript.id + ", Shader info log: " + ilog));
  }
  return shader;
}

function loadShaderArray(gl, shaders) {
  var id = gl.createProgram();
  var shaderObjs = [];
  for (var i=0; i<shaders.length; ++i) {
    try {
      var sh = getShader(gl, shaders[i]);
      shaderObjs.push(sh);
      gl.attachShader(id, sh);
    } catch (e) {
      var pr = {program: id, shaders: shaderObjs};
      deleteShader(gl, pr);
      throw (e);
    }
  }
  var prog = {program: id, shaders: shaderObjs};
  gl.linkProgram(id);
  gl.validateProgram(id);
  if (gl.getProgramParameter(id, gl.LINK_STATUS) != 1) {
    deleteShader(gl,prog);
    throw(new Error("Failed to link shader"));
  }
  if (gl.getProgramParameter(id, gl.VALIDATE_STATUS) != 1) {
    deleteShader(gl,prog);
    throw(new Error("Failed to validate shader"));
  }
  return prog;
}
function loadShader(gl) {
  var sh = [];
  for (var i=1; i<arguments.length; ++i)
    sh.push(arguments[i]);
  return loadShaderArray(gl, sh);
}

function deleteShader(gl, sh) {
  gl.useProgram(null);
  sh.shaders.forEach(function(s){
    gl.detachShader(sh.program, s);
    gl.deleteShader(s);
  });
  gl.deleteProgram(sh.program);
}

function getGLErrorAsString(ctx, err) {
  if (err === ctx.NO_ERROR) {
    return "NO_ERROR";
  }
  for (var name in ctx) {
    if (ctx[name] === err) {
      return name;
    }
  }
  return err.toString();
}

function checkError(gl, msg) {
  var e = gl.getError();
  if (e != gl.NO_ERROR) {
    log("Error " + getGLErrorAsString(gl, e) + " at " + msg);
  }
  return e;
}

function throwError(gl, msg) {
  var e = gl.getError();
  if (e != 0) {
    throw(new Error("Error " + getGLErrorAsString(gl, e) + " at " + msg));
  }
}

Math.cot = function(z) { return 1.0 / Math.tan(z); }

/*
  Matrix utilities, using the OpenGL element order where
  the last 4 elements are the translation column.

  Uses flat arrays as matrices for performance.

  Most operations have in-place variants to avoid allocating temporary matrices.

  Naming logic:
    Matrix.method operates on a 4x4 Matrix and returns a new Matrix.
    Matrix.method3x3 operates on a 3x3 Matrix and returns a new Matrix. Not all operations have a 3x3 version (as 3x3 is usually only used for the normal matrix: Matrix.transpose3x3(Matrix.inverseTo3x3(mat4x4)))
    Matrix.method[3x3]InPlace(args, target) stores its result in the target matrix.

    Matrix.scale([sx, sy, sz]) -- non-uniform scale by vector
    Matrix.scale1(s)           -- uniform scale by scalar
    Matrix.scale3(sx, sy, sz)  -- non-uniform scale by scalars

    Ditto for translate.
*/
Matrix = {
  identity : [
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
  ],

  newIdentity : function() {
    return [
      1.0, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0
    ];
  },

  newIdentity3x3 : function() {
    return [
      1.0, 0.0, 0.0,
      0.0, 1.0, 0.0,
      0.0, 0.0, 1.0
    ];
  },

  copyMatrix : function(src, dst) {
    for (var i=0; i<16; i++) dst[i] = src[i];
    return dst;
  },

  to3x3 : function(m) {
    return [
      m[0], m[1], m[2],
      m[4], m[5], m[6],
      m[8], m[9], m[10]
    ];
  },

  // orthonormal matrix inverse
  inverseON : function(m) {
    var n = this.transpose4x4(m);
    var t = [m[12], m[13], m[14]];
    n[3] = n[7] = n[11] = 0;
    n[12] = -Vec3.dot([n[0], n[4], n[8]], t);
    n[13] = -Vec3.dot([n[1], n[5], n[9]], t);
    n[14] = -Vec3.dot([n[2], n[6], n[10]], t);
    return n;
  },

  inverseTo3x3 : function(m) {
    return this.inverse4x4to3x3InPlace(m, this.newIdentity3x3());
  },

  inverseTo3x3InPlace : function(m,n) {
    var a11 = m[10]*m[5]-m[6]*m[9],
        a21 = -m[10]*m[1]+m[2]*m[9],
        a31 = m[6]*m[1]-m[2]*m[5],
        a12 = -m[10]*m[4]+m[6]*m[8],
        a22 = m[10]*m[0]-m[2]*m[8],
        a32 = -m[6]*m[0]+m[2]*m[4],
        a13 = m[9]*m[4]-m[5]*m[8],
        a23 = -m[9]*m[0]+m[1]*m[8],
        a33 = m[5]*m[0]-m[1]*m[4];
    var det = m[0]*(a11) + m[1]*(a12) + m[2]*(a13);
    if (det == 0) // no inverse
      return [1,0,0,0,1,0,0,0,1];
    var idet = 1 / det;
    n[0] = idet*a11;
    n[1] = idet*a21;
    n[2] = idet*a31;
    n[3] = idet*a12;
    n[4] = idet*a22;
    n[5] = idet*a32;
    n[6] = idet*a13;
    n[7] = idet*a23;
    n[8] = idet*a33;
    return n;
  },

  inverse3x3 : function(m) {
    return this.inverse3x3InPlace(m, this.newIdentity3x3());
  },

  inverse3x3InPlace : function(m,n) {
    var a11 = m[8]*m[4]-m[5]*m[7],
        a21 = -m[8]*m[1]+m[2]*m[7],
        a31 = m[5]*m[1]-m[2]*m[4],
        a12 = -m[8]*m[3]+m[5]*m[6],
        a22 = m[8]*m[0]-m[2]*m[6],
        a32 = -m[5]*m[0]+m[2]*m[3],
        a13 = m[7]*m[4]-m[4]*m[8],
        a23 = -m[7]*m[0]+m[1]*m[6],
        a33 = m[4]*m[0]-m[1]*m[3];
    var det = m[0]*(a11) + m[1]*(a12) + m[2]*(a13);
    if (det == 0) // no inverse
      return [1,0,0,0,1,0,0,0,1];
    var idet = 1 / det;
    n[0] = idet*a11;
    n[1] = idet*a21;
    n[2] = idet*a31;
    n[3] = idet*a12;
    n[4] = idet*a22;
    n[5] = idet*a32;
    n[6] = idet*a13;
    n[7] = idet*a23;
    n[8] = idet*a33;
    return n;
  },

  frustum : function (left, right, bottom, top, znear, zfar) {
    var X = 2*znear/(right-left);
    var Y = 2*znear/(top-bottom);
    var A = (right+left)/(right-left);
    var B = (top+bottom)/(top-bottom);
    var C = -(zfar+znear)/(zfar-znear);
    var D = -2*zfar*znear/(zfar-znear);

    return [
      X, 0, 0, 0,
      0, Y, 0, 0,
      A, B, C, -1,
      0, 0, D, 0
    ];
 },

  perspective : function (fovy, aspect, znear, zfar) {
    var ymax = znear * Math.tan(fovy * Math.PI / 360.0);
    var ymin = -ymax;
    var xmin = ymin * aspect;
    var xmax = ymax * aspect;

    return this.frustum(xmin, xmax, ymin, ymax, znear, zfar);
  },

  mul4x4 : function (a,b) {
    return this.mul4x4InPlace(a,b,this.newIdentity());
  },

  mul4x4InPlace : function (a, b, c) {
        c[0] =   b[0] * a[0] +
                 b[0+1] * a[4] +
                 b[0+2] * a[8] +
                 b[0+3] * a[12];
        c[0+1] = b[0] * a[1] +
                 b[0+1] * a[5] +
                 b[0+2] * a[9] +
                 b[0+3] * a[13];
        c[0+2] = b[0] * a[2] +
                 b[0+1] * a[6] +
                 b[0+2] * a[10] +
                 b[0+3] * a[14];
        c[0+3] = b[0] * a[3] +
                 b[0+1] * a[7] +
                 b[0+2] * a[11] +
                 b[0+3] * a[15];
        c[4] =   b[4] * a[0] +
                 b[4+1] * a[4] +
                 b[4+2] * a[8] +
                 b[4+3] * a[12];
        c[4+1] = b[4] * a[1] +
                 b[4+1] * a[5] +
                 b[4+2] * a[9] +
                 b[4+3] * a[13];
        c[4+2] = b[4] * a[2] +
                 b[4+1] * a[6] +
                 b[4+2] * a[10] +
                 b[4+3] * a[14];
        c[4+3] = b[4] * a[3] +
                 b[4+1] * a[7] +
                 b[4+2] * a[11] +
                 b[4+3] * a[15];
        c[8] =   b[8] * a[0] +
                 b[8+1] * a[4] +
                 b[8+2] * a[8] +
                 b[8+3] * a[12];
        c[8+1] = b[8] * a[1] +
                 b[8+1] * a[5] +
                 b[8+2] * a[9] +
                 b[8+3] * a[13];
        c[8+2] = b[8] * a[2] +
                 b[8+1] * a[6] +
                 b[8+2] * a[10] +
                 b[8+3] * a[14];
        c[8+3] = b[8] * a[3] +
                 b[8+1] * a[7] +
                 b[8+2] * a[11] +
                 b[8+3] * a[15];
        c[12] =   b[12] * a[0] +
                 b[12+1] * a[4] +
                 b[12+2] * a[8] +
                 b[12+3] * a[12];
        c[12+1] = b[12] * a[1] +
                 b[12+1] * a[5] +
                 b[12+2] * a[9] +
                 b[12+3] * a[13];
        c[12+2] = b[12] * a[2] +
                 b[12+1] * a[6] +
                 b[12+2] * a[10] +
                 b[12+3] * a[14];
        c[12+3] = b[12] * a[3] +
                 b[12+1] * a[7] +
                 b[12+2] * a[11] +
                 b[12+3] * a[15];
    return c;
  },

  mulv4 : function (a, v) {
    c = new Array(4);
    for (var i=0; i<4; ++i) {
      var x = 0;
      for (var k=0; k<4; ++k)
        x += v[k] * a[k*4+i];
      c[i] = x;
    }
    return c;
  },

  rotate : function (angle, axis) {
    axis = Vec3.normalize(axis);
    var x=axis[0], y=axis[1], z=axis[2];
    var c = Math.cos(angle);
    var c1 = 1-c;
    var s = Math.sin(angle);
    return [
      x*x*c1+c, y*x*c1+z*s, z*x*c1-y*s, 0,
      x*y*c1-z*s, y*y*c1+c, y*z*c1+x*s, 0,
      x*z*c1+y*s, y*z*c1-x*s, z*z*c1+c, 0,
      0,0,0,1
    ];
  },
  rotateInPlace : function(angle, axis, m) {
    axis = Vec3.normalize(axis);
    var x=axis[0], y=axis[1], z=axis[2];
    var c = Math.cos(angle);
    var c1 = 1-c;
    var s = Math.sin(angle);
    var tmpMatrix = this.tmpMatrix;
    var tmpMatrix2 = this.tmpMatrix2;
    tmpMatrix[0] = x*x*c1+c; tmpMatrix[1] = y*x*c1+z*s; tmpMatrix[2] = z*x*c1-y*s; tmpMatrix[3] = 0;
    tmpMatrix[4] = x*y*c1-z*s; tmpMatrix[5] = y*y*c1+c; tmpMatrix[6] = y*z*c1+x*s; tmpMatrix[7] = 0;
    tmpMatrix[8] = x*z*c1+y*s; tmpMatrix[9] = y*z*c1-x*s; tmpMatrix[10] = z*z*c1+c; tmpMatrix[11] = 0;
    tmpMatrix[12] = 0; tmpMatrix[13] = 0; tmpMatrix[14] = 0; tmpMatrix[15] = 1;
    this.copyMatrix(m, tmpMatrix2);
    return this.mul4x4InPlace(tmpMatrix2, tmpMatrix, m);
  },

  scale : function(v) {
    return [
      v[0], 0, 0, 0,
      0, v[1], 0, 0,
      0, 0, v[2], 0,
      0, 0, 0, 1
    ];
  },
  scale3 : function(x,y,z) {
    return [
      x, 0, 0, 0,
      0, y, 0, 0,
      0, 0, z, 0,
      0, 0, 0, 1
    ];
  },
  scale1 : function(s) {
    return [
      s, 0, 0, 0,
      0, s, 0, 0,
      0, 0, s, 0,
      0, 0, 0, 1
    ];
  },
  scale3InPlace : function(x, y, z, m) {
    var tmpMatrix = this.tmpMatrix;
    var tmpMatrix2 = this.tmpMatrix2;
    tmpMatrix[0] = x; tmpMatrix[1] = 0; tmpMatrix[2] = 0; tmpMatrix[3] = 0;
    tmpMatrix[4] = 0; tmpMatrix[5] = y; tmpMatrix[6] = 0; tmpMatrix[7] = 0;
    tmpMatrix[8] = 0; tmpMatrix[9] = 0; tmpMatrix[10] = z; tmpMatrix[11] = 0;
    tmpMatrix[12] = 0; tmpMatrix[13] = 0; tmpMatrix[14] = 0; tmpMatrix[15] = 1;
    this.copyMatrix(m, tmpMatrix2);
    return this.mul4x4InPlace(tmpMatrix2, tmpMatrix, m);
  },
  scale1InPlace : function(s, m) { return this.scale3InPlace(s, s, s, m); },
  scaleInPlace : function(s, m) { return this.scale3InPlace(s[0],s[1],s[2],m); },

  translate3 : function(x,y,z) {
    return [
      1, 0, 0, 0,
      0, 1, 0, 0,
      0, 0, 1, 0,
      x, y, z, 1
    ];
  },

  translate : function(v) {
    return this.translate3(v[0], v[1], v[2]);
  },
  tmpMatrix : [0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0],
  tmpMatrix2 : [0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0],
  translate3InPlace : function(x,y,z,m) {
    var tmpMatrix = this.tmpMatrix;
    var tmpMatrix2 = this.tmpMatrix2;
    tmpMatrix[0] = 1; tmpMatrix[1] = 0; tmpMatrix[2] = 0; tmpMatrix[3] = 0;
    tmpMatrix[4] = 0; tmpMatrix[5] = 1; tmpMatrix[6] = 0; tmpMatrix[7] = 0;
    tmpMatrix[8] = 0; tmpMatrix[9] = 0; tmpMatrix[10] = 1; tmpMatrix[11] = 0;
    tmpMatrix[12] = x; tmpMatrix[13] = y; tmpMatrix[14] = z; tmpMatrix[15] = 1;
    this.copyMatrix(m, tmpMatrix2);
    return this.mul4x4InPlace(tmpMatrix2, tmpMatrix, m);
  },
  translateInPlace : function(v,m){ return this.translate3InPlace(v[0], v[1], v[2], m); },

  lookAt : function (eye, center, up) {
    var z = Vec3.direction(eye, center);
    var x = Vec3.normalizeInPlace(Vec3.cross(up, z));
    var y = Vec3.normalizeInPlace(Vec3.cross(z, x));

    var m = [
      x[0], y[0], z[0], 0,
      x[1], y[1], z[1], 0,
      x[2], y[2], z[2], 0,
      0, 0, 0, 1
    ];

    var t = [
      1, 0, 0, 0,
      0, 1, 0, 0,
      0, 0, 1, 0,
      -eye[0], -eye[1], -eye[2], 1
    ];

    return this.mul4x4(m,t);
  },

  transpose4x4 : function(m) {
    return [
      m[0], m[4], m[8], m[12],
      m[1], m[5], m[9], m[13],
      m[2], m[6], m[10], m[14],
      m[3], m[7], m[11], m[15]
    ];
  },

  transpose4x4InPlace : function(m) {
    var tmp = 0.0;
    tmp = m[1]; m[1] = m[4]; m[4] = tmp;
    tmp = m[2]; m[2] = m[8]; m[8] = tmp;
    tmp = m[3]; m[3] = m[12]; m[12] = tmp;
    tmp = m[6]; m[6] = m[9]; m[9] = tmp;
    tmp = m[7]; m[7] = m[13]; m[13] = tmp;
    tmp = m[11]; m[11] = m[14]; m[14] = tmp;
    return m;
  },

  transpose3x3 : function(m) {
    return [
      m[0], m[3], m[6],
      m[1], m[4], m[7],
      m[2], m[5], m[8]
    ];
  },

  transpose3x3InPlace : function(m) {
    var tmp = 0.0;
    tmp = m[1]; m[1] = m[3]; m[3] = tmp;
    tmp = m[2]; m[2] = m[6]; m[6] = tmp;
    tmp = m[5]; m[5] = m[7]; m[7] = tmp;
    return m;
  },
}

Vec3 = {
  make : function() { return [0,0,0]; },
  copy : function(v) { return [v[0],v[1],v[2]]; },

  add : function (u,v) {
    return [u[0]+v[0], u[1]+v[1], u[2]+v[2]];
  },

  sub : function (u,v) {
    return [u[0]-v[0], u[1]-v[1], u[2]-v[2]];
  },

  negate : function (u) {
    return [-u[0], -u[1], -u[2]];
  },

  direction : function (u,v) {
    return this.normalizeInPlace(this.sub(u,v));
  },

  normalizeInPlace : function(v) {
    var imag = 1.0 / Math.sqrt(v[0]*v[0] + v[1]*v[1] + v[2]*v[2]);
    v[0] *= imag; v[1] *= imag; v[2] *= imag;
    return v;
  },

  normalize : function(v) {
    return this.normalizeInPlace(this.copy(v));
  },

  scale : function(f, v) {
    return [f*v[0], f*v[1], f*v[2]];
  },

  dot : function(u,v) {
    return u[0]*v[0] + u[1]*v[1] + u[2]*v[2];
  },

  inner : function(u,v) {
    return [u[0]*v[0], u[1]*v[1], u[2]*v[2]];
  },

  cross : function(u,v) {
    return [
      u[1]*v[2] - u[2]*v[1],
      u[2]*v[0] - u[0]*v[2],
      u[0]*v[1] - u[1]*v[0]
    ];
  }
}

Shader = function(gl){
  this.gl = gl;
  this.shaders = [];
  this.uniformLocations = {};
  this.attribLocations = {};
  for (var i=1; i<arguments.length; i++) {
    this.shaders.push(arguments[i]);
  }
}
Shader.prototype = {
  id : null,
  gl : null,
  compiled : false,
  shader : null,
  shaders : [],

  destroy : function() {
    if (this.shader != null) deleteShader(this.gl, this.shader);
  },

  compile : function() {
    this.shader = loadShaderArray(this.gl, this.shaders);
  },

  use : function() {
    if (this.shader == null)
      this.compile();
    this.gl.useProgram(this.shader.program);
  },

  uniform1fv : function(name, value) {
    var loc = this.uniform(name);
    this.gl.uniform1fv(loc, value);
  },

  uniform2fv : function(name, value) {
    var loc = this.uniform(name);
    this.gl.uniform2fv(loc, value);
  },

  uniform3fv : function(name, value) {
    var loc = this.uniform(name);
    this.gl.uniform3fv(loc, value);
  },

  uniform4fv : function(name, value) {
    var loc = this.uniform(name);
    this.gl.uniform4fv(loc, value);
  },

  uniform1f : function(name, value) {
    var loc = this.uniform(name);
    this.gl.uniform1f(loc, value);
  },

  uniform2f : function(name, v1,v2) {
    var loc = this.uniform(name);
    this.gl.uniform2f(loc, v1,v2);
  },

  uniform3f : function(name, v1,v2,v3) {
    var loc = this.uniform(name);
    this.gl.uniform3f(loc, v1,v2,v3);
  },

  uniform4f : function(name, v1,v2,v3,v4) {
    var loc = this.uniform(name);
    this.gl.uniform4f(loc, v1, v2, v3, v4);
  },

  uniform1iv : function(name, value) {
    var loc = this.uniform(name);
    this.gl.uniform1iv(loc, value);
  },

  uniform2iv : function(name, value) {
    var loc = this.uniform(name);
    this.gl.uniform2iv(loc, value);
  },

  uniform3iv : function(name, value) {
    var loc = this.uniform(name);
    this.gl.uniform3iv(loc, value);
  },

  uniform4iv : function(name, value) {
    var loc = this.uniform(name);
    this.gl.uniform4iv(loc, value);
  },

  uniform1i : function(name, value) {
    var loc = this.uniform(name);
    this.gl.uniform1i(loc, value);
  },

  uniform2i : function(name, v1,v2) {
    var loc = this.uniform(name);
    this.gl.uniform2i(loc, v1,v2);
  },

  uniform3i : function(name, v1,v2,v3) {
    var loc = this.uniform(name);
    this.gl.uniform3i(loc, v1,v2,v3);
  },

  uniform4i : function(name, v1,v2,v3,v4) {
    var loc = this.uniform(name);
    this.gl.uniform4i(loc, v1, v2, v3, v4);
  },

  uniformMatrix4fv : function(name, value) {
    var loc = this.uniform(name);
    this.gl.uniformMatrix4fv(loc, false, value);
  },

  uniformMatrix3fv : function(name, value) {
    var loc = this.uniform(name);
    this.gl.uniformMatrix3fv(loc, false, value);
  },

  uniformMatrix2fv : function(name, value) {
    var loc = this.uniform(name);
    this.gl.uniformMatrix2fv(loc, false, value);
  },

  attrib : function(name) {
    if (this.attribLocations[name] == null) {
      var loc = this.gl.getAttribLocation(this.shader.program, name);
      this.attribLocations[name] = loc;
    }
    return this.attribLocations[name];
  },

  uniform : function(name) {
    if (this.uniformLocations[name] == null) {
      var loc = this.gl.getUniformLocation(this.shader.program, name);
      this.uniformLocations[name] = loc;
    }
    return this.uniformLocations[name];
  }
}
Filter = function(gl, shader) {
  Shader.apply(this, arguments);
}
Filter.prototype = new Shader();
Filter.prototype.apply = function(init) {
  this.use();
  var va = this.attrib("Vertex");
  var ta = this.attrib("Tex");
  var vbo = Quad.getCachedVBO(this.gl);
  if (init) init(this);
  vbo.draw(va, null, ta);
}


VBO = function(gl) {
  this.gl = gl;
  this.data = [];
  this.elementsVBO = null;
  for (var i=1; i<arguments.length; i++) {
    if (arguments[i].elements)
      this.elements = arguments[i];
    else
      this.data.push(arguments[i]);
  }
}

VBO.prototype = {
  initialized : false,
  length : 0,
  vbos : null,
  type : 'TRIANGLES',
  elementsVBO : null,
  elements : null,

  setData : function() {
    this.destroy();
    this.data = [];
    for (var i=0; i<arguments.length; i++) {
      if (arguments[i].elements)
        this.elements = arguments[i];
      else
        this.data.push(arguments[i]);
    }
  },

  destroy : function() {
    if (this.vbos != null)
      for (var i=0; i<this.vbos.length; i++)
        this.gl.deleteBuffer(this.vbos[i]);
    if (this.elementsVBO != null)
      this.gl.deleteBuffer(this.elementsVBO);
    this.length = this.elementsLength = 0;
    this.vbos = this.elementsVBO = null;
    this.initialized = false;
  },

  init : function() {
    this.destroy();
    var gl = this.gl;

    gl.getError();
    var vbos = [];
    var length = 0;
    for (var i=0; i<this.data.length; i++)
      vbos.push(gl.createBuffer());
    if (this.elements != null)
      this.elementsVBO = gl.createBuffer();
    try {
      throwError(gl, "genBuffers");
      for (var i = 0; i<this.data.length; i++) {
        var d = this.data[i];
        var dlen = Math.floor(d.data.length / d.size);
        if (i == 0 || dlen < length)
            length = dlen;
        if (!d.floatArray)
          d.floatArray = new Float32Array(d.data);
        gl.bindBuffer(gl.ARRAY_BUFFER, vbos[i]);
        throwError(gl, "bindBuffer");
        gl.bufferData(gl.ARRAY_BUFFER, d.floatArray, gl.STATIC_DRAW);
        throwError(gl, "bufferData");
      }
      if (this.elementsVBO != null) {
        var d = this.elements;
        this.elementsLength = d.data.length;
        this.elementsType = d.type == gl.UNSIGNED_BYTE ? d.type : gl.UNSIGNED_SHORT;
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.elementsVBO);
        throwError(gl, "bindBuffer ELEMENT_ARRAY_BUFFER");
        if (this.elementsType == gl.UNSIGNED_SHORT && !d.ushortArray) {
          d.ushortArray = new Uint16Array(d.data);
          gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, d.ushortArray, gl.STATIC_DRAW);
        } else if (this.elementsType == gl.UNSIGNED_BYTE && !d.ubyteArray) {
          d.ubyteArray = new Uint8Array(d.data);
          gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, d.ubyteArray, gl.STATIC_DRAW);
        }
        throwError(gl, "bufferData ELEMENT_ARRAY_BUFFER");
      }
    } catch(e) {
      for (var i=0; i<vbos.length; i++)
        gl.deleteBuffer(vbos[i]);
      throw(e);
    }

    gl.bindBuffer(gl.ARRAY_BUFFER, null);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, null);

    this.length = length;
    this.vbos = vbos;

    this.initialized = true;
  },

  use : function() {
    if (!this.initialized) this.init();
    var gl = this.gl;
    for (var i=0; i<arguments.length; i++) {
      if (arguments[i] == null || arguments[i] == -1) continue;
      gl.bindBuffer(gl.ARRAY_BUFFER, this.vbos[i]);
      gl.vertexAttribPointer(arguments[i], this.data[i].size, gl.FLOAT, false, 0, 0);
      gl.enableVertexAttribArray(arguments[i]);
    }
    if (this.elementsVBO != null) {
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.elementsVBO);
    }
  },

  draw : function() {
    var args = [];
    this.use.apply(this, arguments);
    var gl = this.gl;
    if (this.elementsVBO != null) {
      gl.drawElements(gl[this.type], this.elementsLength, this.elementsType, 0);
    } else {
      gl.drawArrays(gl[this.type], 0, this.length);
    }
  }
}

FBO = function(gl, width, height, use_depth) {
  this.gl = gl;
  this.width = width;
  this.height = height;
  if (use_depth != null)
    this.useDepth = use_depth;
}
FBO.prototype = {
  initialized : false,
  useDepth : true,
  fbo : null,
  rbo : null,
  texture : null,

  destroy : function() {
    if (this.fbo) this.gl.deleteFramebuffer(this.fbo);
    if (this.rbo) this.gl.deleteRenderbuffer(this.rbo);
    if (this.texture) this.gl.deleteTexture(this.texture);
  },

  init : function() {
    var gl = this.gl;
    var w = this.width, h = this.height;
    var fbo = this.fbo != null ? this.fbo : gl.createFramebuffer();
    var rb;

    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    checkError(gl, "FBO.init bindFramebuffer");
    if (this.useDepth) {
      rb = this.rbo != null ? this.rbo : gl.createRenderbuffer();
      gl.bindRenderbuffer(gl.RENDERBUFFER, rb);
      checkError(gl, "FBO.init bindRenderbuffer");
      gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH_COMPONENT16, w, h);
      checkError(gl, "FBO.init renderbufferStorage");
    }

    var tex = this.texture != null ? this.texture : gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    try {
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, w, h, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
    } catch (e) { // argh, no null texture support
      var tmp = this.getTempCanvas(w,h);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, tmp);
    }
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    checkError(gl, "FBO.init tex");

    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
    checkError(gl, "FBO.init bind tex");

    if (this.useDepth) {
      gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, rb);
      checkError(gl, "FBO.init bind depth buffer");
    }

    var fbstat = gl.checkFramebufferStatus(gl.FRAMEBUFFER);
    if (fbstat != gl.FRAMEBUFFER_COMPLETE) {
      var glv;
      for (var v in gl) {
        try { glv = gl[v]; } catch (e) { glv = null; }
        if (glv == fbstat) { fbstat = v; break; }}
        log("Framebuffer status: " + fbstat);
    }
    checkError(gl, "FBO.init check fbo");

    this.fbo = fbo;
    this.rbo = rb;
    this.texture = tex;
    this.initialized = true;
  },

  getTempCanvas : function(w, h) {
    if (!FBO.tempCanvas) {
      FBO.tempCanvas = document.createElement('canvas');
    }
    FBO.tempCanvas.width = w;
    FBO.tempCanvas.height = h;
    return FBO.tempCanvas;
  },

  use : function() {
    if (!this.initialized) this.init();
    this.gl.bindFramebuffer(this.gl.FRAMEBUFFER, this.fbo);
  }
}

function GLError(err, msg, fileName, lineNumber) {
  this.message = msg;
  this.glError = err;
}

GLError.prototype = new Error();

function makeGLErrorWrapper(gl, fname) {
  return (function() {
    try {
      var rv = gl[fname].apply(gl, arguments);
      var err = gl.getError();
      if (err != gl.NO_ERROR) {
        throw(new GLError(
            err, "GL error "+getGLErrorAsString(gl, err)+" in "+fname));
      }
      return rv;
    } catch (e) {
      if (e.glError !== undefined) {
        throw e;
      }
      throw(new Error("Threw " + e.name +
                      " in " + fname + "\n" +
                      e.message + "\n" +
                      arguments.callee.caller));
    }
  });
}

function wrapGLContext(gl) {
  var wrap = {};
  for (var i in gl) {
    try {
      if (typeof gl[i] == 'function') {
          wrap[i] = makeGLErrorWrapper(gl, i);
      } else {
          wrap[i] = gl[i];
      }
    } catch (e) {
      // log("wrapGLContext: Error accessing " + i);
    }
  }
  wrap.getError = function(){ return gl.getError(); };
  return wrap;
}

function getGLContext(canvas) {
  return canvas.getContext(GL_CONTEXT_ID, {antialias: false});
}

// Assert that f generates a specific GL error.
function assertGLError(gl, err, name, f) {
  if (f == null) { f = name; name = null; }
  var r = false;
  var glErr = 0;
  try { f(); } catch(e) { r=true; glErr = e.glError; }
  if (glErr !== err) {
    if (glErr === undefined) {
      testFailed("assertGLError: UNEXPECTED EXCEPTION", name, f);
    } else {
      testFailed("assertGLError: expected: " + getGLErrorAsString(gl, err) +
                 " actual: " + getGLErrorAsString(gl, glErr), name, f);
    }
    return false;
  }
  return true;
}

// Assert that f generates a GL error from a list.
function assertGLErrorIn(gl, expectedErrorList, name, f) {
  if (f == null) { f = name; name = null; }

  var actualError = 0;
  try {
    f();
  } catch(e) {
    if ('glError' in e) {
      actualError = e.glError;
    } else {
      testFailed("assertGLError: UNEXPCETED EXCEPTION", name, f);
      return false;
    }
  }

  var expectedErrorStrList = [];
  var expectedErrorSet = {};
  for (var i in expectedErrorList) {
    var cur = expectedErrorList[i];
    expectedErrorSet[cur] = true;
    expectedErrorStrList.push(getGLErrorAsString(gl, cur));
  }
  var expectedErrorListStr = "[" + expectedErrorStrList.join(", ") + "]";

  if (actualError in expectedErrorSet) {
    return true;
  }

  testFailed("assertGLError: expected: " + expectedErrorListStr +
             " actual: " + getGLErrorAsString(gl, actualError), name, f);
  return false;
}

// Assert that f generates some GL error. Used in situations where it's
// ambigious which of multiple possible errors will be generated.
function assertSomeGLError(gl, name, f) {
  if (f == null) { f = name; name = null; }
  var r = false;
  var glErr = 0;
  var err = 0;
  try { f(); } catch(e) { r=true; glErr = e.glError; }
  if (glErr === 0) {
    if (glErr === undefined) {
      testFailed("assertGLError: UNEXPECTED EXCEPTION", name, f);
    } else {
      testFailed("assertGLError: expected: " + getGLErrorAsString(gl, err) +
                 " actual: " + getGLErrorAsString(gl, glErr), name, f);
    }
    return false;
  }
  return true;
}

// Assert that f throws an exception but does not generate a GL error.
function assertThrowNoGLError(gl, name, f) {
  if (f == null) { f = name; name = null; }
  var r = false;
  var glErr = undefined;
  var exp;
  try { f(); } catch(e) { r=true; glErr = e.glError; exp = e;}
  if (!r) {
    testFailed(
      "assertThrowNoGLError: should have thrown exception", name, f);
    return false;
  } else {
    if (glErr !== undefined) {
      testFailed(
        "assertThrowNoGLError: should be no GL error but generated: " +
        getGLErrorAsString(gl, glErr), name, f);
      return false;
    }
  }
  testPassed("assertThrowNoGLError", name, f);
  return true;
}

Quad = {
  vertices : [
    -1,-1,0,
    1,-1,0,
    -1,1,0,
    1,-1,0,
    1,1,0,
    -1,1,0
  ],
  normals : [
    0,0,-1,
    0,0,-1,
    0,0,-1,
    0,0,-1,
    0,0,-1,
    0,0,-1
  ],
  texcoords : [
    0,0,
    1,0,
    0,1,
    1,0,
    1,1,
    0,1
  ],
  indices : [0,1,2,1,5,2],
  makeVBO : function(gl) {
    return new VBO(gl,
        {size:3, data: Quad.vertices},
        {size:3, data: Quad.normals},
        {size:2, data: Quad.texcoords}
    )
  },
  cache: {},
  getCachedVBO : function(gl) {
    if (!this.cache[gl])
      this.cache[gl] = this.makeVBO(gl);
    return this.cache[gl];
  }
}
Cube = {
  vertices : [  0.5, -0.5,  0.5, // +X
                0.5, -0.5, -0.5,
                0.5,  0.5, -0.5,
                0.5,  0.5,  0.5,

                0.5,  0.5,  0.5, // +Y
                0.5,  0.5, -0.5,
                -0.5,  0.5, -0.5,
                -0.5,  0.5,  0.5,

                0.5,  0.5,  0.5, // +Z
                -0.5,  0.5,  0.5,
                -0.5, -0.5,  0.5,
                0.5, -0.5,  0.5,

                -0.5, -0.5,  0.5, // -X
                -0.5,  0.5,  0.5,
                -0.5,  0.5, -0.5,
                -0.5, -0.5, -0.5,

                -0.5, -0.5,  0.5, // -Y
                -0.5, -0.5, -0.5,
                0.5, -0.5, -0.5,
                0.5, -0.5,  0.5,

                -0.5, -0.5, -0.5, // -Z
                -0.5,  0.5, -0.5,
                0.5,  0.5, -0.5,
                0.5, -0.5, -0.5,
      ],

  normals : [ 1, 0, 0,
              1, 0, 0,
              1, 0, 0,
              1, 0, 0,

              0, 1, 0,
              0, 1, 0,
              0, 1, 0,
              0, 1, 0,

              0, 0, 1,
              0, 0, 1,
              0, 0, 1,
              0, 0, 1,

              -1, 0, 0,
              -1, 0, 0,
              -1, 0, 0,
              -1, 0, 0,

              0,-1, 0,
              0,-1, 0,
              0,-1, 0,
              0,-1, 0,

              0, 0,-1,
              0, 0,-1,
              0, 0,-1,
              0, 0,-1
      ],

  indices : [],
  create : function(){
    for (var i = 0; i < 6; i++) {
      Cube.indices.push(i*4 + 0);
      Cube.indices.push(i*4 + 1);
      Cube.indices.push(i*4 + 3);
      Cube.indices.push(i*4 + 1);
      Cube.indices.push(i*4 + 2);
      Cube.indices.push(i*4 + 3);
    }
  },

  makeVBO : function(gl) {
    return new VBO(gl,
        {size:3, data: Cube.vertices},
        {size:3, data: Cube.normals},
        {elements: true, data: Cube.indices}
    )
  },
  cache : {},
  getCachedVBO : function(gl) {
    if (!this.cache[gl])
      this.cache[gl] = this.makeVBO(gl);
    return this.cache[gl];
  }
}
Cube.create();

Sphere = {
  vertices : [],
  normals : [],
  indices : [],
  create : function(){
    var r = 0.75;
    function vert(theta, phi)
    {
      var r = 0.75;
      var x, y, z, nx, ny, nz;

      nx = Math.sin(theta) * Math.cos(phi);
      ny = Math.sin(phi);
      nz = Math.cos(theta) * Math.cos(phi);
      Sphere.normals.push(nx);
      Sphere.normals.push(ny);
      Sphere.normals.push(nz);

      x = r * Math.sin(theta) * Math.cos(phi);
      y = r * Math.sin(phi);
      z = r * Math.cos(theta) * Math.cos(phi);
      Sphere.vertices.push(x);
      Sphere.vertices.push(y);
      Sphere.vertices.push(z);
    }
    for (var phi = -Math.PI/2; phi < Math.PI/2; phi += Math.PI/20) {
      var phi2 = phi + Math.PI/20;
      for (var theta = -Math.PI/2; theta <= Math.PI/2; theta += Math.PI/20) {
        vert(theta, phi);
        vert(theta, phi2);
      }
    }
  }
}

Sphere.create();

initGL_CONTEXT_ID = function(){
  var c = document.createElement('canvas');
  var contextNames = ['webgl', 'experimental-webgl'];
  GL_CONTEXT_ID = null;
  for (var i=0; i<contextNames.length; i++) {
    try {
      if (c.getContext(contextNames[i])) {
        GL_CONTEXT_ID = contextNames[i];
        break;
      }
    } catch (e) {
    }
  }
  if (!GL_CONTEXT_ID) {
    log("No WebGL context found. Unable to run tests.");
  }
}

initGL_CONTEXT_ID();
