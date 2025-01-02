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
goog.provide('functional.gles3.es3fSamplerObjectTests');
goog.require('framework.common.tcuTestCase');
goog.require('modules.shared.glsSamplerObjectTest');

goog.scope(function() {

var es3fSamplerObjectTests = functional.gles3.es3fSamplerObjectTests;
var tcuTestCase = framework.common.tcuTestCase;
var glsSamplerObjectTest = modules.shared.glsSamplerObjectTest;

    /** @type {WebGL2RenderingContext} */ var gl;

    // TODO: implement glsSamplerObjectTest and validate constructors
    es3fSamplerObjectTests.init = function() {
        var testGroup = tcuTestCase.runner.testCases;
        /** @type {Array<glsSamplerObjectTest.TestSpec>} */ var simpleTestCases = [
            new glsSamplerObjectTest.TestSpec('diff_wrap_t', 'Different gl.TEXTURE_WRAP_T', gl.TEXTURE_2D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.MIRRORED_REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_wrap_s', 'Different gl.TEXTURE_WRAP_S', gl.TEXTURE_2D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.MIRRORED_REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_wrap_r', 'Different gl.TEXTURE_WRAP_R', gl.TEXTURE_2D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.MIRRORED_REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_min_filter', 'Different gl.TEXTURE_MIN_FILTER', gl.TEXTURE_2D,
                new glsSamplerObjectTest.SamplingState(gl.LINEAR, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_mag_filter', 'Different gl.TEXTURE_MAG_FILTER', gl.TEXTURE_2D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.LINEAR, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_max_lod', 'Different gl.TEXTURE_MAX_LOD', gl.TEXTURE_2D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, -999.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_min_lod', 'Different gl.TEXTURE_MIN_LOD', gl.TEXTURE_2D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 0.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 100.0, 1000.0)
            )
        ];

        /** @type {tcuTestCase.DeqpTest} */ var simpleTexture2D = tcuTestCase.newTest('single_tex_2d', 'Simple 2D texture with sampler');

        for (var testNdx = 0; testNdx < simpleTestCases.length; testNdx++)
            simpleTexture2D.addChild(new glsSamplerObjectTest.TextureSamplerTest(simpleTestCases[testNdx]));

        testGroup.addChild(simpleTexture2D);

        /** @type {Array<glsSamplerObjectTest.TestSpec>} */ var multiTestCases = [
            new glsSamplerObjectTest.TestSpec('diff_wrap_t', 'Different gl.TEXTURE_WRAP_T', gl.TEXTURE_2D,
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.MIRRORED_REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.MIRRORED_REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_wrap_s', 'Different gl.TEXTURE_WRAP_S', gl.TEXTURE_2D,
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.MIRRORED_REPEAT, gl.REPEAT, -1000.0, 1000.0),
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.MIRRORED_REPEAT, gl.REPEAT, -1000.0, 1000.0),
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_wrap_r', 'Different gl.TEXTURE_WRAP_R', gl.TEXTURE_2D,
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.MIRRORED_REPEAT, -1000.0, 1000.0),
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.MIRRORED_REPEAT, -1000.0, 1000.0),
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_min_filter', 'Different gl.TEXTURE_MIN_FILTER', gl.TEXTURE_2D,
                    new glsSamplerObjectTest.SamplingState(gl.LINEAR, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                    new glsSamplerObjectTest.SamplingState(gl.LINEAR, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_mag_filter', 'Different gl.TEXTURE_MAG_FILTER', gl.TEXTURE_2D,
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.LINEAR, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.LINEAR, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_max_lod', 'Different gl.TEXTURE_MAX_LOD', gl.TEXTURE_2D,
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, -999.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_min_lod', 'Different gl.TEXTURE_MIN_LOD', gl.TEXTURE_2D,
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 0.0, 1000.0),
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 0.0, 1000.0),
                    new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 100.0, 1000.0)
            )
        ];

        /** @type {tcuTestCase.DeqpTest} */ var multiTexture2D = tcuTestCase.newTest('multi_tex_2d', 'Multiple texture units 2D texture with sampler');

        for (var testNdx = 0; testNdx < multiTestCases.length; testNdx++)
            multiTexture2D.addChild(new glsSamplerObjectTest.MultiTextureSamplerTest(multiTestCases[testNdx]));

        testGroup.addChild(multiTexture2D);

        /** @type {Array<glsSamplerObjectTest.TestSpec>} */ var simpleTestCases3D = [
        new glsSamplerObjectTest.TestSpec('diff_wrap_t', 'Different gl.TEXTURE_WRAP_T', gl.TEXTURE_3D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.MIRRORED_REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_wrap_s', 'Different gl.TEXTURE_WRAP_S', gl.TEXTURE_3D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.MIRRORED_REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_wrap_r', 'Different gl.TEXTURE_WRAP_R', gl.TEXTURE_3D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.MIRRORED_REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_min_filter', 'Different gl.TEXTURE_MIN_FILTER', gl.TEXTURE_3D,
                new glsSamplerObjectTest.SamplingState(gl.LINEAR, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_mag_filter', 'Different gl.TEXTURE_MAG_FILTER', gl.TEXTURE_3D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.LINEAR, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_max_lod', 'Different gl.TEXTURE_MAX_LOD', gl.TEXTURE_3D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, -999.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_min_lod', 'Different gl.TEXTURE_MIN_LOD', gl.TEXTURE_3D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 0.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 100.0, 1000.0)
            )
        ];

        /** @type {tcuTestCase.DeqpTest} */ var simpleTexture3D = tcuTestCase.newTest('single_tex_3d', 'Simple 3D texture with sampler');

        for (var testNdx = 0; testNdx < simpleTestCases3D.length; testNdx++)
            simpleTexture3D.addChild(new glsSamplerObjectTest.TextureSamplerTest(simpleTestCases3D[testNdx]));

        testGroup.addChild(simpleTexture3D);

        /** @type {Array<glsSamplerObjectTest.TestSpec>} */ var multiTestCases3D = [
            new glsSamplerObjectTest.TestSpec('diff_wrap_t', 'Different gl.TEXTURE_WRAP_T', gl.TEXTURE_3D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.MIRRORED_REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.MIRRORED_REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_wrap_s', 'Different gl.TEXTURE_WRAP_S', gl.TEXTURE_3D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.MIRRORED_REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.MIRRORED_REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_wrap_r', 'Different gl.TEXTURE_WRAP_R', gl.TEXTURE_3D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.MIRRORED_REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.MIRRORED_REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_min_filter', 'Different gl.TEXTURE_MIN_FILTER', gl.TEXTURE_3D,
                new glsSamplerObjectTest.SamplingState(gl.LINEAR, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.LINEAR, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_mag_filter', 'Different gl.TEXTURE_MAG_FILTER', gl.TEXTURE_3D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.LINEAR, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.LINEAR, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_max_lod', 'Different gl.TEXTURE_MAX_LOD', gl.TEXTURE_3D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, -999.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_min_lod', 'Different gl.TEXTURE_MIN_LOD', gl.TEXTURE_3D,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 0.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 0.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 100.0, 1000.0)
            )
        ];

        /** @type {tcuTestCase.DeqpTest} */ var multiTexture3D = tcuTestCase.newTest('multi_tex_3d', 'Multiple texture units 3D texture with sampler');

        for (var testNdx = 0; testNdx < multiTestCases3D.length; testNdx++)
            multiTexture3D.addChild(new glsSamplerObjectTest.MultiTextureSamplerTest(multiTestCases3D[testNdx]));

        testGroup.addChild(multiTexture3D);

        /** @type {Array<glsSamplerObjectTest.TestSpec>} */ var simpleTestCasesCube = [
            new glsSamplerObjectTest.TestSpec('diff_wrap_t', 'Different gl.TEXTURE_WRAP_T', gl.TEXTURE_CUBE_MAP,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.MIRRORED_REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_wrap_s', 'Different gl.TEXTURE_WRAP_S', gl.TEXTURE_CUBE_MAP,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.MIRRORED_REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_wrap_r', 'Different gl.TEXTURE_WRAP_R', gl.TEXTURE_CUBE_MAP,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.MIRRORED_REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_min_filter', 'Different gl.TEXTURE_MIN_FILTER', gl.TEXTURE_CUBE_MAP,
                new glsSamplerObjectTest.SamplingState(gl.LINEAR, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_mag_filter', 'Different gl.TEXTURE_MAG_FILTER', gl.TEXTURE_CUBE_MAP,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.LINEAR, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_max_lod', 'Different gl.TEXTURE_MAX_LOD', gl.TEXTURE_CUBE_MAP,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, -999.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_min_lod', 'Different gl.TEXTURE_MIN_LOD', gl.TEXTURE_CUBE_MAP,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 0.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 100.0, 1000.0)
            )
        ];

        /** @type {tcuTestCase.DeqpTest} */ var simpleTextureCube = tcuTestCase.newTest('single_cubemap', 'Simple cubemap texture with sampler');

        for (var testNdx = 0; testNdx < simpleTestCasesCube.length; testNdx++)
            simpleTextureCube.addChild(new glsSamplerObjectTest.TextureSamplerTest(simpleTestCasesCube[testNdx]));

        testGroup.addChild(simpleTextureCube);

        /** @type {Array<glsSamplerObjectTest.TestSpec>} */ var multiTestCasesCube = [
            new glsSamplerObjectTest.TestSpec('diff_wrap_t', 'Different gl.TEXTURE_WRAP_T', gl.TEXTURE_CUBE_MAP,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.MIRRORED_REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.MIRRORED_REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_wrap_s', 'Different gl.TEXTURE_WRAP_S', gl.TEXTURE_CUBE_MAP,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.MIRRORED_REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.MIRRORED_REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_wrap_r', 'Different gl.TEXTURE_WRAP_R', gl.TEXTURE_CUBE_MAP,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.MIRRORED_REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.MIRRORED_REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_min_filter', 'Different gl.TEXTURE_MIN_FILTER', gl.TEXTURE_CUBE_MAP,
                new glsSamplerObjectTest.SamplingState(gl.LINEAR, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.LINEAR, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_mag_filter', 'Different gl.TEXTURE_MAG_FILTER', gl.TEXTURE_CUBE_MAP,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.LINEAR, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.LINEAR, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_max_lod', 'Different gl.TEXTURE_MAX_LOD', gl.TEXTURE_CUBE_MAP,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, -1000.0, -999.0)
            ),
            new glsSamplerObjectTest.TestSpec('diff_min_lod', 'Different gl.TEXTURE_MIN_LOD', gl.TEXTURE_CUBE_MAP,
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 0.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 0.0, 1000.0),
                new glsSamplerObjectTest.SamplingState(gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST, gl.REPEAT, gl.REPEAT, gl.REPEAT, 100.0, 1000.0)
            )
        ];

        /** @type {tcuTestCase.DeqpTest} */ var multiTextureCube = tcuTestCase.newTest('multi_cubemap', 'Multiple texture units cubemap textures with sampler');

        for (var testNdx = 0; testNdx < multiTestCasesCube.length; testNdx++)
            multiTextureCube.addChild(new glsSamplerObjectTest.MultiTextureSamplerTest(multiTestCasesCube[testNdx]));

        testGroup.addChild(multiTextureCube);
    };

    es3fSamplerObjectTests.run = function(context) {
        gl = context;
        //Set up Test Root parameters
        var testName = 'sampler_object';
        var testDescription = 'Sampler Object Tests';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.setRoot(tcuTestCase.newTest(testName, testDescription, null));

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);

        try {
            //Create test cases
            es3fSamplerObjectTests.init();
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fSamplerObjectTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});
