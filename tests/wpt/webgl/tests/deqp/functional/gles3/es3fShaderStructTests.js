/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
 *
 * Copyright 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the 'License');
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an 'AS IS' BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

'use strict';
goog.provide('functional.gles3.es3fShaderStructTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
// goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTexture');
goog.require('modules.shared.glsShaderRenderCase');
goog.require('framework.common.tcuStringTemplate');

goog.scope(function() {
	var es3fShaderStructTests = functional.gles3.es3fShaderStructTests;
	var tcuTestCase = framework.common.tcuTestCase;
	var tcuTexture = framework.common.tcuTexture;
	var tcuTextureUtil = framework.common.tcuTextureUtil;
	var deMath = framework.delibs.debase.deMath;
	// var gluShaderUtil = framework.opengl.gluShaderUtil;
	var glsShaderRenderCase = modules.shared.glsShaderRenderCase;
	var gluTexture = framework.opengl.gluTexture;
	var tcuStringTemplate = framework.common.tcuStringTemplate;

	/** @typedef {function(WebGLProgram, Array<number>)} */
	es3fShaderStructTests.SetupUniformsFunc;

	/** @const {number} */ es3fShaderStructTests.TEXTURE_BRICK = 0;


	/**
	 * @constructor
	 * @extends {glsShaderRenderCase.ShaderRenderCase}
	 * @param  {string} name
	 * @param  {string} description
	 * @param  {boolean} isVertexCase
	 * @param  {boolean} usesTextures
	 * @param  {glsShaderRenderCase.ShaderEvalFunc} evalFunc
	 * @param  {?es3fShaderStructTests.SetupUniformsFunc} setupUniformsFunc
	 * @param  {string} vertShaderSource
	 * @param  {string} fragShaderSource
	 */
	es3fShaderStructTests.ShaderStructCase = function(name, description, isVertexCase, usesTextures, evalFunc, setupUniformsFunc, vertShaderSource, fragShaderSource) {
		glsShaderRenderCase.ShaderRenderCase.call(this, name, description, isVertexCase, evalFunc);
		/** @type {?es3fShaderStructTests.SetupUniformsFunc} */ this.m_setupUniforms = setupUniformsFunc;
		/** @type {boolean} */ this.m_usesTexture = usesTextures;
		/** @type {gluTexture.Texture2D} */ this.m_brickTexture = null;
		/** @type {string} */ this.m_vertShaderSource = vertShaderSource;
		/** @type {string} */ this.m_fragShaderSource = fragShaderSource;
	};

	es3fShaderStructTests.ShaderStructCase.prototype = Object.create(glsShaderRenderCase.ShaderRenderCase.prototype);
	es3fShaderStructTests.ShaderStructCase.prototype.constructor = es3fShaderStructTests.ShaderStructCase;

	es3fShaderStructTests.ShaderStructCase.prototype.init = function() {
		if (this.m_usesTexture) {
			this.m_brickTexture = gluTexture.texture2DFromInternalFormat(gl, gl.RGBA8, 256, 256);
			var ref = this.m_brickTexture.getRefTexture();
			for (var i = 0 ; i < ref.getNumLevels(); i++) {
				ref.allocLevel(i);
				tcuTextureUtil.fillWithGrid(ref.getLevel(i), 8, [0.2, 0.7, 0.1, 1.0], [0.7, 0.1, 0.5, 0.8]);
			}
			this.m_brickTexture.upload();

			this.m_textures.push(new glsShaderRenderCase.TextureBinding(
				this.m_brickTexture,
				new tcuTexture.Sampler(
					tcuTexture.WrapMode.CLAMP_TO_EDGE,
					tcuTexture.WrapMode.CLAMP_TO_EDGE,
					tcuTexture.WrapMode.CLAMP_TO_EDGE,
					tcuTexture.FilterMode.LINEAR,
					tcuTexture.FilterMode.LINEAR)));

			assertMsgOptions(this.m_textures.length === 1, 'Only one texture required', false, true);
		}
		this.postinit();
	};

	es3fShaderStructTests.ShaderStructCase.prototype.deinit = function() {
		glsShaderRenderCase.ShaderRenderCase.prototype.deinit.call(this);
		this.m_brickTexture = null;
	};

	/**
	 * @param  {WebGLProgram} programID
	 * @param  {Array<number>} constCoords
	 */
	es3fShaderStructTests.ShaderStructCase.prototype.setupUniforms = function(programID, constCoords) {
		glsShaderRenderCase.ShaderRenderCase.prototype.setupUniforms.call(this, programID, constCoords);
		if (this.m_setupUniforms)
			this.m_setupUniforms(programID, constCoords);
	};

	/**
	 * @param {string} name
	 * @param {string} description
	 * @param {boolean} isVertexCase
	 * @param {boolean} usesTextures
	 * @param {glsShaderRenderCase.ShaderEvalFunc} evalFunc
	 * @param {?es3fShaderStructTests.SetupUniformsFunc} setupUniforms
	 * @param {string} shaderSrc
	 */
	es3fShaderStructTests.ShaderStructCase.createStructCase = function(name, description, isVertexCase, usesTextures, evalFunc, setupUniforms, shaderSrc) {
		/** @type {string} */ var defaultVertSrc =
			'#version 300 es\n' +
			'in highp vec4 a_position;\n' +
			'in highp vec4 a_coords;\n' +
			'out mediump vec4 v_coords;\n\n' +
			'void main (void)\n' +
			'{\n' +
			'	v_coords = a_coords;\n' +
			'	gl_Position = a_position;\n' +
			'}\n';
		/** @type {string} */ var defaultFragSrc =
			'#version 300 es\n' +
			'in mediump vec4 v_color;\n' +
			'layout(location = 0) out mediump vec4 o_color;\n\n' +
			'void main (void)\n' +
			'{\n' +
			'	o_color = v_color;\n' +
			'}\n';

		// Fill in specialization parameters.
		var spParams = {};
		if (isVertexCase) {
			spParams["HEADER"] =
				"#version 300 es\n" +
				"in highp vec4 a_position;\n" +
				"in highp vec4 a_coords;\n" +
				"out mediump vec4 v_color;";
			spParams["COORDS"]		= "a_coords";
			spParams["DST"]			= "v_color";
			spParams["ASSIGN_POS"]	= "gl_Position = a_position;";
		}
		else {
			spParams["HEADER"]	=
				"#version 300 es\n" +
				"in mediump vec4 v_coords;\n" +
				"layout(location = 0) out mediump vec4 o_color;";
			spParams["COORDS"]			= "v_coords";
			spParams["DST"]				= "o_color";
			spParams["ASSIGN_POS"]		= "";
		}

		if (isVertexCase)
			return new es3fShaderStructTests.ShaderStructCase(name, description, isVertexCase, usesTextures, evalFunc, setupUniforms, tcuStringTemplate.specialize(shaderSrc, spParams), defaultFragSrc);
		else
			return new es3fShaderStructTests.ShaderStructCase(name, description, isVertexCase, usesTextures, evalFunc, setupUniforms, defaultVertSrc, tcuStringTemplate.specialize(shaderSrc, spParams));
	};

	/**
	 * @constructor
	 * @extends {tcuTestCase.DeqpTest}
	 */
	es3fShaderStructTests.LocalStructTests = function() {
		tcuTestCase.DeqpTest.call(this, 'local', 'Local structs');
		this.makeExecutable();
	};

	es3fShaderStructTests.LocalStructTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
	es3fShaderStructTests.LocalStructTests.prototype.constructor = es3fShaderStructTests.LocalStructTests;

	es3fShaderStructTests.LocalStructTests.prototype.init = function() {
		var currentCtx = this;
		function LocalStructCase(name, description, shaderSource, evalFunction) {
			currentCtx.addChild(es3fShaderStructTests.ShaderStructCase.createStructCase(name + "_vertex", description, true, false, evalFunction, null, shaderSource));
			currentCtx.addChild(es3fShaderStructTests.ShaderStructCase.createStructCase(name + "_fragment", description, false, false, evalFunction, null, shaderSource));
		};

		LocalStructCase('basic', 'Basic struct usage',
			'${HEADER}\n' +
			'uniform int ui_one;\n' +
			'\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump vec3	b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s = S(${COORDS}.x, vec3(0.0), ui_one);\n' +
			'	s.b = ${COORDS}.yzw;\n' +
			'	${DST} = vec4(s.a, s.b.x, s.b.y, s.c);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[0];
				c.color[1] = c.coords[1];
				c.color[2] = c.coords[2];
			});

		LocalStructCase('nested', "Nested struct",
			'${HEADER}\n' +
			"uniform int ui_zero;\n" +
			"uniform int ui_one;\n" +
			"\n" +
			"struct T {\n" +
			"	int				a;\n" +
			"	mediump vec2	b;\n" +
			"};\n" +
			"struct S {\n" +
			'	mediump float	a;\n' +
			'	T				b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s = S(${COORDS}.x, T(0, vec2(0.0)), ui_one);\n' +
			'	s.b = T(ui_zero, ${COORDS}.yz);\n' +
			'	${DST} = vec4(s.a, s.b.b, s.b.a + s.c);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[0];
				c.color[1] = c.coords[1];
				c.color[2] = c.coords[2];
			});

		LocalStructCase('array_member', "Struct with array member",
			'${HEADER}\n' +
			'uniform int ui_one;\n' +
			'\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump float	b[3];\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s;\n' +
			'	s.a = ${COORDS}.w;\n' +
			'	s.c = ui_one;\n' +
			'	s.b[0] = ${COORDS}.z;\n' +
			'	s.b[1] = ${COORDS}.y;\n' +
			'	s.b[2] = ${COORDS}.x;\n' +
			'	${DST} = vec4(s.a, s.b[0], s.b[1], s.c);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[3];
				c.color[1] = c.coords[2];
				c.color[2] = c.coords[1];
			});

		LocalStructCase('array_member_dynamic_index', "Struct with array member, dynamic indexing",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump float	b[3];\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s;\n' +
			'	s.a = ${COORDS}.w;\n' +
			'	s.c = ui_one;\n' +
			'	s.b[0] = ${COORDS}.z;\n' +
			'	s.b[1] = ${COORDS}.y;\n' +
			'	s.b[2] = ${COORDS}.x;\n' +
			'	${DST} = vec4(s.b[ui_one], s.b[ui_zero], s.b[ui_two], s.c);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[1];
				c.color[1] = c.coords[2];
				c.color[2] = c.coords[0];
			});

		LocalStructCase('struct_array', "Struct array",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump int		b;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s[3];\n' +
			'	s[0] = S(${COORDS}.x, ui_zero);\n' +
			'	s[1].a = ${COORDS}.y;\n' +
			'	s[1].b = ui_one;\n' +
			'	s[2] = S(${COORDS}.z, ui_two);\n' +
			'	${DST} = vec4(s[2].a, s[1].a, s[0].a, s[2].b - s[1].b + s[0].b);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[2];
				c.color[1] = c.coords[1];
				c.color[2] = c.coords[0];
			});

		LocalStructCase('struct_array_dynamic_index', "Struct array with dynamic indexing",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump int		b;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s[3];\n' +
			'	s[0] = S(${COORDS}.x, ui_zero);\n' +
			'	s[1].a = ${COORDS}.y;\n' +
			'	s[1].b = ui_one;\n' +
			'	s[2] = S(${COORDS}.z, ui_two);\n' +
			'	${DST} = vec4(s[ui_two].a, s[ui_one].a, s[ui_zero].a, s[ui_two].b - s[ui_one].b + s[ui_zero].b);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[2];
				c.color[1] = c.coords[1];
				c.color[2] = c.coords[0];
			});

		LocalStructCase('nested_struct_array', "Nested struct array",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'uniform mediump float uf_two;\n' +
			'uniform mediump float uf_three;\n' +
			'uniform mediump float uf_four;\n' +
			'uniform mediump float uf_half;\n' +
			'uniform mediump float uf_third;\n' +
			'uniform mediump float uf_fourth;\n' +
			'\n' +
			'struct T {\n' +
			'	mediump float	a;\n' +
			'	mediump vec2	b[2];\n' +
			'};\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	T				b[3];\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s[2];\n' +
			'\n' +
			'	// S[0]\n' +
			'	s[0].a         = ${COORDS}.x;\n' +
			'	s[0].b[0].a    = uf_half;\n' +
			'	s[0].b[0].b[0] = ${COORDS}.xy;\n' +
			'	s[0].b[0].b[1] = ${COORDS}.zw;\n' +
			'	s[0].b[1].a    = uf_third;\n' +
			'	s[0].b[1].b[0] = ${COORDS}.zw;\n' +
			'	s[0].b[1].b[1] = ${COORDS}.xy;\n' +
			'	s[0].b[2].a    = uf_fourth;\n' +
			'	s[0].b[2].b[0] = ${COORDS}.xz;\n' +
			'	s[0].b[2].b[1] = ${COORDS}.yw;\n' +
			'	s[0].c         = ui_zero;\n' +
			'\n' +
			'	// S[1]\n' +
			'	s[1].a         = ${COORDS}.w;\n' +
			'	s[1].b[0].a    = uf_two;\n' +
			'	s[1].b[0].b[0] = ${COORDS}.xx;\n' +
			'	s[1].b[0].b[1] = ${COORDS}.yy;\n' +
			'	s[1].b[1].a    = uf_three;\n' +
			'	s[1].b[1].b[0] = ${COORDS}.zz;\n' +
			'	s[1].b[1].b[1] = ${COORDS}.ww;\n' +
			'	s[1].b[2].a    = uf_four;\n' +
			'	s[1].b[2].b[0] = ${COORDS}.yx;\n' +
			'	s[1].b[2].b[1] = ${COORDS}.wz;\n' +
			'	s[1].c         = ui_one;\n' +
			'\n' +
			'	mediump float r = (s[0].b[1].b[0].x + s[1].b[2].b[1].y) * s[0].b[0].a; // (z + z) * 0.5\n' +
			'	mediump float g = s[1].b[0].b[0].y * s[0].b[2].a * s[1].b[2].a; // x * 0.25 * 4\n' +
			'	mediump float b = (s[0].b[2].b[1].y + s[0].b[1].b[0].y + s[1].a) * s[0].b[1].a; // (w + w + w) * 0.333\n' +
			'	mediump float a = float(s[0].c) + s[1].b[2].a - s[1].b[1].a; // 0 + 4.0 - 3.0\n' +
			'	${DST} = vec4(r, g, b, a);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[2];
				c.color[1] = c.coords[0];
				c.color[2] = c.coords[3];
			});

		LocalStructCase('nested_struct_array_dynamic_index', "Nested struct array with dynamic indexing",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'uniform mediump float uf_two;\n' +
			'uniform mediump float uf_three;\n' +
			'uniform mediump float uf_four;\n' +
			'uniform mediump float uf_half;\n' +
			'uniform mediump float uf_third;\n' +
			'uniform mediump float uf_fourth;\n' +
			'\n' +
			'struct T {\n' +
			'	mediump float	a;\n' +
			'	mediump vec2	b[2];\n' +
			'};\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	T				b[3];\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s[2];\n' +
			'\n' +
			'	// S[0]\n' +
			'	s[0].a         = ${COORDS}.x;\n' +
			'	s[0].b[0].a    = uf_half;\n' +
			'	s[0].b[0].b[0] = ${COORDS}.xy;\n' +
			'	s[0].b[0].b[1] = ${COORDS}.zw;\n' +
			'	s[0].b[1].a    = uf_third;\n' +
			'	s[0].b[1].b[0] = ${COORDS}.zw;\n' +
			'	s[0].b[1].b[1] = ${COORDS}.xy;\n' +
			'	s[0].b[2].a    = uf_fourth;\n' +
			'	s[0].b[2].b[0] = ${COORDS}.xz;\n' +
			'	s[0].b[2].b[1] = ${COORDS}.yw;\n' +
			'	s[0].c         = ui_zero;\n' +
			'\n' +
			'	// S[1]\n' +
			'	s[1].a         = ${COORDS}.w;\n' +
			'	s[1].b[0].a    = uf_two;\n' +
			'	s[1].b[0].b[0] = ${COORDS}.xx;\n' +
			'	s[1].b[0].b[1] = ${COORDS}.yy;\n' +
			'	s[1].b[1].a    = uf_three;\n' +
			'	s[1].b[1].b[0] = ${COORDS}.zz;\n' +
			'	s[1].b[1].b[1] = ${COORDS}.ww;\n' +
			'	s[1].b[2].a    = uf_four;\n' +
			'	s[1].b[2].b[0] = ${COORDS}.yx;\n' +
			'	s[1].b[2].b[1] = ${COORDS}.wz;\n' +
			'	s[1].c         = ui_one;\n' +
			'\n' +
			'	mediump float r = (s[0].b[ui_one].b[ui_one-1].x + s[ui_one].b[ui_two].b[ui_zero+1].y) * s[0].b[0].a; // (z + z) * 0.5\n' +
			'	mediump float g = s[ui_two-1].b[ui_two-2].b[ui_zero].y * s[0].b[ui_two].a * s[ui_one].b[2].a; // x * 0.25 * 4\n' +
			'	mediump float b = (s[ui_zero].b[ui_one+1].b[1].y + s[0].b[ui_one*ui_one].b[0].y + s[ui_one].a) * s[0].b[ui_two-ui_one].a; // (w + w + w) * 0.333\n' +
			'	mediump float a = float(s[ui_zero].c) + s[ui_one-ui_zero].b[ui_two].a - s[ui_zero+ui_one].b[ui_two-ui_one].a; // 0 + 4.0 - 3.0\n' +
			'	${DST} = vec4(r, g, b, a);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[2];
				c.color[1] = c.coords[0];
				c.color[2] = c.coords[3];
			});

		LocalStructCase('parameter', "Struct as a function parameter",
			'${HEADER}\n' +
			'uniform int ui_one;\n' +
			'\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump vec3	b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'mediump vec4 myFunc (S s)\n' +
			'{\n' +
			'	return vec4(s.a, s.b.x, s.b.y, s.c);\n' +
			'}\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s = S(${COORDS}.x, vec3(0.0), ui_one);\n' +
			'	s.b = ${COORDS}.yzw;\n' +
			'	${DST} = myFunc(s);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[0];
				c.color[1] = c.coords[1];
				c.color[2] = c.coords[2];
			});

		LocalStructCase('parameter_nested', "Nested struct as a function parameter",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'\n' +
			'struct T {\n' +
			'	int				a;\n' +
			'	mediump vec2	b;\n' +
			'};\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	T				b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'mediump vec4 myFunc (S s)\n' +
			'{\n' +
			'	return vec4(s.a, s.b.b, s.b.a + s.c);\n' +
			'}\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s = S(${COORDS}.x, T(0, vec2(0.0)), ui_one);\n' +
			'	s.b = T(ui_zero, ${COORDS}.yz);\n' +
			'	${DST} = myFunc(s);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[0];
				c.color[1] = c.coords[1];
				c.color[2] = c.coords[2];
			});

		LocalStructCase('return', "Struct as a return value",
			'${HEADER}\n' +
			'uniform int ui_one;\n' +
			'\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump vec3	b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'S myFunc (void)\n' +
			'{\n' +
			'	S s = S(${COORDS}.x, vec3(0.0), ui_one);\n' +
			'	s.b = ${COORDS}.yzw;\n' +
			'	return s;\n' +
			'}\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s = myFunc();\n' +
			'	${DST} = vec4(s.a, s.b.x, s.b.y, s.c);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[0];
				c.color[1] = c.coords[1];
				c.color[2] = c.coords[2];
			});

		LocalStructCase('return_nested', "Nested struct",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'\n' +
			'struct T {\n' +
			'	int				a;\n' +
			'	mediump vec2	b;\n' +
			'};\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	T				b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'S myFunc (void)\n' +
			'{\n' +
			'	S s = S(${COORDS}.x, T(0, vec2(0.0)), ui_one);\n' +
			'	s.b = T(ui_zero, ${COORDS}.yz);\n' +
			'	return s;\n' +
			'}\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s = myFunc();\n' +
			'	${DST} = vec4(s.a, s.b.b, s.b.a + s.c);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[0];
				c.color[1] = c.coords[1];
				c.color[2] = c.coords[2];
			});

		LocalStructCase('conditional_assignment', "Conditional struct assignment",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform mediump float uf_one;\n' +
			'\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump vec3	b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s = S(${COORDS}.x, ${COORDS}.yzw, ui_zero);\n' +
			'	if (uf_one > 0.0)\n' +
			'		s = S(${COORDS}.w, ${COORDS}.zyx, ui_one);\n' +
			'	${DST} = vec4(s.a, s.b.xy, s.c);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[3];
				c.color[1] = c.coords[2];
				c.color[2] = c.coords[1];
			});

		LocalStructCase('loop_assignment', "Struct assignment in loop",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump vec3	b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s = S(${COORDS}.x, ${COORDS}.yzw, ui_zero);\n' +
			'	for (int i = 0; i < 3; i++)\n' +
			'	{\n' +
			'		if (i == 1)\n' +
			'			s = S(${COORDS}.w, ${COORDS}.zyx, ui_one);\n' +
			'	}\n' +
			'	${DST} = vec4(s.a, s.b.xy, s.c);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[3];
				c.color[1] = c.coords[2];
				c.color[2] = c.coords[1];
			});

		LocalStructCase('dynamic_loop_assignment', "Struct assignment in loop",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_three;\n' +
			'\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump vec3	b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s = S(${COORDS}.x, ${COORDS}.yzw, ui_zero);\n' +
			'	for (int i = 0; i < ui_three; i++)\n' +
			'	{\n' +
			'		if (i == ui_one)\n' +
			'			s = S(${COORDS}.w, ${COORDS}.zyx, ui_one);\n' +
			'	}\n' +
			'	${DST} = vec4(s.a, s.b.xy, s.c);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[3];
				c.color[1] = c.coords[2];
				c.color[2] = c.coords[1];
			});

		LocalStructCase('nested_conditional_assignment', "Conditional assignment of nested struct",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform mediump float uf_one;\n' +
			'\n' +
			'struct T {\n' +
			'	int				a;\n' +
			'	mediump vec2	b;\n' +
			'};\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	T				b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s = S(${COORDS}.x, T(ui_one, ${COORDS}.yz), ui_one);\n' +
			'	if (uf_one > 0.0)\n' +
			'		s.b = T(ui_zero, ${COORDS}.zw);\n' +
			'	${DST} = vec4(s.a, s.b.b, s.c - s.b.a);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[0];
				c.color[1] = c.coords[2];
				c.color[2] = c.coords[3];
			});

		LocalStructCase('nested_loop_assignment', "Nested struct assignment in loop",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform mediump float uf_one;\n' +
			'\n' +
			'struct T {\n' +
			'	int				a;\n' +
			'	mediump vec2	b;\n' +
			'};\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	T				b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s = S(${COORDS}.x, T(ui_one, ${COORDS}.yz), ui_one);\n' +
			'	for (int i = 0; i < 3; i++)\n' +
			'	{\n' +
			'		if (i == 1)\n' +
			'			s.b = T(ui_zero, ${COORDS}.zw);\n' +
			'	}\n' +
			'	${DST} = vec4(s.a, s.b.b, s.c - s.b.a);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[0];
				c.color[1] = c.coords[2];
				c.color[2] = c.coords[3];
			});

		LocalStructCase('nested_dynamic_loop_assignment', "Nested struct assignment in dynamic loop",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_three;\n' +
			'uniform mediump float uf_one;\n' +
			'\n' +
			'struct T {\n' +
			'	int				a;\n' +
			'	mediump vec2	b;\n' +
			'};\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	T				b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s = S(${COORDS}.x, T(ui_one, ${COORDS}.yz), ui_one);\n' +
			'	for (int i = 0; i < ui_three; i++)\n' +
			'	{\n' +
			'		if (i == ui_one)\n' +
			'			s.b = T(ui_zero, ${COORDS}.zw);\n' +
			'	}\n' +
			'	${DST} = vec4(s.a, s.b.b, s.c - s.b.a);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[0];
				c.color[1] = c.coords[2];
				c.color[2] = c.coords[3];
			});

		LocalStructCase('loop_struct_array', "Struct array usage in loop",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump int		b;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s[3];\n' +
			'	s[0] = S(${COORDS}.x, ui_zero);\n' +
			'	s[1].a = ${COORDS}.y;\n' +
			'	s[1].b = -ui_one;\n' +
			'	s[2] = S(${COORDS}.z, ui_two);\n' +
			'\n' +
			'	mediump float rgb[3];\n' +
			'	int alpha = 0;\n' +
			'	for (int i = 0; i < 3; i++)\n' +
			'	{\n' +
			'		rgb[i] = s[2-i].a;\n' +
			'		alpha += s[i].b;\n' +
			'	}\n' +
			'	${DST} = vec4(rgb[0], rgb[1], rgb[2], alpha);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[2];
				c.color[1] = c.coords[1];
				c.color[2] = c.coords[0];
			});

		LocalStructCase('loop_nested_struct_array', "Nested struct array usage in loop",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'uniform mediump float uf_two;\n' +
			'uniform mediump float uf_three;\n' +
			'uniform mediump float uf_four;\n' +
			'uniform mediump float uf_half;\n' +
			'uniform mediump float uf_third;\n' +
			'uniform mediump float uf_fourth;\n' +
			'uniform mediump float uf_sixth;\n' +
			'\n' +
			'struct T {\n' +
			'	mediump float	a;\n' +
			'	mediump vec2	b[2];\n' +
			'};\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	T				b[3];\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s[2];\n' +
			'\n' +
			'	// S[0]\n' +
			'	s[0].a         = ${COORDS}.x;\n' +
			'	s[0].b[0].a    = uf_half;\n' +
			'	s[0].b[0].b[0] = ${COORDS}.yx;\n' +
			'	s[0].b[0].b[1] = ${COORDS}.zx;\n' +
			'	s[0].b[1].a    = uf_third;\n' +
			'	s[0].b[1].b[0] = ${COORDS}.yy;\n' +
			'	s[0].b[1].b[1] = ${COORDS}.wy;\n' +
			'	s[0].b[2].a    = uf_fourth;\n' +
			'	s[0].b[2].b[0] = ${COORDS}.zx;\n' +
			'	s[0].b[2].b[1] = ${COORDS}.zy;\n' +
			'	s[0].c         = ui_zero;\n' +
			'\n' +
			'	// S[1]\n' +
			'	s[1].a         = ${COORDS}.w;\n' +
			'	s[1].b[0].a    = uf_two;\n' +
			'	s[1].b[0].b[0] = ${COORDS}.zx;\n' +
			'	s[1].b[0].b[1] = ${COORDS}.zy;\n' +
			'	s[1].b[1].a    = uf_three;\n' +
			'	s[1].b[1].b[0] = ${COORDS}.zz;\n' +
			'	s[1].b[1].b[1] = ${COORDS}.ww;\n' +
			'	s[1].b[2].a    = uf_four;\n' +
			'	s[1].b[2].b[0] = ${COORDS}.yx;\n' +
			'	s[1].b[2].b[1] = ${COORDS}.wz;\n' +
			'	s[1].c         = ui_one;\n' +
			'\n' +
			'	mediump float r = 0.0; // (x*3 + y*3) / 6.0\n' +
			'	mediump float g = 0.0; // (y*3 + z*3) / 6.0\n' +
			'	mediump float b = 0.0; // (z*3 + w*3) / 6.0\n' +
			'	mediump float a = 1.0;\n' +
			'	for (int i = 0; i < 2; i++)\n' +
			'	{\n' +
			'		for (int j = 0; j < 3; j++)\n' +
			'		{\n' +
			'			r += s[0].b[j].b[i].y;\n' +
			'			g += s[i].b[j].b[0].x;\n' +
			'			b += s[i].b[j].b[1].x;\n' +
			'			a *= s[i].b[j].a;\n' +
			'		}\n' +
			'	}\n' +
			'	${DST} = vec4(r*uf_sixth, g*uf_sixth, b*uf_sixth, a);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = (c.coords[0] + c.coords[1]) * 0.5;
				c.color[1] = (c.coords[1] + c.coords[2]) * 0.5;
				c.color[2] = (c.coords[2] + c.coords[3]) * 0.5;
			});

		LocalStructCase('dynamic_loop_struct_array', "Struct array usage in dynamic loop",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'uniform int ui_three;\n' +
			'\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump int		b;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s[3];\n' +
			'	s[0] = S(${COORDS}.x, ui_zero);\n' +
			'	s[1].a = ${COORDS}.y;\n' +
			'	s[1].b = -ui_one;\n' +
			'	s[2] = S(${COORDS}.z, ui_two);\n' +
			'\n' +
			'	mediump float rgb[3];\n' +
			'	int alpha = 0;\n' +
			'	for (int i = 0; i < ui_three; i++)\n' +
			'	{\n' +
			'		rgb[i] = s[2-i].a;\n' +
			'		alpha += s[i].b;\n' +
			'	}\n' +
			'	${DST} = vec4(rgb[0], rgb[1], rgb[2], alpha);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = c.coords[2];
				c.color[1] = c.coords[1];
				c.color[2] = c.coords[0];
			});

		LocalStructCase('dynamic_loop_nested_struct_array', "Nested struct array usage in dynamic loop",
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'uniform int ui_three;\n' +
			'uniform mediump float uf_two;\n' +
			'uniform mediump float uf_three;\n' +
			'uniform mediump float uf_four;\n' +
			'uniform mediump float uf_half;\n' +
			'uniform mediump float uf_third;\n' +
			'uniform mediump float uf_fourth;\n' +
			'uniform mediump float uf_sixth;\n' +
			'\n' +
			'struct T {\n' +
			'	mediump float	a;\n' +
			'	mediump vec2	b[2];\n' +
			'};\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	T				b[3];\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S s[2];\n' +
			'\n' +
			'	// S[0]\n' +
			'	s[0].a         = ${COORDS}.x;\n' +
			'	s[0].b[0].a    = uf_half;\n' +
			'	s[0].b[0].b[0] = ${COORDS}.yx;\n' +
			'	s[0].b[0].b[1] = ${COORDS}.zx;\n' +
			'	s[0].b[1].a    = uf_third;\n' +
			'	s[0].b[1].b[0] = ${COORDS}.yy;\n' +
			'	s[0].b[1].b[1] = ${COORDS}.wy;\n' +
			'	s[0].b[2].a    = uf_fourth;\n' +
			'	s[0].b[2].b[0] = ${COORDS}.zx;\n' +
			'	s[0].b[2].b[1] = ${COORDS}.zy;\n' +
			'	s[0].c         = ui_zero;\n' +
			'\n' +
			'	// S[1]\n' +
			'	s[1].a         = ${COORDS}.w;\n' +
			'	s[1].b[0].a    = uf_two;\n' +
			'	s[1].b[0].b[0] = ${COORDS}.zx;\n' +
			'	s[1].b[0].b[1] = ${COORDS}.zy;\n' +
			'	s[1].b[1].a    = uf_three;\n' +
			'	s[1].b[1].b[0] = ${COORDS}.zz;\n' +
			'	s[1].b[1].b[1] = ${COORDS}.ww;\n' +
			'	s[1].b[2].a    = uf_four;\n' +
			'	s[1].b[2].b[0] = ${COORDS}.yx;\n' +
			'	s[1].b[2].b[1] = ${COORDS}.wz;\n' +
			'	s[1].c         = ui_one;\n' +
			'\n' +
			'	mediump float r = 0.0; // (x*3 + y*3) / 6.0\n' +
			'	mediump float g = 0.0; // (y*3 + z*3) / 6.0\n' +
			'	mediump float b = 0.0; // (z*3 + w*3) / 6.0\n' +
			'	mediump float a = 1.0;\n' +
			'	for (int i = 0; i < ui_two; i++)\n' +
			'	{\n' +
			'		for (int j = 0; j < ui_three; j++)\n' +
			'		{\n' +
			'			r += s[0].b[j].b[i].y;\n' +
			'			g += s[i].b[j].b[0].x;\n' +
			'			b += s[i].b[j].b[1].x;\n' +
			'			a *= s[i].b[j].a;\n' +
			'		}\n' +
			'	}\n' +
			'	${DST} = vec4(r*uf_sixth, g*uf_sixth, b*uf_sixth, a);\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				c.color[0] = (c.coords[0] + c.coords[1]) * 0.5;
				c.color[1] = (c.coords[1] + c.coords[2]) * 0.5;
				c.color[2] = (c.coords[2] + c.coords[3]) * 0.5;
			});

		LocalStructCase('basic_equal', "Basic struct equality",
			'${HEADER}\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump vec3	b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S a = S(floor(${COORDS}.x), vec3(0.0, floor(${COORDS}.y), 2.3), ui_one);\n' +
			'	S b = S(floor(${COORDS}.x+0.5), vec3(0.0, floor(${COORDS}.y), 2.3), ui_one);\n' +
			'	S c = S(floor(${COORDS}.x), vec3(0.0, floor(${COORDS}.y+0.5), 2.3), ui_one);\n' +
			'	S d = S(floor(${COORDS}.x), vec3(0.0, floor(${COORDS}.y), 2.3), ui_two);\n' +
			'	${DST} = vec4(0.0, 0.0, 0.0, 1.0);\n' +
			'	if (a == b) ${DST}.x = 1.0;\n' +
			'	if (a == c) ${DST}.y = 1.0;\n' +
			'	if (a == d) ${DST}.z = 1.0;\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				if (Math.floor(c.coords[0]) === Math.floor(c.coords[0] + 0.5))
					c.color[0] = 1.0;
				if (Math.floor(c.coords[1]) === Math.floor(c.coords[1] + 0.5))
					c.color[1] = 1.0;
			});

		LocalStructCase('basic_not_equal', "Basic struct equality",
			'${HEADER}\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump vec3	b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S a = S(floor(${COORDS}.x), vec3(0.0, floor(${COORDS}.y), 2.3), ui_one);\n' +
			'	S b = S(floor(${COORDS}.x+0.5), vec3(0.0, floor(${COORDS}.y), 2.3), ui_one);\n' +
			'	S c = S(floor(${COORDS}.x), vec3(0.0, floor(${COORDS}.y+0.5), 2.3), ui_one);\n' +
			'	S d = S(floor(${COORDS}.x), vec3(0.0, floor(${COORDS}.y), 2.3), ui_two);\n' +
			'	${DST} = vec4(0.0, 0.0, 0.0, 1.0);\n' +
			'	if (a != b) ${DST}.x = 1.0;\n' +
			'	if (a != c) ${DST}.y = 1.0;\n' +
			'	if (a != d) ${DST}.z = 1.0;\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				if (Math.floor(c.coords[0]) != Math.floor(c.coords[0] + 0.5))
					c.color[0] = 1.0;
				if (Math.floor(c.coords[1]) != Math.floor(c.coords[1] + 0.5))
					c.color[1] = 1.0;
				c.color[2] = 1.0;
			});

		LocalStructCase('nested_equal', "Nested struct struct equality",
			'${HEADER}\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'\n' +
			'struct T {\n' +
			'	mediump vec3	a;\n' +
			'	int				b;\n' +
			'};\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	T				b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S a = S(floor(${COORDS}.x), T(vec3(0.0, floor(${COORDS}.y), 2.3), ui_one), 1);\n' +
			'	S b = S(floor(${COORDS}.x+0.5), T(vec3(0.0, floor(${COORDS}.y), 2.3), ui_one), 1);\n' +
			'	S c = S(floor(${COORDS}.x), T(vec3(0.0, floor(${COORDS}.y+0.5), 2.3), ui_one), 1);\n' +
			'	S d = S(floor(${COORDS}.x), T(vec3(0.0, floor(${COORDS}.y), 2.3), ui_two), 1);\n' +
			'	${DST} = vec4(0.0, 0.0, 0.0, 1.0);\n' +
			'	if (a == b) ${DST}.x = 1.0;\n' +
			'	if (a == c) ${DST}.y = 1.0;\n' +
			'	if (a == d) ${DST}.z = 1.0;\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				if (Math.floor(c.coords[0]) == Math.floor(c.coords[0] + 0.5))
					c.color[0] = 1.0;
				if (Math.floor(c.coords[1]) == Math.floor(c.coords[1] + 0.5))
					c.color[1] = 1.0;
			});

		LocalStructCase('nested_not_equal', "Nested struct struct equality",
			'${HEADER}\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'\n' +
			'struct T {\n' +
			'	mediump vec3	a;\n' +
			'	int				b;\n' +
			'};\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	T				b;\n' +
			'	int				c;\n' +
			'};\n' +
			'\n' +
			'void main (void)\n' +
			'{\n' +
			'	S a = S(floor(${COORDS}.x), T(vec3(0.0, floor(${COORDS}.y), 2.3), ui_one), 1);\n' +
			'	S b = S(floor(${COORDS}.x+0.5), T(vec3(0.0, floor(${COORDS}.y), 2.3), ui_one), 1);\n' +
			'	S c = S(floor(${COORDS}.x), T(vec3(0.0, floor(${COORDS}.y+0.5), 2.3), ui_one), 1);\n' +
			'	S d = S(floor(${COORDS}.x), T(vec3(0.0, floor(${COORDS}.y), 2.3), ui_two), 1);\n' +
			'	${DST} = vec4(0.0, 0.0, 0.0, 1.0);\n' +
			'	if (a != b) ${DST}.x = 1.0;\n' +
			'	if (a != c) ${DST}.y = 1.0;\n' +
			'	if (a != d) ${DST}.z = 1.0;\n' +
			'	${ASSIGN_POS}\n' +
			'}\n',
			function(c) {
				if (Math.floor(c.coords[0]) != Math.floor(c.coords[0] + 0.5))
					c.color[0] = 1.0;
				if (Math.floor(c.coords[1]) != Math.floor(c.coords[1] + 0.5))
					c.color[1] = 1.0;
				c.color[2] = 1.0;
			});
	};

	/**
	 * @constructor
	 * @extends {tcuTestCase.DeqpTest}
	 */
	es3fShaderStructTests.UniformStructTests = function() {
		tcuTestCase.DeqpTest.call(this, 'uniform', 'Uniform structs');
		this.makeExecutable();
	};

	es3fShaderStructTests.UniformStructTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
	es3fShaderStructTests.UniformStructTests.prototype.constructor = es3fShaderStructTests.UniformStructTests;

	/**
	 * @param {WebGLProgram} programID
	 * @param {string} name
	 * @param {Array<number>} vec
	 */
	es3fShaderStructTests.setUniform2fv = function(programID, name, vec) {
		/** @type {WebGLUniformLocation} */ var loc = gl.getUniformLocation(programID, name);
		gl.uniform2fv(loc, vec);
	};

	/**
	 * @param {WebGLProgram} programID
	 * @param {string} name
	 * @param {Array<number>} vec
	 */
	es3fShaderStructTests.setUniform3fv = function(programID, name, vec) {
		/** @type {WebGLUniformLocation} */ var loc = gl.getUniformLocation(programID, name);
		gl.uniform3fv(loc, vec);
	};

	/**
	* @param {WebGLProgram} programID
	* @param {string} name
	* @param {number} value
	*/
	es3fShaderStructTests.setUniform1i = function(programID, name, value) {
		/** @type {WebGLUniformLocation} */ var loc = gl.getUniformLocation(programID, name);
		gl.uniform1i(loc, value);
	};

	/**
	* @param {WebGLProgram} programID
	* @param {string} name
	* @param {number} value
	*/
	es3fShaderStructTests.setUniform1f = function(programID, name, value) {
		/** @type {WebGLUniformLocation} */ var loc = gl.getUniformLocation(programID, name);
		gl.uniform1f(loc, value);
	};

	/**
	* @param {WebGLProgram} programID
	* @param {string} name
	* @param {Array<number>} vec
	*/
	es3fShaderStructTests.setUniform1fv = function(programID, name, vec) {
		/** @type {WebGLUniformLocation} */ var loc = gl.getUniformLocation(programID, name);
		gl.uniform1fv(loc, vec);
	};

	es3fShaderStructTests.UniformStructTests.prototype.init = function() {
		var currentCtx = this;
		function UniformStructCase(name, description, textures, shaderSrc, setUniformsFunc, evalFunc) {
			currentCtx.addChild(es3fShaderStructTests.ShaderStructCase.createStructCase(name + "_vertex", description, true, textures, evalFunc, setUniformsFunc, shaderSrc));
			currentCtx.addChild(es3fShaderStructTests.ShaderStructCase.createStructCase(name + "_fragment", description, false, textures, evalFunc, setUniformsFunc, shaderSrc));
		}

		UniformStructCase('basic', "Basic struct usage", false,
			'${HEADER}\n' +
			'uniform int ui_one;\n' +
			'' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump vec3	b;\n' +
			'	int				c;\n' +
			'};\n' +
			'uniform S s;\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'	${DST} = vec4(s.a, s.b.x, s.b.y, s.c);\n' +
			'	${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				es3fShaderStructTests.setUniform1f(programID, "s.a", constCoords[0]);
				es3fShaderStructTests.setUniform3fv(programID, "s.b", deMath.swizzle(constCoords, [1, 2, 3]));
				es3fShaderStructTests.setUniform1i(programID, "s.c", 1);
			},
			function(c) {
				c.color[0] = c.constCoords[0];
				c.color[1] = c.constCoords[1];
				c.color[2] = c.constCoords[2];
			});

		UniformStructCase('nested', "Nested struct", false,
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'' +
			'struct T {\n' +
			'	int				a;\n' +
			'	mediump vec2	b;\n' +
			'};\n' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	T				b;\n' +
			'	int				c;\n' +
			'};\n' +
			'uniform S s;\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'	${DST} = vec4(s.a, s.b.b, s.b.a + s.c);\n' +
			'	${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				es3fShaderStructTests.setUniform1f(programID, "s.a", constCoords[0]);
				es3fShaderStructTests.setUniform1i(programID, "s.b.a", 0);
				es3fShaderStructTests.setUniform2fv(programID, "s.b.b", deMath.swizzle(constCoords, [1,2]));
				es3fShaderStructTests.setUniform1i(programID, "s.c", 1);
			},
			function(c) {
				c.color[0] = c.constCoords[0];
				c.color[1] = c.constCoords[1];
				c.color[2] = c.constCoords[2];
			});

		UniformStructCase('array_member', "Struct with array member", false,
			'${HEADER}\n' +
			'uniform int ui_one;\n' +
			'' +
			'struct S {\n' +
			'	mediump float	a;\n' +
			'	mediump float	b[3];\n' +
			'	int				c;\n' +
			'};\n' +
			'uniform S s;\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'	${DST} = vec4(s.a, s.b[0], s.b[1], s.c);\n' +
			'	${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords){
				es3fShaderStructTests.setUniform1f(programID, "s.a", constCoords[3]);
				es3fShaderStructTests.setUniform1i(programID, "s.c", 1);

				/** @type {Array<number>} */ var b = [];
				b[0] = constCoords[2];
				b[1] = constCoords[1];
				b[2] = constCoords[0];
				es3fShaderStructTests.setUniform1fv(programID, "s.b", b);
			},
			function(c) {
				c.color[0] = c.constCoords[3];
				c.color[1] = c.constCoords[2];
				c.color[2] = c.constCoords[1];
			});

		UniformStructCase('array_member_dynamic_index', "Struct with array member, dynamic indexing", false,
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'' +
			'struct S {\n' +
			'    mediump float    a;\n' +
			'    mediump float    b[3];\n' +
			'    int                c;\n' +
			'};\n' +
			'uniform S s;\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'    ${DST} = vec4(s.b[ui_one], s.b[ui_zero], s.b[ui_two], s.c);\n' +
			'    ${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				es3fShaderStructTests.setUniform1f(programID, "s.a", constCoords[3]);
				es3fShaderStructTests.setUniform1i(programID, "s.c", 1);

				/** @type {Array<number>} */ var b = [];
				b[0] = constCoords[2];
				b[1] = constCoords[1];
				b[2] = constCoords[0];
				es3fShaderStructTests.setUniform1fv(programID, "s.b", b);
			},
			function(c) {
				c.color[0] = c.constCoords[1];
				c.color[1] = c.constCoords[2];
				c.color[2] = c.constCoords[0];
			});

		UniformStructCase('struct_array', "Struct array", false,
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'' +
			'struct S {\n' +
			'    mediump float    a;\n' +
			'    mediump int        b;\n' +
			'};\n' +
			'uniform S s[3];\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'    ${DST} = vec4(s[2].a, s[1].a, s[0].a, s[2].b - s[1].b + s[0].b);\n' +
			'    ${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				es3fShaderStructTests.setUniform1f(programID, "s[0].a", constCoords[0]);
				es3fShaderStructTests.setUniform1i(programID, "s[0].b", 0);
				es3fShaderStructTests.setUniform1f(programID, "s[1].a", constCoords[1]);
				es3fShaderStructTests.setUniform1i(programID, "s[1].b", 1);
				es3fShaderStructTests.setUniform1f(programID, "s[2].a", constCoords[2]);
				es3fShaderStructTests.setUniform1i(programID, "s[2].b", 2);
			},
			function(c) {
				c.color[0] = c.constCoords[2];
				c.color[1] = c.constCoords[1];
				c.color[2] = c.constCoords[0];
			});

		UniformStructCase('struct_array_dynamic_index', "Struct array with dynamic indexing", false,
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'' +
			'struct S {\n' +
			'    mediump float    a;\n' +
			'    mediump int        b;\n' +
			'};\n' +
			'uniform S s[3];\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'    ${DST} = vec4(s[ui_two].a, s[ui_one].a, s[ui_zero].a, s[ui_two].b - s[ui_one].b + s[ui_zero].b);\n' +
			'    ${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				es3fShaderStructTests.setUniform1f(programID, "s[0].a", constCoords[0]);
				es3fShaderStructTests.setUniform1i(programID, "s[0].b", 0);
				es3fShaderStructTests.setUniform1f(programID, "s[1].a", constCoords[1]);
				es3fShaderStructTests.setUniform1i(programID, "s[1].b", 1);
				es3fShaderStructTests.setUniform1f(programID, "s[2].a", constCoords[2]);
				es3fShaderStructTests.setUniform1i(programID, "s[2].b", 2);
			},
			function(c) {
				c.color[0] = c.constCoords[2];
				c.color[1] = c.constCoords[1];
				c.color[2] = c.constCoords[0];
			});

		UniformStructCase('nested_struct_array', "Nested struct array", false,
			'${HEADER}\n' +
			'struct T {\n' +
			'    mediump float    a;\n' +
			'    mediump vec2    b[2];\n' +
			'};\n' +
			'struct S {\n' +
			'    mediump float    a;\n' +
			'    T                b[3];\n' +
			'    int                c;\n' +
			'};\n' +
			'uniform S s[2];\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'    mediump float r = (s[0].b[1].b[0].x + s[1].b[2].b[1].y) * s[0].b[0].a; // (z + z) * 0.5\n' +
			'    mediump float g = s[1].b[0].b[0].y * s[0].b[2].a * s[1].b[2].a; // x * 0.25 * 4\n' +
			'    mediump float b = (s[0].b[2].b[1].y + s[0].b[1].b[0].y + s[1].a) * s[0].b[1].a; // (w + w + w) * 0.333\n' +
			'    mediump float a = float(s[0].c) + s[1].b[2].a - s[1].b[1].a; // 0 + 4.0 - 3.0\n' +
			'    ${DST} = vec4(r, g, b, a);\n' +
			'    ${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				/** @type {Array<number>} */ var arr = [];

				es3fShaderStructTests.setUniform1f(programID, "s[0].a", constCoords[0]);
				arr = deMath.swizzle(constCoords, [0,1,2,3]);
				es3fShaderStructTests.setUniform1f(programID, "s[0].b[0].a", 0.5);
				es3fShaderStructTests.setUniform2fv(programID, "s[0].b[0].b", arr);
				arr = deMath.swizzle(constCoords, [2,3,0,1]);
				es3fShaderStructTests.setUniform1f(programID, "s[0].b[1].a", 1.0/3.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[0].b[1].b", arr);
				arr = deMath.swizzle(constCoords, [0,2,1,3]);
				es3fShaderStructTests.setUniform1f(programID, "s[0].b[2].a", 1.0/4.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[0].b[2].b", arr);
				es3fShaderStructTests.setUniform1i(programID, "s[0].c", 0);

				es3fShaderStructTests.setUniform1f(programID, "s[1].a", constCoords[3]);
				arr = deMath.swizzle(constCoords, [0,0,1,1]);
				es3fShaderStructTests.setUniform1f(programID, "s[1].b[0].a", 2.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[1].b[0].b", arr);
				arr = deMath.swizzle(constCoords, [2,2,3,3]);
				es3fShaderStructTests.setUniform1f(programID, "s[1].b[1].a", 3.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[1].b[1].b", arr);
				arr = deMath.swizzle(constCoords, [1,0,3,2]);
				es3fShaderStructTests.setUniform1f(programID, "s[1].b[2].a", 4.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[1].b[2].b", arr);
				es3fShaderStructTests.setUniform1i(programID, "s[1].c", 1);
			},
			function(c) {
				c.color[0] = c.constCoords[2];
				c.color[1] = c.constCoords[0];
				c.color[2] = c.constCoords[3];
			});

		UniformStructCase('nested_struct_array_dynamic_index', "Nested struct array with dynamic indexing", false,
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'' +
			'struct T {\n' +
			'    mediump float    a;\n' +
			'    mediump vec2    b[2];\n' +
			'};\n' +
			'struct S {\n' +
			'    mediump float    a;\n' +
			'    T                b[3];\n' +
			'    int                c;\n' +
			'};\n' +
			'uniform S s[2];\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'    mediump float r = (s[0].b[ui_one].b[ui_one-1].x + s[ui_one].b[ui_two].b[ui_zero+1].y) * s[0].b[0].a; // (z + z) * 0.5\n' +
			'    mediump float g = s[ui_two-1].b[ui_two-2].b[ui_zero].y * s[0].b[ui_two].a * s[ui_one].b[2].a; // x * 0.25 * 4\n' +
			'    mediump float b = (s[ui_zero].b[ui_one+1].b[1].y + s[0].b[ui_one*ui_one].b[0].y + s[ui_one].a) * s[0].b[ui_two-ui_one].a; // (w + w + w) * 0.333\n' +
			'    mediump float a = float(s[ui_zero].c) + s[ui_one-ui_zero].b[ui_two].a - s[ui_zero+ui_one].b[ui_two-ui_one].a; // 0 + 4.0 - 3.0\n' +
			'    ${DST} = vec4(r, g, b, a);\n' +
			'    ${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords){
				/** @type {Array<number>} */ var arr = [];

				es3fShaderStructTests.setUniform1f(programID, "s[0].a", constCoords[0]);
				arr = constCoords;
				es3fShaderStructTests.setUniform1f(programID, "s[0].b[0].a", 0.5);
				es3fShaderStructTests.setUniform2fv(programID, "s[0].b[0].b", arr);
				arr = deMath.swizzle(constCoords, [2,3,0,1]);
				es3fShaderStructTests.setUniform1f(programID, "s[0].b[1].a", 1.0/3.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[0].b[1].b", arr);
				arr = deMath.swizzle(constCoords, [0,2,1,3]);
				es3fShaderStructTests.setUniform1f(programID, "s[0].b[2].a", 1.0/4.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[0].b[2].b", arr);
				es3fShaderStructTests.setUniform1i(programID, "s[0].c", 0);

				es3fShaderStructTests.setUniform1f(programID, "s[1].a", constCoords[3]);
				arr = deMath.swizzle(constCoords, [0,0,1,1]);
				es3fShaderStructTests.setUniform1f(programID, "s[1].b[0].a", 2.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[1].b[0].b", arr);
				arr = deMath.swizzle(constCoords, [2,2,3,3]);
				es3fShaderStructTests.setUniform1f(programID, "s[1].b[1].a", 3.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[1].b[1].b", arr);
				arr = deMath.swizzle(constCoords, [1,0,3,2]);
				es3fShaderStructTests.setUniform1f(programID, "s[1].b[2].a", 4.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[1].b[2].b", arr);
				es3fShaderStructTests.setUniform1i(programID, "s[1].c", 1);
			},
			function(c) {
				c.color[0] = c.constCoords[2];
				c.color[1] = c.constCoords[0];
				c.color[2] = c.constCoords[3];
			});

		UniformStructCase('loop_struct_array', "Struct array usage in loop", false,
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'' +
			'struct S {\n' +
			'    mediump float    a;\n' +
			'    mediump int        b;\n' +
			'};\n' +
			'uniform S s[3];\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'    mediump float rgb[3];\n' +
			'    int alpha = 0;\n' +
			'    for (int i = 0; i < 3; i++)\n' +
			'    {\n' +
			'        rgb[i] = s[2-i].a;\n' +
			'        alpha += s[i].b;\n' +
			'    }\n' +
			'    ${DST} = vec4(rgb[0], rgb[1], rgb[2], alpha);\n' +
			'    ${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				es3fShaderStructTests.setUniform1f(programID, "s[0].a", constCoords[0]);
				es3fShaderStructTests.setUniform1i(programID, "s[0].b", 0);
				es3fShaderStructTests.setUniform1f(programID, "s[1].a", constCoords[1]);
				es3fShaderStructTests.setUniform1i(programID, "s[1].b", -1);
				es3fShaderStructTests.setUniform1f(programID, "s[2].a", constCoords[2]);
				es3fShaderStructTests.setUniform1i(programID, "s[2].b", 2);
			},
			function(c) {
				c.color[0] = c.constCoords[2];
				c.color[1] = c.constCoords[1];
				c.color[2] = c.constCoords[0];
			});

		UniformStructCase('loop_nested_struct_array', "Nested struct array usage in loop", false,
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'uniform mediump float uf_two;\n' +
			'uniform mediump float uf_three;\n' +
			'uniform mediump float uf_four;\n' +
			'uniform mediump float uf_half;\n' +
			'uniform mediump float uf_third;\n' +
			'uniform mediump float uf_fourth;\n' +
			'uniform mediump float uf_sixth;\n' +
			'' +
			'struct T {\n' +
			'    mediump float    a;\n' +
			'    mediump vec2    b[2];\n' +
			'};\n' +
			'struct S {\n' +
			'    mediump float    a;\n' +
			'    T                b[3];\n' +
			'    int                c;\n' +
			'};\n' +
			'uniform S s[2];\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'    mediump float r = 0.0; // (x*3 + y*3) / 6.0\n' +
			'    mediump float g = 0.0; // (y*3 + z*3) / 6.0\n' +
			'    mediump float b = 0.0; // (z*3 + w*3) / 6.0\n' +
			'    mediump float a = 1.0;\n' +
			'    for (int i = 0; i < 2; i++)\n' +
			'    {\n' +
			'        for (int j = 0; j < 3; j++)\n' +
			'        {\n' +
			'            r += s[0].b[j].b[i].y;\n' +
			'            g += s[i].b[j].b[0].x;\n' +
			'            b += s[i].b[j].b[1].x;\n' +
			'            a *= s[i].b[j].a;\n' +
			'        }\n' +
			'    }\n' +
			'    ${DST} = vec4(r*uf_sixth, g*uf_sixth, b*uf_sixth, a);\n' +
			'    ${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				/** @type {Array<number>} */ var arr = [];

				es3fShaderStructTests.setUniform1f(programID, "s[0].a", constCoords[0]);
				arr = deMath.swizzle(constCoords, [1,0,2,0]);
				es3fShaderStructTests.setUniform1f(programID, "s[0].b[0].a", 0.5);
				es3fShaderStructTests.setUniform2fv(programID, "s[0].b[0].b", arr);
				arr = deMath.swizzle(constCoords, [1,1,3,1]);
				es3fShaderStructTests.setUniform1f(programID, "s[0].b[1].a", 1.0/3.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[0].b[1].b", arr);
				arr = deMath.swizzle(constCoords, [2,1,2,1]);
				es3fShaderStructTests.setUniform1f(programID, "s[0].b[2].a", 1.0/4.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[0].b[2].b", arr);
				es3fShaderStructTests.setUniform1i(programID, "s[0].c", 0);

				es3fShaderStructTests.setUniform1f(programID, "s[1].a", constCoords[3]);
				arr = deMath.swizzle(constCoords, [2,0,2,1]);
				es3fShaderStructTests.setUniform1f(programID, "s[1].b[0].a", 2.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[1].b[0].b", arr);
				arr = deMath.swizzle(constCoords, [2,2,3,3]);
				es3fShaderStructTests.setUniform1f(programID, "s[1].b[1].a", 3.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[1].b[1].b", arr);
				arr = deMath.swizzle(constCoords, [1,0,3,2]);
				es3fShaderStructTests.setUniform1f(programID, "s[1].b[2].a", 4.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[1].b[2].b", arr);
				es3fShaderStructTests.setUniform1i(programID, "s[1].c", 1);
			},
			function(c) {
				c.color[0] = (c.constCoords[0] + c.constCoords[1]) * 0.5;
				c.color[1] = (c.constCoords[1] + c.constCoords[2]) * 0.5;
				c.color[2] = (c.constCoords[2] + c.constCoords[3]) * 0.5;
			});

		UniformStructCase('dynamic_loop_struct_array', "Struct array usage in dynamic loop", false,
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'uniform int ui_three;\n' +
			'' +
			'struct S {\n' +
			'    mediump float    a;\n' +
			'    mediump int        b;\n' +
			'};\n' +
			'uniform S s[3];\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'    mediump float rgb[3];\n' +
			'    int alpha = 0;\n' +
			'    for (int i = 0; i < ui_three; i++)\n' +
			'    {\n' +
			'        rgb[i] = s[2-i].a;\n' +
			'        alpha += s[i].b;\n' +
			'    }\n' +
			'    ${DST} = vec4(rgb[0], rgb[1], rgb[2], alpha);\n' +
			'    ${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				es3fShaderStructTests.setUniform1f(programID, "s[0].a", constCoords[0]);
				es3fShaderStructTests.setUniform1i(programID, "s[0].b", 0);
				es3fShaderStructTests.setUniform1f(programID, "s[1].a", constCoords[1]);
				es3fShaderStructTests.setUniform1i(programID, "s[1].b", -1);
				es3fShaderStructTests.setUniform1f(programID, "s[2].a", constCoords[2]);
				es3fShaderStructTests.setUniform1i(programID, "s[2].b", 2);
			},
			function(c) {
				c.color[0] = c.constCoords[2];
				c.color[1] = c.constCoords[1];
				c.color[2] = c.constCoords[0];
			});

		UniformStructCase('dynamic_loop_nested_struct_array', "Nested struct array usage in dynamic loop", false,
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'uniform int ui_two;\n' +
			'uniform int ui_three;\n' +
			'uniform mediump float uf_two;\n' +
			'uniform mediump float uf_three;\n' +
			'uniform mediump float uf_four;\n' +
			'uniform mediump float uf_half;\n' +
			'uniform mediump float uf_third;\n' +
			'uniform mediump float uf_fourth;\n' +
			'uniform mediump float uf_sixth;\n' +
			'' +
			'struct T {\n' +
			'    mediump float    a;\n' +
			'    mediump vec2    b[2];\n' +
			'};\n' +
			'struct S {\n' +
			'    mediump float    a;\n' +
			'    T                b[3];\n' +
			'    int                c;\n' +
			'};\n' +
			'uniform S s[2];\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'    mediump float r = 0.0; // (x*3 + y*3) / 6.0\n' +
			'    mediump float g = 0.0; // (y*3 + z*3) / 6.0\n' +
			'    mediump float b = 0.0; // (z*3 + w*3) / 6.0\n' +
			'    mediump float a = 1.0;\n' +
			'    for (int i = 0; i < ui_two; i++)\n' +
			'    {\n' +
			'        for (int j = 0; j < ui_three; j++)\n' +
			'        {\n' +
			'            r += s[0].b[j].b[i].y;\n' +
			'            g += s[i].b[j].b[0].x;\n' +
			'            b += s[i].b[j].b[1].x;\n' +
			'            a *= s[i].b[j].a;\n' +
			'        }\n' +
			'    }\n' +
			'    ${DST} = vec4(r*uf_sixth, g*uf_sixth, b*uf_sixth, a);\n' +
			'    ${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				/** @type {Array<number>} */ var arr = [];

				es3fShaderStructTests.setUniform1f(programID, "s[0].a", constCoords[0]);
				arr = deMath.swizzle(constCoords, [1,0,2,0]);
				es3fShaderStructTests.setUniform1f(programID, "s[0].b[0].a", 0.5);
				es3fShaderStructTests.setUniform2fv(programID, "s[0].b[0].b", arr);
				arr = deMath.swizzle(constCoords, [1,1,3,1]);
				es3fShaderStructTests.setUniform1f(programID, "s[0].b[1].a", 1.0/3.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[0].b[1].b", arr);
				arr = deMath.swizzle(constCoords, [2,1,2,1]);
				es3fShaderStructTests.setUniform1f(programID, "s[0].b[2].a",    1.0/4.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[0].b[2].b", arr);
				es3fShaderStructTests.setUniform1i(programID, "s[0].c", 0);

				es3fShaderStructTests.setUniform1f(programID, "s[1].a", constCoords[3]);
				arr = deMath.swizzle(constCoords, [2,0,2,1]);
				es3fShaderStructTests.setUniform1f(programID, "s[1].b[0].a", 2.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[1].b[0].b", arr);
				arr = deMath.swizzle(constCoords, [2,2,3,3]);
				es3fShaderStructTests.setUniform1f(programID, "s[1].b[1].a", 3.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[1].b[1].b", arr);
				arr = deMath.swizzle(constCoords, [1,0,3,2]);
				es3fShaderStructTests.setUniform1f(programID, "s[1].b[2].a", 4.0);
				es3fShaderStructTests.setUniform2fv(programID, "s[1].b[2].b", arr);
				es3fShaderStructTests.setUniform1i(programID, "s[1].c", 1);
			},
			function(c) {
				c.color[0] = (c.constCoords[0] + c.constCoords[1]) * 0.5;
				c.color[1] = (c.constCoords[1] + c.constCoords[2]) * 0.5;
				c.color[2] = (c.constCoords[2] + c.constCoords[3]) * 0.5;
			});

		UniformStructCase('sampler', "Sampler in struct", true,
			'${HEADER}\n' +
			'uniform int ui_one;\n' +
			'' +
			'struct S {\n' +
			'    mediump float    a;\n' +
			'    mediump vec3    b;\n' +
			'    sampler2D        c;\n' +
			'};\n' +
			'uniform S s;\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'    ${DST} = vec4(texture(s.c, ${COORDS}.xy * s.b.xy + s.b.z).rgb, s.a);\n' +
			'    ${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				es3fShaderStructTests.setUniform1f(programID, "s.a", 1.0);
				es3fShaderStructTests.setUniform3fv(programID, "s.b", [0.25, 0.25, 0.5]);
				es3fShaderStructTests.setUniform1i(programID, "s.c", 0);
			},
			function(c) {
				var tex2d = c.texture2D(es3fShaderStructTests.TEXTURE_BRICK, deMath.addScalar(deMath.scale(deMath.swizzle(c.coords, [0,1]), 0.25), 0.5))

				c.color[0] = tex2d[0];
				c.color[1] = tex2d[1];
				c.color[2] = tex2d[2];
			});

		UniformStructCase('sampler_nested', "Sampler in nested struct", true,
			'${HEADER}\n' +
			'uniform int ui_zero;\n' +
			'uniform int ui_one;\n' +
			'' +
			'struct T {\n' +
			'    sampler2D        a;\n' +
			'    mediump vec2    b;\n' +
			'};\n' +
			'struct S {\n' +
			'    mediump float    a;\n' +
			'    T                b;\n' +
			'    int                c;\n' +
			'};\n' +
			'uniform S s;\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'    ${DST} = vec4(texture(s.b.a, ${COORDS}.xy * s.b.b + s.a).rgb, s.c);\n' +
			'    ${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				es3fShaderStructTests.setUniform1f(programID, "s.a", 0.5);
				es3fShaderStructTests.setUniform1i(programID, "s.b.a", 0);
				es3fShaderStructTests.setUniform2fv(programID, "s.b.b", [0.25, 0.25]);
				es3fShaderStructTests.setUniform1i(programID, "s.c", 1);
			},
			function(c) {
				var tex2d = c.texture2D(es3fShaderStructTests.TEXTURE_BRICK, deMath.addScalar(deMath.scale(deMath.swizzle(c.coords, [0,1]), 0.25), 0.5));
				c.color[0] = tex2d[0];
				c.color[1] = tex2d[1];
				c.color[2] = tex2d[2];
			});

		UniformStructCase('sampler_array', "Sampler in struct array", true,
			'${HEADER}\n' +
			'uniform int ui_one;\n' +
			'' +
			'struct S {\n' +
			'    mediump float    a;\n' +
			'    mediump vec3    b;\n' +
			'    sampler2D        c;\n' +
			'};\n' +
			'uniform S s[2];\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'    ${DST} = vec4(texture(s[1].c, ${COORDS}.xy * s[0].b.xy + s[1].b.z).rgb, s[0].a);\n' +
			'    ${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				es3fShaderStructTests.setUniform1f(programID, "s[0].a", 1.0);
				es3fShaderStructTests.setUniform3fv(programID, "s[0].b", [0.25, 0.25, 0.25]);
				es3fShaderStructTests.setUniform1i(programID, "s[0].c", 1);
				es3fShaderStructTests.setUniform1f(programID, "s[1].a", 0.0);
				es3fShaderStructTests.setUniform3fv(programID, "s[1].b", [0.5, 0.5, 0.5]);
				es3fShaderStructTests.setUniform1i(programID, "s[1].c", 0);
			},
			function(c) {
				var tex2d = c.texture2D(es3fShaderStructTests.TEXTURE_BRICK, deMath.addScalar(deMath.scale(deMath.swizzle(c.coords, [0,1]), 0.25), 0.5));
				c.color[0] = tex2d[0];
				c.color[1] = tex2d[1];
				c.color[2] = tex2d[2];
			});

		UniformStructCase('equal', "Struct equality", false,
			'${HEADER}\n' +
			'uniform mediump float uf_one;\n' +
			'uniform int ui_two;\n' +
			'' +
			'struct S {\n' +
			'    mediump float    a;\n' +
			'    mediump vec3    b;\n' +
			'    int                c;\n' +
			'};\n' +
			'uniform S a;\n' +
			'uniform S b;\n' +
			'uniform S c;\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'    S d = S(uf_one, vec3(0.0, floor(${COORDS}.y+1.0), 2.0), ui_two);\n' +
			'    ${DST} = vec4(0.0, 0.0, 0.0, 1.0);\n' +
			'    if (a == b) ${DST}.x = 1.0;\n' +
			'    if (a == c) ${DST}.y = 1.0;\n' +
			'    if (a == d) ${DST}.z = 1.0;\n' +
			'    ${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				es3fShaderStructTests.setUniform1f(programID, "a.a", 1.0);
				es3fShaderStructTests.setUniform3fv(programID, "a.b", [0.0, 1.0, 2.0]);
				es3fShaderStructTests.setUniform1i(programID, "a.c", 2);
				es3fShaderStructTests.setUniform1f(programID, "b.a", 1.0);
				es3fShaderStructTests.setUniform3fv(programID, "b.b", [0.0, 1.0, 2.0]);
				es3fShaderStructTests.setUniform1i(programID, "b.c", 2);
				es3fShaderStructTests.setUniform1f(programID, "c.a", 1.0);
				es3fShaderStructTests.setUniform3fv(programID, "c.b", [0.0, 1.1, 2.0]);
				es3fShaderStructTests.setUniform1i(programID, "c.c", 2);
			},
			function(c) {
				c.color[0] = 1.0;
				c.color[1] = 0.0;
				if (Math.floor(c.coords[1] + 1.0) == Math.floor(1.1))
					c.color[2] = 1.0;
			});

		UniformStructCase('not_equal', "Struct equality", false,
			'${HEADER}\n' +
			'uniform mediump float uf_one;\n' +
			'uniform int ui_two;\n' +
			'' +
			'struct S {\n' +
			'    mediump float    a;\n' +
			'    mediump vec3    b;\n' +
			'    int                c;\n' +
			'};\n' +
			'uniform S a;\n' +
			'uniform S b;\n' +
			'uniform S c;\n' +
			'' +
			'void main (void)\n' +
			'{\n' +
			'    S d = S(uf_one, vec3(0.0, floor(${COORDS}.y+1.0), 2.0), ui_two);\n' +
			'    ${DST} = vec4(0.0, 0.0, 0.0, 1.0);\n' +
			'    if (a != b) ${DST}.x = 1.0;\n' +
			'    if (a != c) ${DST}.y = 1.0;\n' +
			'    if (a != d) ${DST}.z = 1.0;\n' +
			'    ${ASSIGN_POS}\n' +
			'}',
			function(programID, constCoords) {
				es3fShaderStructTests.setUniform1f(programID, "a.a", 1.0);
				es3fShaderStructTests.setUniform3fv(programID, "a.b", [0.0, 1.0, 2.0]);
				es3fShaderStructTests.setUniform1i(programID, "a.c", 2);
				es3fShaderStructTests.setUniform1f(programID, "b.a", 1.0);
				es3fShaderStructTests.setUniform3fv(programID, "b.b", [0.0, 1.0, 2.0]);
				es3fShaderStructTests.setUniform1i(programID, "b.c", 2);
				es3fShaderStructTests.setUniform1f(programID, "c.a", 1.0);
				es3fShaderStructTests.setUniform3fv(programID, "c.b", [0.0, 1.1, 2.0]);
				es3fShaderStructTests.setUniform1i(programID, "c.c", 2);
			},
			function(c) {
				c.color[0] = 0.0;
				c.color[1] = 1.0;
				if (Math.floor(c.coords[1] + 1.0) != Math.floor(1.1))
					c.color[2] = 1.0;
			});

	};

	/**
	 * @constructor
	 * @extends {tcuTestCase.DeqpTest}
	 */
	es3fShaderStructTests.ShaderStructTests = function() {
		tcuTestCase.DeqpTest.call(this, 'struct', 'Struct Tests');
	};

	es3fShaderStructTests.ShaderStructTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
	es3fShaderStructTests.ShaderStructTests.prototype.constructor = es3fShaderStructTests.ShaderStructTests;

	es3fShaderStructTests.ShaderStructTests.prototype.init = function() {
		this.addChild(new es3fShaderStructTests.LocalStructTests());
		this.addChild(new es3fShaderStructTests.UniformStructTests());
	};

	/**
     * Run test
     * @param {WebGL2RenderingContext} context
     */
    es3fShaderStructTests.run = function(context) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;
        state.setRoot(new es3fShaderStructTests.ShaderStructTests());

        //Set up name and description of this test series.
        setCurrentTestName(state.testCases.fullName());
        description(state.testCases.getDescription());
        try {
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fShaderStructTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };


});
