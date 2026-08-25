/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
 *
 * Copyright 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

'use strict';
goog.provide('functional.gles3.es3fShaderSwitchTests');
goog.require('framework.common.tcuStringTemplate');
goog.require('framework.common.tcuTestCase');
goog.require('modules.shared.glsShaderRenderCase');


goog.scope(function() {
    var es3fShaderSwitchTests = functional.gles3.es3fShaderSwitchTests;
    var glsShaderRenderCase = modules.shared.glsShaderRenderCase;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuStringTemplate = framework.common.tcuStringTemplate;

    /**
     * @constructor
     * @extends {glsShaderRenderCase.ShaderRenderCase}
     * @param {string} name
     * @param {string} description
     * @param {boolean} isVertexCase
     * @param {string} vtxSource
     * @param {string} fragSource
     * @param {glsShaderRenderCase.ShaderEvalFunc=} evalFunc
     */
    es3fShaderSwitchTests.ShaderSwitchCase = function(name, description, isVertexCase, vtxSource, fragSource, evalFunc) {
        glsShaderRenderCase.ShaderRenderCase.call(this, name, description, isVertexCase, evalFunc);
        /** @type {string} */ this.m_vertShaderSource = vtxSource;
        /** @type {string} */ this.m_fragShaderSource = fragSource;
    };

    es3fShaderSwitchTests.ShaderSwitchCase.prototype = Object.create(glsShaderRenderCase.ShaderRenderCase.prototype);
    es3fShaderSwitchTests.ShaderSwitchCase.prototype.constructor = es3fShaderSwitchTests.ShaderSwitchCase;

    /**
     * @enum {number}
     */
    es3fShaderSwitchTests.SwitchType = {
        STATIC: 0,
        UNIFORM: 1,
        DYNAMIC: 2
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} evalCtx */
    es3fShaderSwitchTests.evalSwitchStatic = function(evalCtx) {
        evalCtx.color[0] = evalCtx.coords[1];
        evalCtx.color[1] = evalCtx.coords[2];
        evalCtx.color[2] = evalCtx.coords[3];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} evalCtx */
    es3fShaderSwitchTests.evalSwitchUniform = function(evalCtx) {
        evalCtx.color[0] = evalCtx.coords[1];
        evalCtx.color[1] = evalCtx.coords[2];
        evalCtx.color[2] = evalCtx.coords[3];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} evalCtx */
    es3fShaderSwitchTests.evalSwitchDynamic = function(evalCtx) {
        switch (Math.floor(evalCtx.coords[2]*1.5 + 2.0)) {
            case 0:
                evalCtx.color[0] = evalCtx.coords[0];
                evalCtx.color[1] = evalCtx.coords[1];
                evalCtx.color[2] = evalCtx.coords[2];
                break;
            case 1:
                evalCtx.color[0] = evalCtx.coords[3];
                evalCtx.color[1] = evalCtx.coords[2];
                evalCtx.color[2] = evalCtx.coords[1];
                break;
            case 2:
                evalCtx.color[0] = evalCtx.coords[1];
                evalCtx.color[1] = evalCtx.coords[2];
                evalCtx.color[2] = evalCtx.coords[3];
                break;
            case 3:
                evalCtx.color[0] = evalCtx.coords[2];
                evalCtx.color[1] = evalCtx.coords[1];
                evalCtx.color[2] = evalCtx.coords[0];
                break;
            default:
                evalCtx.color[0] = evalCtx.coords[0];
                evalCtx.color[1] = evalCtx.coords[0];
                evalCtx.color[2] = evalCtx.coords[0];
                break;
        }
    };

    /**
     * @param  {string} name
     * @param  {string} desc
     * @param  {es3fShaderSwitchTests.SwitchType} type
     * @param  {boolean} isVertex
     * @param  {string} switchBody
     * @return {es3fShaderSwitchTests.ShaderSwitchCase}
     */
    es3fShaderSwitchTests.makeSwitchCase = function(name, desc, type, isVertex, switchBody) {
        /** @type {string} */ var vtx = '';
        /** @type {string} */ var frag = '';
        /** @type {string} */ var op = '';

        vtx += "#version 300 es\n" +
               "in highp vec4 a_position;\n" +
               "in highp vec4 a_coords;\n";
        frag += "#version 300 es\n" +
                "layout(location = 0) out mediump vec4 o_color;\n";

        if (isVertex) {
            vtx += "out mediump vec4 v_color;\n";
            frag += "in mediump vec4 v_color;\n";
        } else {
            vtx += "out highp vec4 v_coords;\n";
            frag += "in highp vec4 v_coords;\n";
        }

        if (type === es3fShaderSwitchTests.SwitchType.UNIFORM)
            op += "uniform highp int ui_two;\n";

        vtx += isVertex ? op : '';
        frag += isVertex ? '' : op;
        op = '';

        vtx += "\n" +
            "void main (void)\n" +
            "{\n" +
            "    gl_Position = a_position;\n";
        frag += "\n" +
                "void main (void)\n" +
                "{\n";

        // Setup.
        op += "    highp vec4 coords = " + (isVertex ? "a_coords" : "v_coords") + ";\n";
        op += "    mediump vec3 res = vec3(0.0);\n\n";
        vtx += isVertex ? op : '';
        frag += isVertex ? '' : op;
        op = '';

        // Switch body.
        var params = {};
        params["CONDITION"] = type == es3fShaderSwitchTests.SwitchType.STATIC ? "2"     :
                              type == es3fShaderSwitchTests.SwitchType.UNIFORM ? "ui_two" :
                              type == es3fShaderSwitchTests.SwitchType.DYNAMIC ? "int(floor(coords.z*1.5 + 2.0))" : "???";

        op += tcuStringTemplate.specialize(switchBody, params);
        op += "\n";

        vtx += isVertex ? op : '';
        frag += isVertex ? '' : op;
        op = '';

        if (isVertex) {
            vtx += "    v_color = vec4(res, 1.0);\n";
            frag += "    o_color = v_color;\n";
        } else {
            vtx += "    v_coords = a_coords;\n";
            frag += "    o_color = vec4(res, 1.0);\n";
        }

        vtx += "}\n";
        frag += "}\n";

        return new es3fShaderSwitchTests.ShaderSwitchCase(name, desc, isVertex, vtx, frag,
                                    type === es3fShaderSwitchTests.SwitchType.STATIC ? es3fShaderSwitchTests.evalSwitchStatic :
                                    type === es3fShaderSwitchTests.SwitchType.UNIFORM ? es3fShaderSwitchTests.evalSwitchUniform :
                                    type === es3fShaderSwitchTests.SwitchType.DYNAMIC ? es3fShaderSwitchTests.evalSwitchDynamic : undefined);
    };

    /**
     * @param {tcuTestCase.DeqpTest} group
     * @param {string} name
     * @param {string} desc
     * @param {string} switchBody
     */
    es3fShaderSwitchTests.makeSwitchCases = function(group, name, desc, switchBody) {
        /** @type {Array<string>} */ var switchTypeNames = ["static", "uniform", "dynamic"];
        for (var type in es3fShaderSwitchTests.SwitchType) {
            group.addChild(es3fShaderSwitchTests.makeSwitchCase(name + "_" + switchTypeNames[es3fShaderSwitchTests.SwitchType[type]] + "_vertex", desc, es3fShaderSwitchTests.SwitchType[type], true, switchBody));
            group.addChild(es3fShaderSwitchTests.makeSwitchCase(name + "_" + switchTypeNames[es3fShaderSwitchTests.SwitchType[type]] + "_fragment", desc, es3fShaderSwitchTests.SwitchType[type], false, switchBody));
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fShaderSwitchTests.ShaderSwitchTests = function() {
        tcuTestCase.DeqpTest.call(this, 'switch', 'Switch statement tests');
    };

    es3fShaderSwitchTests.ShaderSwitchTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fShaderSwitchTests.ShaderSwitchTests.prototype.constructor = es3fShaderSwitchTests.ShaderSwitchTests;

    es3fShaderSwitchTests.ShaderSwitchTests.prototype.init = function() {
        // Expected swizzles:
        // 0: xyz
        // 1: wzy
        // 2: yzw
        // 3: zyx
        es3fShaderSwitchTests.makeSwitchCases(this, "basic", "Basic switch statement usage",
            '   switch (${CONDITION})\n' +
            '    {\n' +
            '        case 0:        res = coords.xyz;    break;\n' +
            '        case 1:        res = coords.wzy;    break;\n' +
            '        case 2:        res = coords.yzw;    break;\n' +
            '        case 3:        res = coords.zyx;    break;\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "const_expr_in_label", "Constant expression in label",
            '    const int t = 2;\n' +
            '    switch (${CONDITION})\n' +
            '    {\n' +
            '        case int(0.0):    res = coords.xyz;    break;\n' +
            '        case 2-1:        res = coords.wzy;    break;\n' +
            '        case 3&(1<<1):    res = coords.yzw;    break;\n' +
            '        case t+1:        res = coords.zyx;    break;\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "default_label", "Default label usage",
            '    switch (${CONDITION})\n' +
            '    {\n' +
            '        case 0:        res = coords.xyz;    break;\n' +
            '        case 1:        res = coords.wzy;    break;\n' +
            '        case 3:        res = coords.zyx;    break;\n' +
            '        default:    res = coords.yzw;\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "default_not_last", "Default label usage",
            '    switch (${CONDITION})\n' +
            '    {\n' +
            '        case 0:        res = coords.xyz;    break;\n' +
            '        default:    res = coords.yzw;    break;\n' +
            '        case 1:        res = coords.wzy;    break;\n' +
            '        case 3:        res = coords.zyx;    break;\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "no_default_label", "No match in switch without default label",
            '    res = coords.yzw;\n\n' +
            '    switch (${CONDITION})\n' +
            '    {\n' +
            '        case 0:        res = coords.xyz;    break;\n' +
            '        case 1:        res = coords.wzy;    break;\n' +
            '        case 3:        res = coords.zyx;    break;\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "fall_through", "Fall-through",
            '    switch (${CONDITION})\n' +
            '    {\n' +
            '        case 0:        res = coords.xyz;    break;\n' +
            '        case 1:        res = coords.wzy;    break;\n' +
            '        case 2:        coords = coords.yzwx;\n' +
            '        case 4:        res = vec3(coords);    break;\n' +
            '        case 3:        res = coords.zyx;    break;\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "fall_through_default", "Fall-through",
            '    switch (${CONDITION})\n' +
            '    {\n' +
            '        case 0:        res = coords.xyz;    break;\n' +
            '        case 1:        res = coords.wzy;    break;\n' +
            '        case 3:        res = coords.zyx;    break;\n' +
            '        case 2:        coords = coords.yzwx;\n' +
            '        default:    res = vec3(coords);\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "conditional_fall_through", "Fall-through",
            '    highp vec4 tmp = coords;\n' +
            '    switch (${CONDITION})\n' +
            '    {\n' +
            '        case 0:        res = coords.xyz;    break;\n' +
            '        case 1:        res = coords.wzy;    break;\n' +
            '        case 2:\n' +
            '            tmp = coords.yzwx;\n' +
            '        case 3:\n' +
            '            res = vec3(tmp);\n' +
            '            if (${CONDITION} != 3)\n' +
            '                break;\n' +
            '        default:    res = tmp.zyx;        break;\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "conditional_fall_through_2", "Fall-through",
            '    highp vec4 tmp = coords;\n' +
            '    mediump int c = ${CONDITION};\n' +
            '    switch (c)\n' +
            '    {\n' +
            '        case 0:        res = coords.xyz;    break;\n' +
            '        case 1:        res = coords.wzy;    break;\n' +
            '        case 2:\n' +
            '            c += ${CONDITION};\n' +
            '            tmp = coords.yzwx;\n' +
            '        case 3:\n' +
            '            res = vec3(tmp);\n' +
            '            if (c == 4)\n' +
            '                break;\n' +
            '        default:    res = tmp.zyx;        break;\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "scope", "Basic switch statement usage",
            '    switch (${CONDITION})\n' +
            '    {\n' +
            '        case 0:        res = coords.xyz;    break;\n' +
            '        case 1:        res = coords.wzy;    break;\n' +
            '        case 2:\n' +
            '        {\n' +
            '            mediump vec3 t = coords.yzw;\n' +
            '            res = t;\n' +
            '            break;\n' +
            '        }\n' +
            '        case 3:        res = coords.zyx;    break;\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "switch_in_if", "Switch in for loop",
            '    if (${CONDITION} >= 0)\n' +
            '    {\n' +
            '        switch (${CONDITION})\n' +
            '        {\n' +
            '            case 0:        res = coords.xyz;    break;\n' +
            '            case 1:        res = coords.wzy;    break;\n' +
            '            case 2:        res = coords.yzw;    break;\n' +
            '            case 3:        res = coords.zyx;    break;\n' +
            '        }\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "switch_in_for_loop", "Switch in for loop",
            '    for (int i = 0; i <= ${CONDITION}; i++)\n' +
            '    {\n' +
            '        switch (i)\n' +
            '        {\n' +
            '            case 0:        res = coords.xyz;    break;\n' +
            '            case 1:        res = coords.wzy;    break;\n' +
            '            case 2:        res = coords.yzw;    break;\n' +
            '            case 3:        res = coords.zyx;    break;\n' +
            '        }\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "switch_in_while_loop", "Switch in while loop",
            '    int i = 0;\n' +
            '    while (i <= ${CONDITION})\n' +
            '    {\n' +
            '        switch (i)\n' +
            '        {\n' +
            '            case 0:        res = coords.xyz;    break;\n' +
            '            case 1:        res = coords.wzy;    break;\n' +
            '            case 2:        res = coords.yzw;    break;\n' +
            '            case 3:        res = coords.zyx;    break;\n' +
            '        }\n' +
            '        i += 1;\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "switch_in_do_while_loop", "Switch in do-while loop",
            '    int i = 0;\n' +
            '    do\n' +
            '    {\n' +
            '        switch (i)\n' +
            '        {\n' +
            '            case 0:        res = coords.xyz;    break;\n' +
            '            case 1:        res = coords.wzy;    break;\n' +
            '            case 2:        res = coords.yzw;    break;\n' +
            '            case 3:        res = coords.zyx;    break;\n' +
            '        }\n' +
            '        i += 1;\n' +
            '    } while (i <= ${CONDITION});\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "if_in_switch", "Basic switch statement usage",
            '    switch (${CONDITION})\n' +
            '    {\n' +
            '        case 0:        res = coords.xyz;    break;\n' +
            '        case 1:        res = coords.wzy;    break;\n' +
            '        default:\n' +
            '            if (${CONDITION} == 2)\n' +
            '                res = coords.yzw;\n' +
            '            else\n' +
            '                res = coords.zyx;\n' +
            '            break;\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "for_loop_in_switch", "Basic switch statement usage",
            '    switch (${CONDITION})\n' +
            '    {\n' +
            '        case 0:        res = coords.xyz;    break;\n' +
            '        case 1:\n' +
            '        case 2:\n' +
            '        {\n' +
            '            highp vec3 t = coords.yzw;\n' +
            '            for (int i = 0; i < ${CONDITION}; i++)\n' +
            '                t = t.zyx;\n' +
            '            res = t;\n' +
            '            break;\n' +
            '        }\n' +
            '        default:    res = coords.zyx;    break;\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "while_loop_in_switch", "Basic switch statement usage",
            '    switch (${CONDITION})\n' +
            '    {\n' +
            '        case 0:        res = coords.xyz;    break;\n' +
            '        case 1:\n' +
            '        case 2:\n' +
            '        {\n' +
            '            highp vec3 t = coords.yzw;\n' +
            '            int i = 0;\n' +
            '            while (i < ${CONDITION})\n' +
            '            {\n' +
            '                t = t.zyx;\n' +
            '                i += 1;\n' +
            '            }\n' +
            '            res = t;\n' +
            '            break;\n' +
            '        }\n' +
            '        default:    res = coords.zyx;    break;\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "do_while_loop_in_switch", "Basic switch statement usage",
            '    switch (${CONDITION})\n' +
            '    {\n' +
            '        case 0:        res = coords.xyz;    break;\n' +
            '        case 1:\n' +
            '        case 2:\n' +
            '        {\n' +
            '            highp vec3 t = coords.yzw;\n' +
            '            int i = 0;\n' +
            '            do\n' +
            '            {\n' +
            '                t = t.zyx;\n' +
            '                i += 1;\n' +
            '            } while (i < ${CONDITION});\n' +
            '            res = t;\n' +
            '            break;\n' +
            '        }\n' +
            '        default:    res = coords.zyx;    break;\n' +
            '    }\n');

        es3fShaderSwitchTests.makeSwitchCases(this, "switch_in_switch", "Basic switch statement usage",
            '    switch (${CONDITION})\n' +
            '    {\n' +
            '        case 0:        res = coords.xyz;    break;\n' +
            '        case 1:\n' +
            '        case 2:\n' +
            '            switch (${CONDITION} - 1)\n' +
            '            {\n' +
            '                case 0:        res = coords.wzy;    break;\n' +
            '                case 1:        res = coords.yzw;    break;\n' +
            '            }\n' +
            '            break;\n' +
            '        default:    res = coords.zyx;    break;\n' +
            '}\n');

        // Negative cases.
        // This is being tested somwhere else: data/gles3/shaders/switch.html
    };

    /**
     * Run test
     * @param {WebGL2RenderingContext} context
     */
    es3fShaderSwitchTests.run = function(context) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;
        state.setRoot(new es3fShaderSwitchTests.ShaderSwitchTests());

        //Set up name and description of this test series.
        setCurrentTestName(state.testCases.fullName());
        description(state.testCases.getDescription());
        try {
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fShaderSwitchTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});
