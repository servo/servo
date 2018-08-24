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
goog.provide('framework.referencerenderer.rrRenderer');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deString');
goog.require('framework.delibs.debase.deUtil');
goog.require('framework.opengl.simplereference.sglrShaderProgram');
goog.require('framework.referencerenderer.rrDefs');
goog.require('framework.referencerenderer.rrFragmentOperations');
goog.require('framework.referencerenderer.rrGenericVector');
goog.require('framework.referencerenderer.rrMultisamplePixelBufferAccess');
goog.require('framework.referencerenderer.rrRenderState');
goog.require('framework.referencerenderer.rrShadingContext');
goog.require('framework.referencerenderer.rrVertexAttrib');
goog.require('framework.referencerenderer.rrVertexPacket');

goog.scope(function() {

var rrRenderer = framework.referencerenderer.rrRenderer;
var rrVertexPacket = framework.referencerenderer.rrVertexPacket;
var rrDefs = framework.referencerenderer.rrDefs;
var rrFragmentOperations = framework.referencerenderer.rrFragmentOperations;
var deMath = framework.delibs.debase.deMath;
var tcuTextureUtil = framework.common.tcuTextureUtil;
var tcuTexture = framework.common.tcuTexture;
var rrRenderState = framework.referencerenderer.rrRenderState;
var rrMultisamplePixelBufferAccess = framework.referencerenderer.rrMultisamplePixelBufferAccess;
var rrShadingContext = framework.referencerenderer.rrShadingContext;
var rrGenericVector = framework.referencerenderer.rrGenericVector;
var sglrShaderProgram = framework.opengl.simplereference.sglrShaderProgram;
var rrVertexAttrib = framework.referencerenderer.rrVertexAttrib;
var deString = framework.delibs.debase.deString;
var deUtil = framework.delibs.debase.deUtil;

/**
 * @enum
 */
rrRenderer.PrimitiveType = {
    TRIANGLES: 0, //!< Separate rrRenderer.triangles
    TRIANGLE_STRIP: 1, //!< rrRenderer.Triangle strip
    TRIANGLE_FAN: 2, //!< rrRenderer.Triangle fan

    LINES: 3, //!< Separate lines
    LINE_STRIP: 4, //!< Line strip
    LINE_LOOP: 5, //!< Line loop

    POINTS: 6 //!< Points
};

// /**
//  * @constructor
//  * @param {boolean} depthEnabled Is depth buffer enabled
//  */
// rrRenderer.RasterizationInternalBuffers = function(depthEnabled) {
//     /*std::vector<rrFragmentOperations.Fragment>*/ this.fragmentPackets = [];
//     /*std::vector<GenericVec4>*/ this.shaderOutputs = [];
//     /*std::vector<Fragment>*/ this.shadedFragments = [];
//     /*float**/ this.fragmentDepthBuffer = depthEnabled ? [] : null;
// };

/**
 * @constructor
 * @param {number=} id
 */
rrRenderer.DrawContext = function(id) {
    this.primitiveID = id || 0;

};

/**
 * Transform [x, y] to window (pixel) coordinates.
 * z and w are unchanged
 * @param {rrRenderState.RenderState} state
 * @param {rrVertexPacket.VertexPacket} packet
 * Wreturn {Array<number>}
 */
rrRenderer.transformGLToWindowCoords = function(state, packet) {
    var transformed = [packet.position[0] / packet.position[3],
                                packet.position[1] / packet.position[3],
                                packet.position[2],
                                packet.position[3]];
    var viewport = state.viewport.rect;
    var halfW = viewport.width / 2;
    var halfH = viewport.height / 2;
    var oX = viewport.left + halfW;
    var oY = viewport.bottom + halfH;

    return [
        transformed[0] * halfW + oX,
        transformed[1] * halfH + oY,
        transformed[2],
        transformed[3]
    ];
};

/**
 * @constructor
 * @param {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} colorMultisampleBuffer
 * @param {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess=} depthMultisampleBuffer
 * @param {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess=} stencilMultisampleBuffer
 */
rrRenderer.RenderTarget = function(colorMultisampleBuffer, depthMultisampleBuffer, stencilMultisampleBuffer) {
    this.MAX_COLOR_BUFFERS = 4;
    this.colorBuffers = [];
    this.colorBuffers[0] = colorMultisampleBuffer;
    this.depthBuffer = depthMultisampleBuffer || new rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess();
    this.stencilBuffer = stencilMultisampleBuffer || new rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess();
    this.numColorBuffers = 1;
};

// NOTE: Program object is useless. Let's just use the sglrShaderProgram
// /**
//  * @constructor
//  * @param {rrShaders.VertexShader} vertexShader_
//  * @param {rrShaders.FragmentShader} fragmentShader_
//  */
// var Program = function(vertexShader_, fragmentShader_) {
//     this.vertexShader = vertexShader_;
//     this.fragmentShader = fragmentShader_;
// };

/**
 * @constructor
 * @param {ArrayBuffer} data
 * @param {rrDefs.IndexType} type
 * @param {number} offset
 * @param {number=} baseVertex_
 */
rrRenderer.DrawIndices = function(data, type, offset, baseVertex_) {
    /** @type {ArrayBuffer} */ this.data = data;
    /** @type {number} */ this.baseVertex = baseVertex_ || 0;
    /** @type {rrDefs.IndexType} */ this.indexType = type;
    /** @type {goog.NumberArray} */ this.access = null;
    switch (type) {
        case rrDefs.IndexType.INDEXTYPE_UINT8: this.access = new Uint8Array(data).subarray(offset); break;
        case rrDefs.IndexType.INDEXTYPE_UINT16: this.access = new Uint16Array(data).subarray(offset / 2); break;
        case rrDefs.IndexType.INDEXTYPE_UINT32: this.access = new Uint32Array(data).subarray(offset / 4); break;
        default: throw new Error('Invalid type: ' + type);
    }
};

/**
 * @return {number}
 */
rrRenderer.DrawIndices.prototype.readIndexArray = function(index) { return this.access[index]; };

/**
 * @constructor
 * @param {rrRenderer.PrimitiveType} primitiveType
 * @param {number} numElements
 * @param {(number|rrRenderer.DrawIndices)} indices
 */
rrRenderer.PrimitiveList = function(primitiveType, numElements, indices) {
    /** @type {rrRenderer.PrimitiveType} */ this.m_primitiveType = primitiveType;
    /** @type {number} */ this.m_numElements = numElements;
    if (typeof indices == 'number') {
        // !< primitive list for drawArrays-like call
        this.m_indices = null;
        this.m_indexType = null;
        this.m_baseVertex = indices;
    } else {
        // !< primitive list for drawElements-like call
        this.m_indices = indices;
        this.m_indexType = indices.indexType;
        this.m_baseVertex = indices.baseVertex;
    }
    this.m_iterator = 0;
};

/**
 * @param {number} elementNdx
 * @return {number}
 */
rrRenderer.PrimitiveList.prototype.getIndex = function(elementNdx) {
    if (this.m_indices) {
        var index = this.m_baseVertex + this.m_indices.readIndexArray(elementNdx);
        if (index < 0)
            throw new Error('Index must not be negative');

        return index;
    } else
        return this.m_baseVertex + elementNdx;
};

/**
 * @param {number} elementNdx
 * @param {number} restartIndex
 * @return {boolean}
 */
rrRenderer.PrimitiveList.prototype.isRestartIndex = function(elementNdx, restartIndex) {
    // implicit index or explicit index (without base vertex) equals restart
    if (this.m_indices)
        return this.m_indices.readIndexArray(elementNdx) == restartIndex;
    else
        return elementNdx == restartIndex;
};

/**
 * @return {number}
 */
rrRenderer.PrimitiveList.prototype.getNumElements = function() {return this.m_numElements;};

/**
 * @return {rrRenderer.PrimitiveType}
 */
rrRenderer.PrimitiveList.prototype.getPrimitiveType = function() {return this.m_primitiveType;};

/**
 * @return {?rrDefs.IndexType}
 */
rrRenderer.PrimitiveList.prototype.getIndexType = function() {return this.m_indexType;};

/**
 * Generate a primitive from indices
 * @param {boolean=} reset Restart generating primitives. Default false
 * @return {Array<number>}
 */
rrRenderer.PrimitiveList.prototype.getNextPrimitive = function(reset) {
    if (reset)
        this.m_iterator = 0;
    var result = [];
    var i = this.m_iterator;
    switch (this.m_primitiveType) {
        case rrRenderer.PrimitiveType.TRIANGLES:
            if (this.m_iterator + 3 <= this.m_numElements) {
                result = [i, i + 1, i + 2];
                this.m_iterator += 3;
            }
            break;
        case rrRenderer.PrimitiveType.TRIANGLE_STRIP:
            if (this.m_iterator + 3 <= this.m_numElements) {
                result = [i, i + 1, i + 2];
                this.m_iterator += 1;
            }
            break;
        case rrRenderer.PrimitiveType.TRIANGLE_FAN:
            if (this.m_iterator + 3 <= this.m_numElements) {
                result = [0, i + 1, i + 2];
                this.m_iterator += 1;
            }
            break;
        case rrRenderer.PrimitiveType.LINES:
            if (this.m_iterator + 2 <= this.m_numElements) {
                result = [i, i + 1];
                this.m_iterator += 2;
            }
            break;
        case rrRenderer.PrimitiveType.LINE_STRIP:
            if (this.m_iterator + 2 <= this.m_numElements) {
                result = [i, i + 1];
                this.m_iterator += 1;
            }
            break;
        case rrRenderer.PrimitiveType.LINE_LOOP:
            if (this.m_iterator == this.m_numElements)
                break;
            if (this.m_iterator + 2 <= this.m_numElements)
                result = [i, i + 1];
            else
                result = [i, 0];
            this.m_iterator += 1;
            break;
        case rrRenderer.PrimitiveType.POINTS:
            if (this.m_iterator == this.m_numElements)
                break;
            else
                result = [i];
            this.m_iterator += 1;
            break;
        default:
            throw new Error('Unsupported primitive type: ' + deString.enumToString(rrRenderer.PrimitiveType, this.m_primitiveType));
    }

    return result;
};

/**
 * @param {rrRenderState.RenderState} state
 * @param {rrRenderer.RenderTarget} renderTarget
 * @param {Array<rrFragmentOperations.Fragment>} fragments Fragments to write
*/
rrRenderer.writeFragments = function(state, renderTarget, fragments) {
    /* TODO: Add blending, depth, stencil ... */
    var colorbuffer = renderTarget.colorBuffers[0].raw();
    for (var i = 0; i < fragments.length; i++) {
        var fragment = fragments[i];
        colorbuffer.setPixel(fragment.value, 0, fragment.pixelCoord[0], fragment.pixelCoord[1]);
    }

};

/**
 * @param {rrRenderState.RenderState} renderState
 * @param {rrRenderer.RenderTarget} renderTarget
 * @param {Array<rrFragmentOperations.Fragment>} fragments Fragments to write
*/
rrRenderer.writeFragments2 = function(renderState, renderTarget, fragments) {
    /*
void FragmentProcessor::render (const rr::MultisamplePixelBufferAccess& msColorBuffer,
                                const rr::MultisamplePixelBufferAccess& msDepthBuffer,
                                const rr::MultisamplePixelBufferAccess& msStencilBuffer,
                                const Fragment*                             fragments,
                                int numFragments,
                                FaceType fragmentFacing,
                                const FragmentOperationState& state)
*/

    /** @const */ var fragmentFacing = rrDefs.FaceType.FACETYPE_FRONT;
    var colorBuffer = renderTarget.colorBuffers[0].raw();
    var depthBuffer = renderTarget.depthBuffer.raw();
    var stencilBuffer = renderTarget.stencilBuffer.raw();
    var state = renderState.fragOps;

    var hasDepth = depthBuffer.getWidth() > 0 && depthBuffer.getHeight() > 0 && depthBuffer.getDepth() > 0;
    var hasStencil = stencilBuffer.getWidth() > 0 && stencilBuffer.getHeight() > 0 && stencilBuffer.getDepth() > 0;
    var doDepthTest = hasDepth && state.depthTestEnabled;
    var doStencilTest = hasStencil && state.stencilTestEnabled;

    var colorbufferClass = tcuTexture.getTextureChannelClass(colorBuffer.getFormat().type);
    var fragmentDataType = rrGenericVector.GenericVecType.FLOAT;
    switch (colorbufferClass) {
        case tcuTexture.TextureChannelClass.SIGNED_INTEGER:
            fragmentDataType = rrGenericVector.GenericVecType.INT32;
            break;
        case tcuTexture.TextureChannelClass.UNSIGNED_INTEGER:
            fragmentDataType = rrGenericVector.GenericVecType.UINT32;
            break;
    }

    if (!((!hasDepth || colorBuffer.getWidth() == depthBuffer.getWidth()) && (!hasStencil || colorBuffer.getWidth() == stencilBuffer.getWidth())))
        throw new Error('Attachment must have the same width');
    if (!((!hasDepth || colorBuffer.getHeight() == depthBuffer.getHeight()) && (!hasStencil || colorBuffer.getHeight() == stencilBuffer.getHeight())))
        throw new Error('Attachment must have the same height');
    if (!((!hasDepth || colorBuffer.getDepth() == depthBuffer.getDepth()) && (!hasStencil || colorBuffer.getDepth() == stencilBuffer.getDepth())))
        throw new Error('Attachment must have the same depth');

    var stencilState = state.stencilStates[fragmentFacing];
    var colorMaskFactor = [state.colorMask[0] ? 1 : 0, state.colorMask[1] ? 1 : 0, state.colorMask[2] ? 1 : 0, state.colorMask[3] ? 1 : 0];
    var colorMaskNegationFactor = [state.colorMask[0] ? false : true, state.colorMask[1] ? false : true, state.colorMask[2] ? false : true, state.colorMask[3] ? false : true];
    var sRGBTarget = state.sRGBEnabled && colorBuffer.getFormat().isSRGB();

    // Scissor test.

    if (state.scissorTestEnabled)
        rrFragmentOperations.executeScissorTest(fragments, state.scissorRectangle);

    // Stencil test.

    if (doStencilTest) {
        rrFragmentOperations.executeStencilCompare(fragments, stencilState, state.numStencilBits, stencilBuffer);
        rrFragmentOperations.executeStencilSFail(fragments, stencilState, state.numStencilBits, stencilBuffer);
    }

    // Depth test.
    // \note Current value of isAlive is needed for dpPass and dpFail, so it's only updated after them and not right after depth test.

    if (doDepthTest) {
        rrFragmentOperations.executeDepthCompare(fragments, state.depthFunc, depthBuffer);

        if (state.depthMask)
            rrFragmentOperations.executeDepthWrite(fragments, depthBuffer);
    }

    // Do dpFail and dpPass stencil writes.

    if (doStencilTest)
        rrFragmentOperations.executeStencilDpFailAndPass(fragments, stencilState, state.numStencilBits, stencilBuffer);

    // Kill the samples that failed depth test.

    if (doDepthTest) {
        for (var i = 0; i < fragments.length; i++)
            fragments[i].isAlive = fragments[i].isAlive && fragments[i].depthPassed;
    }

    // Paint fragments to target

    switch (fragmentDataType) {
        case rrGenericVector.GenericVecType.FLOAT:
            // Blend calculation - only if using blend.
            if (state.blendMode == rrRenderState.BlendMode.STANDARD) {
                // Put dst color to register, doing srgb-to-linear conversion if needed.
                for (var i = 0; i < fragments.length; i++) {
                    var frag = fragments[i];
                    if (frag.isAlive) {
                        var dstColor = colorBuffer.getPixel(0, frag.pixelCoord[0], frag.pixelCoord[1]);

                        /* TODO: Check frag.value and frag.value1 types */
                        frag.clampedBlendSrcColor = deMath.clampVector(frag.value, 0, 1);
                        frag.clampedBlendSrc1Color = deMath.clampVector(frag.value1, 0, 1);
                        frag.clampedBlendDstColor = deMath.clampVector(sRGBTarget ? tcuTexture.sRGBToLinear(dstColor) : dstColor, 0, 1);
                    }
                }

                // Calculate blend factors to register.
                rrFragmentOperations.executeBlendFactorComputeRGB(fragments, state.blendColor, state.blendRGBState);
                rrFragmentOperations.executeBlendFactorComputeA(fragments, state.blendColor, state.blendAState);

                // Compute blended color.
                rrFragmentOperations.executeBlend(fragments, state.blendRGBState, state.blendAState);
            } else {
                // Not using blend - just put values to register as-is.

                for (var i = 0; i < fragments.length; i++) {
                    var frag = fragments[i];
                    if (frag.isAlive) {
                        frag.blendedRGB = deMath.swizzle(frag.value, [0, 1, 2]);
                        frag.blendedA = frag.value[3];
                    }
                }
            }

            // Finally, write the colors to the color buffer.

            if (state.colorMask[0] && state.colorMask[1] && state.colorMask[2] && state.colorMask[3]) {
                /* TODO: Add quick path */
                // if (colorBuffer.getFormat().isEqual(new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8)))
                //     executeRGBA8ColorWrite(fragments, colorBuffer);
                // else
                    rrFragmentOperations.executeColorWrite(fragments, sRGBTarget, colorBuffer);
            } else if (state.colorMask[0] || state.colorMask[1] || state.colorMask[2] || state.colorMask[3])
                rrFragmentOperations.executeMaskedColorWrite(fragments, colorMaskFactor, colorMaskNegationFactor, sRGBTarget, colorBuffer);
            break;

        case rrGenericVector.GenericVecType.INT32:
            // Write fragments
            for (var i = 0; i < fragments.length; i++) {
                var frag = fragments[i];
                if (frag.isAlive) {
                    frag.signedValue = frag.value;
                }
            }

            if (state.colorMask[0] || state.colorMask[1] || state.colorMask[2] || state.colorMask[3])
                rrFragmentOperations.executeSignedValueWrite(fragments, state.colorMask, colorBuffer);
            break;

        case rrGenericVector.GenericVecType.UINT32:
            // Write fragments
           for (var i = 0; i < fragments.length; i++) {
                var frag = fragments[i];
                if (frag.isAlive) {
                    frag.unsignedValue = frag.value;
                }
            }

            if (state.colorMask[0] || state.colorMask[1] || state.colorMask[2] || state.colorMask[3])
                rrFragmentOperations.executeUnsignedValueWrite(fragments, state.colorMask, colorBuffer);
            break;

        default:
            throw new Error('Unrecognized fragment data type:' + fragmentDataType);
    }
};

/**
 * Determines the index of the corresponding vertex according to top/right conditions.
 * @param {boolean} isTop
 * @param {boolean} isRight
 * @return {number}
 */
rrRenderer.getIndexOfCorner = function(isTop, isRight, vertexPackets) {
    var x = null;
    var y = null;

    var xcriteria = isRight ? Math.max : Math.min;
    var ycriteria = isTop ? Math.max : Math.min;

    // Determine corner values
    for (var i = 0; i < vertexPackets.length; i++) {
        x = x != null ? xcriteria(vertexPackets[i].position[0], x) : vertexPackets[i].position[0];
        y = y != null ? ycriteria(vertexPackets[i].position[1], y) : vertexPackets[i].position[1];
    }

    // Search for matching vertex
    for (var v = 0; v < vertexPackets.length; v++)
        if (vertexPackets[v].position[0] == x &&
            vertexPackets[v].position[1] == y)
            return v;

    throw new Error('Corner not found');
};

/**
 * Check that point is in the clipping volume
 * @param {number} x
 * @param {number} y
 * @param {number} z
 * @param {rrRenderState.WindowRectangle} rect
 * @return {boolean}
 */
rrRenderer.clipTest = function(x, y, z, rect) {
    x = Math.round(x);
    y = Math.round(y);
    if (!deMath.deInBounds32(x, rect.left, rect.left + rect.width))
        return false;
    if (!deMath.deInBounds32(y, rect.bottom, rect.bottom + rect.height))
        return false;
    if (z < 0 || z > 1)
        return false;
    return true;
};

// Rasterizer configuration
rrRenderer.RASTERIZER_SUBPIXEL_BITS = 8;
rrRenderer.RASTERIZER_MAX_SAMPLES_PER_FRAGMENT = 16;

// Referenced from rrRasterizer.hpp

/**
 * Get coverage bit value
 * @param {number} numSamples
 * @param {number} x
 * @param {number} y
 * @param {number} sampleNdx
 * @return {number}
 */
rrRenderer.getCoverageBit = function(numSamples, x, y, sampleNdx) {
    var maxSamples = 16;
    assertMsgOptions(maxSamples >= rrRenderer.RASTERIZER_MAX_SAMPLES_PER_FRAGMENT, 'maxSamples should not greater than ' + rrRenderer.RASTERIZER_MAX_SAMPLES_PER_FRAGMENT, false, true);
    assertMsgOptions(deMath.deInRange32(numSamples, 1, maxSamples) && deMath.deInBounds32(x, 0, 2) && deMath.deInBounds32(y, 0, 2), 'numSamples, x or y not in bound', false, true);
    return 1 << ((x * 2 + y) * numSamples + sampleNdx);
};

/**
 * Get all sample bits for fragment
 * @param {number} numSamples
 * @param {number} x
 * @param {number} y
 * @return {number}
 */
rrRenderer.getCoverageFragmentSampleBits = function(numSamples, x, y) {
    assertMsgOptions(deMath.deInBounds32(x, 0, 2) && deMath.deInBounds32(y, 0, 2), 'x or y is not in bound 0 to 2', false, true);
    var fragMask = (1 << numSamples) - 1;
    return fragMask << (x * 2 + y) * numSamples;
};

/**
 * Set coverage bit in coverage mask
 * @param {number} mask
 * @param {number} numSamples
 * @param {number} x
 * @param {number} y
 * @param {number} sampleNdx
 * @param {number} val
 * @return {number}
 */
rrRenderer.setCoverageValue = function(mask, numSamples, x, y, sampleNdx, val) {
    var bit = rrRenderer.getCoverageBit(numSamples, x, y, sampleNdx);
    return val ? (mask | bit) : (mask & ~bit);
};

/**
 * Test if any sample for fragment is live
 * @param {number} mask
 * @param {number} numSamples
 * @param {number} x
 * @param {number} y
 * @return {number}
 */
rrRenderer.getCoverageAnyFragmentSampleLive = function(mask, numSamples, x, y) {
    return (mask & rrRenderer.getCoverageFragmentSampleBits(numSamples, x, y)) != 0;
};

// Referenced from rrRasterizer.cpp

/**
 * Pixel coord to sub pixel coord
 * @param {number} v
 * @return {number}
 */
rrRenderer.toSubpixelCoord = function(v) {
    return Math.trunc(v * (1 << rrRenderer.RASTERIZER_SUBPIXEL_BITS) + (v < 0 ? -0.5 : 0.5));
};

/**
 * Floor sub pixel coord to pixel coord
 * @param {number} coord
 * @param {boolean} fillEdge
 * @return {number}
 */
rrRenderer.floorSubpixelToPixelCoord = function(coord, fillEdge) {
    if (coord >= 0)
        return Math.trunc((coord - (fillEdge ? 1 : 0)) >> rrRenderer.RASTERIZER_SUBPIXEL_BITS);
    else
        return Math.trunc((coord - ((1 << rrRenderer.RASTERIZER_SUBPIXEL_BITS) - (fillEdge ? 0 : 1))) >> rrRenderer.RASTERIZER_SUBPIXEL_BITS);
};

/**
 * Ceil sub pixel coord to pixel coord
 * @param {number} coord
 * @param {boolean} fillEdge
 * @return {number}
 */
rrRenderer.ceilSubpixelToPixelCoord = function(coord, fillEdge) {
    if (coord >= 0)
        return Math.trunc((coord + (1 << rrRenderer.RASTERIZER_SUBPIXEL_BITS) - (fillEdge ? 0 : 1)) >> rrRenderer.RASTERIZER_SUBPIXEL_BITS);
    else
        return Math.trunc((coord + (fillEdge ? 1 : 0)) >> rrRenderer.RASTERIZER_SUBPIXEL_BITS);
};

/**
 * \brief Edge function - referenced from struct EdgeFunction in rrRasterizer.hpp
 *
 * Edge function can be evaluated for point P (in a fixed-point coordinates
 * with RASTERIZER_SUBPIXEL_BITS fractional part) by computing
 * D = a * Px + b * Py + c
 *
 * D will be fixed-point value where lower (RASTERIZER_SUBPIXEL_BITS * 2) bits
 * will be fractional part.
 *
 * Member function evaluateEdge, reverseEdge and isInsideCCW are referenced from rrRasterizer.cpp.
 *
 * @param {number} a
 * @param {number} b
 * @param {number} c
 * @param {boolean} inclusive
 */
rrRenderer.edgeFunction = function(a, b, c, inclusive) {
    this.a = a;
    this.b = b;
    this.c = c;
    this.inclusive = inclusive; // True if edge is inclusive according to fill rules
};

/**
 * Evaluate point (x,y)
 * @param {number} x
 * @param {number} y
 * @return {number}
 */
rrRenderer.edgeFunction.prototype.evaluateEdge = function(x, y) {
    return this.a * x + this.b * y + this.c;
};

/**
 * Reverse edge (e.g. from CCW to CW)
 */
rrRenderer.edgeFunction.prototype.reverseEdge = function () {
    this.a = -this.a;
    this.b = -this.b;
    this.c = -this.c;
    this.inclusive = !this.inclusive;
};

/**
 * Determine if a point with value edgeVal is inside the CCW region of the edge
 * @param {number} edgeVal
 * @return {boolean}
 */
rrRenderer.edgeFunction.prototype.isInsideCCW = function(edgeVal) {
    return this.inclusive ? edgeVal >= 0 : edgeVal > 0;
};

/**
 * Init an edge function in counter-clockwise (CCW) orientation
 * @param {number} horizontalFill
 * @param {number} verticalFill
 * @param {number} x0
 * @param {number} y0
 * @param {number} x1
 * @param {number} y1
 * @return {rrRenderer.edgeFunction}
 */
rrRenderer.initEdgeCCW = function(horizontalFill, verticalFill, x0, y0, x1, y1) {
    var xd = x1 - x0;
    var yd = y1 - y0;
    var inclusive = false;

    if (yd == 0)
        inclusive = verticalFill == rrRenderState.VerticalFill.BOTTOM ? xd >= 0 : xd <= 0;
    else
        inclusive = horizontalFill == rrRenderState.HorizontalFill.LEFT ? yd <= 0 : yd >=0;

    return new rrRenderer.edgeFunction(y0 - y1, x1 - x0, x0 * y1 - y0 * x1, inclusive);
};

/**
 * \brief Triangle rasterizer - referenced from class TriangleRasterizer in rrRasterizer.hpp
 *
 * Triangle rasterizer implements following features:
 * - Rasterization using fixed-point coordinates
 * - 1-sample rasterization (the value of numSamples always equals 1 in sglrReferenceContext)
 * - Depth interpolation
 * - Perspective-correct barycentric computation for interpolation
 * - Visible face determination
 * - Clipping - native dEQP does clipping before rasterization; see function drawBasicPrimitives
 *              in rrRenderer.cpp for more details
 *
 * It does not (and will not) implement following:
 * - Triangle setup
 * - Degenerate elimination
 * - Coordinate transformation (inputs are in screen-space)
 * - Culling - logic can be implemented outside by querying visible face
 * - Scissoring - (this can be done by controlling viewport rectangle)
 * - Any per-fragment operations
 *
 * @param {rrRenderState.RenderState} state
 */
rrRenderer.triangleRasterizer = function(state) {
    this.m_viewport = state.viewport;
    this.m_winding = state.rasterization.winding;
    this.m_horizontalFill = state.rasterization.horizontalFill;
    this.m_verticalFill = state.rasterization.verticalFill;
};

/**
 * Initialize triangle rasterization
 * @param {vec} v0  Screen-space coordinates (x, y, z) and 1/w for vertex 0
 * @param {vec} v1  Screen-space coordinates (x, y, z) and 1/w for vertex 1
 * @param {vec} v2  Screen-space coordinates (x, y, z) and 1/w for vertex 2
 */
rrRenderer.triangleRasterizer.prototype.init = function(v0, v1, v2) {
    this.m_v0 = v0;
    this.m_v1 = v1;
    this.m_v2 = v2;

    // Positions in fixed-point coordinates
    var x0 = rrRenderer.toSubpixelCoord(v0[0]);
    var y0 = rrRenderer.toSubpixelCoord(v0[1]);
    var x1 = rrRenderer.toSubpixelCoord(v1[0]);
    var y1 = rrRenderer.toSubpixelCoord(v1[1]);
    var x2 = rrRenderer.toSubpixelCoord(v2[0]);
    var y2 = rrRenderer.toSubpixelCoord(v2[1]);

    // Initialize edge functions
    if (this.m_winding == rrRenderState.Winding.CCW) {
        this.m_edge01 = rrRenderer.initEdgeCCW(this.m_horizontalFill, this.m_verticalFill, x0, y0, x1, y1);
        this.m_edge12 = rrRenderer.initEdgeCCW(this.m_horizontalFill, this.m_verticalFill, x1, y1, x2, y2);
        this.m_edge20 = rrRenderer.initEdgeCCW(this.m_horizontalFill, this.m_verticalFill, x2, y2, x0, y0);
    } else {
        // Reverse edges
        this.m_edge01 = rrRenderer.initEdgeCCW(this.m_horizontalFill, this.m_verticalFill, x1, y1, x0, y0);
        this.m_edge12 = rrRenderer.initEdgeCCW(this.m_horizontalFill, this.m_verticalFill, x2, y2, x1, y1);
        this.m_edge20 = rrRenderer.initEdgeCCW(this.m_horizontalFill, this.m_verticalFill, x0, y0, x2, y2);
    }

    // Determine face
    var s = this.m_edge01.evaluateEdge(x2, y2);
    var positiveArea = (this.m_winding == rrRenderState.Winding.CCW ) ? s > 0 : s < 0;
    this.m_face = positiveArea ? rrDefs.FaceType.FACETYPE_FRONT : rrDefs.FaceType.FACETYPE_BACK;
    if (!positiveArea) {
        // Reverse edges so that we can use CCW area tests & interpolation
        this.m_edge01.reverseEdge();
        this.m_edge12.reverseEdge();
        this.m_edge20.reverseEdge();
    }

    // Bounding box
    var minX = Math.min(x0, x1, x2);
    var maxX = Math.max(x0, x1, x2);
    var minY = Math.min(y0, y1, y2);
    var maxY = Math.max(y0, y1, y2);

    this.m_bboxMin = [];
    this.m_bboxMax = [];
    this.m_bboxMin[0] = rrRenderer.floorSubpixelToPixelCoord(minX, this.m_horizontalFill == rrRenderState.HorizontalFill.LEFT);
    this.m_bboxMin[1] = rrRenderer.floorSubpixelToPixelCoord(minY, this.m_verticalFill == rrRenderState.VerticalFill.BOTTOM);
    this.m_bboxMax[0] = rrRenderer.ceilSubpixelToPixelCoord(maxX, this.m_horizontalFill == rrRenderState.HorizontalFill.RIGHT);
    this.m_bboxMax[1] = rrRenderer.ceilSubpixelToPixelCoord(maxY, this.m_verticalFill == rrRenderState.VerticalFill.TOP);

    // Clamp to viewport
    var wX0 = this.m_viewport.rect.left;
    var wY0 = this.m_viewport.rect.bottom;
    var wX1 = wX0 + this.m_viewport.rect.width - 1;
    var wY1 = wY0 + this.m_viewport.rect.height - 1;

    this.m_bboxMin[0] = deMath.clamp(this.m_bboxMin[0], wX0, wX1);
    this.m_bboxMin[1] = deMath.clamp(this.m_bboxMin[1], wY0, wY1);
    this.m_bboxMax[0] = deMath.clamp(this.m_bboxMax[0], wX0, wX1);
    this.m_bboxMax[1] = deMath.clamp(this.m_bboxMax[1], wY0, wY1);

    this.m_curPos = [this.m_bboxMin[0], this.m_bboxMin[1]];
};

rrRenderer.triangleRasterizer.prototype.rasterize = function() {
    var fragmentPackets = [];
    var halfPixel = 1 << (rrRenderer.RASTERIZER_SUBPIXEL_BITS - 1);

    // For depth interpolation; given barycentrics A, B, C = (1 - A -B)
    // We can reformulate the usual z = z0 * A + z1 * B + z2 * C into more
    // stable equation z = A * (z0 - z2) + B * (z1 - z2) + z2
    var za = this.m_v0[2] - this.m_v2[2];
    var zb = this.m_v1[2] - this.m_v2[2];
    var zc = this.m_v2[2];

    var zn = this.m_viewport.zn;
    var zf = this.m_viewport.zf;
    var depthScale = (zf - zn) / 2;
    var depthBias = (zf + zn) / 2;

    while (this.m_curPos[1] <= this.m_bboxMax[1]) {
        var x0 = this.m_curPos[0];
        var y0 = this.m_curPos[1];

        // Subpixel coords of (x0, y0), (x0 + 1, y0), (x0, y0 + 1), (x0 + 1, y0 + 1)
        var sx0 = rrRenderer.toSubpixelCoord(x0) + halfPixel;
        var sx1 = rrRenderer.toSubpixelCoord(x0 + 1) + halfPixel;
        var sy0 = rrRenderer.toSubpixelCoord(y0) + halfPixel;
        var sy1 = rrRenderer.toSubpixelCoord(y0 + 1) + halfPixel;

        var sx = [sx0, sx1, sx0, sx1];
        var sy = [sy0, sy0, sy1, sy1];

        // Viewport test
        var outX1 = x0 + 1 == this.m_viewport.rect.left + this.m_viewport.rect.width;
        var outY1 = y0 + 1 == this.m_viewport.rect.bottom + this.m_viewport.rect.height;

        // Coverage
        var coverage = 0;

        // Evaluate edge values
        var e01 = [];
        var e12 = [];
        var e20 = [];
        for (var i = 0; i < 4; i++) {
            e01.push(this.m_edge01.evaluateEdge(sx[i], sy[i]));
            e12.push(this.m_edge12.evaluateEdge(sx[i], sy[i]));
            e20.push(this.m_edge20.evaluateEdge(sx[i], sy[i]));
        }

        // Compute coverage mask
        coverage = rrRenderer.setCoverageValue(coverage, 1, 0, 0, 0, this.m_edge01.isInsideCCW(e01[0]) && this.m_edge12.isInsideCCW(e12[0]) && this.m_edge20.isInsideCCW(e20[0]));
        coverage = rrRenderer.setCoverageValue(coverage, 1, 1, 0, 0, !outX1 && this.m_edge01.isInsideCCW(e01[1]) && this.m_edge12.isInsideCCW(e12[1]) && this.m_edge20.isInsideCCW(e20[1]));
        coverage = rrRenderer.setCoverageValue(coverage, 1, 0, 1, 0, !outY1 && this.m_edge01.isInsideCCW(e01[2]) && this.m_edge12.isInsideCCW(e12[2]) && this.m_edge20.isInsideCCW(e20[2]));
        coverage = rrRenderer.setCoverageValue(coverage, 1, 1, 1, 0, !outX1 && !outY1 && this.m_edge01.isInsideCCW(e01[3]) && this.m_edge12.isInsideCCW(e12[3]) && this.m_edge20.isInsideCCW(e20[3]));

        // Advance to next location
        this.m_curPos[0] += 2;
        if (this.m_curPos[0] > this.m_bboxMax[0]) {
            this.m_curPos[0] = this.m_bboxMin[0];
            this.m_curPos[1] += 2;
        }

        if (coverage == 0)
            continue; // Discard

        // Compute depth and barycentric coordinates
        var edgeSum = deMath.add(deMath.add(e01, e12), e20);
        var z0 = deMath.divide(e12, edgeSum);
        var z1 = deMath.divide(e20, edgeSum);

        var b0 = deMath.multiply(e12, [this.m_v0[3], this.m_v0[3], this.m_v0[3], this.m_v0[3]]);
        var b1 = deMath.multiply(e20, [this.m_v1[3], this.m_v1[3], this.m_v1[3], this.m_v1[3]]);
        var b2 = deMath.multiply(e01, [this.m_v2[3], this.m_v2[3], this.m_v2[3], this.m_v2[3]]);
        var bSum = deMath.add(deMath.add(b0, b1), b2);
        var barycentric0 = deMath.divide(b0, bSum);
        var barycentric1 = deMath.divide(b1, bSum);
        var barycentric2 = deMath.subtract(deMath.subtract([1, 1, 1, 1], barycentric0), barycentric1);

        // In native dEQP, after rasterization, the pixel (x0, y0) actually represents four pixels:
        // (x0, y0), (x0 + 1, y0), (x0, y0 + 1) and (x0 + 1, y0 + 1).
        // The barycentrics and depths of these four pixels are to be computed after rasterization:
        // barycentrics are computed in function shadeFragments in es3fFboTestUtil.cpp;
        // depths are computed in function writeFragmentPackets in rrRenderer.cpp.

        // In js, pixels are processed one after another, so their depths and barycentrics should be computed immediately.

        // Determine if (x0, y0), (x0 + 1, y0), (x0, y0 + 1), (x0 + 1, y0 + 1) can be rendered
        for (var fragNdx = 0; fragNdx < 4; fragNdx++) {
            var xo = fragNdx % 2;
            var yo = Math.trunc(fragNdx / 2);
            var x = x0 + xo;
            var y = y0 + yo;

            // The value of numSamples always equals 1 in sglrReferenceContext.
            if(rrRenderer.getCoverageAnyFragmentSampleLive(coverage, 1, xo, yo)) {
                // Barycentric coordinates - referenced from function readTriangleVarying in rrShadingContext.hpp
                var b = [barycentric0[fragNdx], barycentric1[fragNdx], barycentric2[fragNdx]];

                // Depth - referenced from writeFragmentPackets in rrRenderer.cpp
                var depth = z0[fragNdx] * za + z1[fragNdx] * zb + zc;
                depth = depth * depthScale + depthBias;

                // Clip test
                // Native dEQP does clipping test before rasterization.
                if (!rrRenderer.clipTest(x, y, depth, this.m_viewport.rect))
                    continue;

                fragmentPackets.push(new rrFragmentOperations.Fragment(b, [x, y], depth));
            }
        }
    }
    return fragmentPackets;
};

/**
 * @param {rrRenderState.RenderState} state
 * @param {rrRenderer.RenderTarget} renderTarget
 * @param {sglrShaderProgram.ShaderProgram} program
 * @param {Array<rrVertexAttrib.VertexAttrib>} vertexAttribs
 * @param {rrRenderer.PrimitiveType} primitive
 * @param {(number|rrRenderer.DrawIndices)} first Index of first quad vertex
 * @param {number} count Number of indices
 * @param {number} instanceID
 */
rrRenderer.drawTriangles = function(state, renderTarget, program, vertexAttribs, primitive, first, count, instanceID) {

    /**
     * @param {Array<rrVertexPacket.VertexPacket>} vertices
     * @param {Array<number>} indices
     * @return {Array<rrVertexPacket.VertexPacket>}
     */
    var selectVertices = function(vertices, indices) {
        var result = [];
        for (var i = 0; i < indices.length; i++)
            result.push(vertices[indices[i]]);
        return result;
    };

    // Referenced from native dEQP Renderer::drawInstanced() in rrRenderer.cpp

    var primitives = new rrRenderer.PrimitiveList(primitive, count, first);
    // Do not draw if nothing to draw
    if (primitives.getNumElements() == 0)
        return;

    // Prepare transformation
    var numVaryings = program.vertexShader.getOutputs().length;
    var vpalloc = new rrVertexPacket.VertexPacketAllocator(numVaryings);
    var vertexPackets = vpalloc.allocArray(primitives.getNumElements());
    var drawContext = new rrRenderer.DrawContext();
    drawContext.primitiveID = 0;

    var numberOfVertices = primitives.getNumElements();
    var numVertexPackets = 0;
    for (var elementNdx = 0; elementNdx < numberOfVertices; ++elementNdx) {

        // input
        vertexPackets[numVertexPackets].instanceNdx = instanceID;
        vertexPackets[numVertexPackets].vertexNdx = primitives.getIndex(elementNdx);

        // output
        vertexPackets[numVertexPackets].pointSize = state.point.pointSize; // default value from the current state
        vertexPackets[numVertexPackets].position = [0, 0, 0, 0]; // no undefined values

        ++numVertexPackets;

    }
    program.shadeVertices(vertexAttribs, vertexPackets, numVertexPackets);

    // Referenced from native dEQP Renderer::rasterizePrimitive() for triangle rasterization in rrRenderer.cpp

    // In native dEQP, only maxFragmentPackets packets are processed per rasterize-shade-write loop;
    // in js all packets are processed in one loop.

    var rasterizer = new rrRenderer.triangleRasterizer(state);

    for (var prim = primitives.getNextPrimitive(true); prim.length > 0; prim = primitives.getNextPrimitive()) {
        var vertices = selectVertices(vertexPackets, prim);

        var v0 = rrRenderer.transformGLToWindowCoords(state, vertices[0]);
        var v1 = rrRenderer.transformGLToWindowCoords(state, vertices[1]);
        var v2 = rrRenderer.transformGLToWindowCoords(state, vertices[2]);

        rasterizer.init(v0, v1, v2);

        // Culling
        if ((state.cullMode == rrRenderState.CullMode.FRONT && rasterizer.m_face == rrDefs.FaceType.FACETYPE_FRONT) ||
            (state.cullMode == rrRenderState.CullMode.BACK && rasterizer.m_face == rrDefs.FaceType.FACETYPE_BACK))
        return;

        /* TODO: Add Polygon Offset and Depth Clamp */

        // Compute a conservative integer bounding box for the triangle
        var minX = Math.floor(Math.min(v0[0], v1[0], v2[0]));
        var maxX = Math.ceil(Math.max(v0[0], v1[0], v2[0]));
        var minY = Math.floor(Math.min(v0[1], v1[1], v2[1]));
        var maxY = Math.ceil(Math.max(v0[1], v1[1], v2[1]));

        // Shading context
        var shadingContext = new rrShadingContext.FragmentShadingContext(
            vertices[0].outputs,
            vertices[1].outputs,
            vertices[2].outputs
        );
        shadingContext.setSize(maxX - minX, maxY - minY);

        // Rasterize
        var fragmentPackets = rasterizer.rasterize();

        // Shade
        program.shadeFragments(fragmentPackets, shadingContext);

        // Handle fragment shader outputs
        rrRenderer.writeFragments2(state, renderTarget, fragmentPackets);
    }
};

/**
 * @param {rrRenderState.RenderState} state
 * @param {rrRenderer.RenderTarget} renderTarget
 * @param {sglrShaderProgram.ShaderProgram} program
 * @param {Array<rrVertexAttrib.VertexAttrib>} vertexAttribs
 * @param {rrRenderer.PrimitiveType} primitive
 * @param {(number|rrRenderer.DrawIndices)} first Index of first quad vertex
 * @param {number} count Number of indices
 * @param {number} instanceID
 */
rrRenderer.drawLines = function(state, renderTarget, program, vertexAttribs, primitive, first, count, instanceID) {

    /**
     * @param {Array<rrVertexPacket.VertexPacket>} vertices
     * @param {Array<number>} indices
     * @return {Array<rrVertexPacket.VertexPacket>}
     */
    var selectVertices = function(vertices, indices) {
        var result = [];
        for (var i = 0; i < indices.length; i++)
            result.push(vertices[indices[i]]);
        return result;
    };

    var lengthSquared = function(a) {
        var sqSum = 0;
        for (var i = 0; i < a.length; i++)
            sqSum += a[i] * a[i];
        return sqSum;
    };

    var dot = function(a, b) {
        var res = 0;
        for (var i = 0; i < a.length; i++)
            res += a[i] * b[i];
        return res;
    };

    var rasterizeLine = function(v0, v1) {
        var d = [
            Math.abs(v1[0] - v0[0]),
            Math.abs(v1[1] - v0[1])];
        var xstep = v0[0] < v1[0] ? 1 : -1;
        var ystep = v0[1] < v1[1] ? 1 : -1;
        var x = v0[0];
        var y = v0[1];
        var offset = d[0] - d[1];
        var lenV = [v1[0] - v0[0], v1[1] - v0[1]];
        var lenSq = lengthSquared(lenV);

        var packets = [];

        while (true) {
            var t = dot([x - v0[0], y - v0[1]], lenV) / lenSq;
            var depth = (1 - t) * v0[2] + t * v1[2];
            var b = [0, 0, 0];
            b[0] = 1 - t;
            b[1] = t;

            if (x == v1[0] && y == v1[1])
                break;

            depth = depth * depthScale + depthBias;
            packets.push(new rrFragmentOperations.Fragment(b, [x, y], depth));

            var offset2 = 2 * offset;
            if (offset2 > -1 * d[1]) {
                x += xstep;
                offset -= d[1];
            }

            if (offset2 < d[0]) {
                y += ystep;
                offset += d[0];
            }
        }
        return packets;
    };

    var primitives = new rrRenderer.PrimitiveList(primitive, count, first);
    // Do not draw if nothing to draw
    if (primitives.getNumElements() == 0)
        return;

    // Prepare transformation
    var numVaryings = program.vertexShader.getOutputs().length;
    var vpalloc = new rrVertexPacket.VertexPacketAllocator(numVaryings);
    var vertexPackets = vpalloc.allocArray(primitives.getNumElements());
    var drawContext = new rrRenderer.DrawContext();
    drawContext.primitiveID = 0;

    var numberOfVertices = primitives.getNumElements();
    var numVertexPackets = 0;
    for (var elementNdx = 0; elementNdx < numberOfVertices; ++elementNdx) {

        // input
        vertexPackets[numVertexPackets].instanceNdx = instanceID;
        vertexPackets[numVertexPackets].vertexNdx = primitives.getIndex(elementNdx);

        // output
        vertexPackets[numVertexPackets].pointSize = state.point.pointSize; // default value from the current state
        vertexPackets[numVertexPackets].position = [0, 0, 0, 0]; // no undefined values

        ++numVertexPackets;

    }
    program.shadeVertices(vertexAttribs, vertexPackets, numVertexPackets);

    var zn = state.viewport.zn;
    var zf = state.viewport.zf;
    var depthScale = (zf - zn) / 2;
    var depthBias = (zf + zn) / 2;

    // For each quad, we get a group of six vertex packets
    for (var prim = primitives.getNextPrimitive(true); prim.length > 0; prim = primitives.getNextPrimitive()) {
        var linePackets = selectVertices(vertexPackets, prim);

        var v0 = rrRenderer.transformGLToWindowCoords(state, linePackets[0]);
        var v1 = rrRenderer.transformGLToWindowCoords(state, linePackets[1]);
        v0[2] = linePackets[0].position[2];
        v1[2] = linePackets[1].position[2];

        v0[0] = Math.floor(v0[0]);
        v0[1] = Math.floor(v0[1]);
        v1[0] = Math.floor(v1[0]);
        v1[1] = Math.floor(v1[1]);

        var lineWidth = state.line.lineWidth;

        var shadingContext = new rrShadingContext.FragmentShadingContext(
            linePackets[0].outputs,
            linePackets[1].outputs,
            null
        );
        var isXmajor = Math.abs(v1[0] - v0[0]) >= Math.abs(v1[1] - v0[1]);
        var packets = [];
        if (isXmajor)
            packets = rasterizeLine([v0[0], v0[1] - (lineWidth - 1) / 2, v0[2]],
                                    [v1[0], v1[1] - (lineWidth - 1) / 2, v1[2]]);
        else
            packets = rasterizeLine([v0[0] - (lineWidth - 1) / 2, v0[1], v0[2]],
                                    [v1[0] - (lineWidth - 1) / 2, v1[1], v1[2]]);
        var numPackets = packets.length;
        if (lineWidth > 1)
            for (var i = 0; i < numPackets; i++) {
                var p = packets[i];
                for (var j = 1; j < lineWidth; j++) {
                    var p2 = deUtil.clone(p);
                    if (isXmajor)
                        p2.pixelCoord[1] += j;
                    else
                        p2.pixelCoord[0] += j;
                    packets.push(p2);
                }
            }

        var clipped = [];
        for (var i = 0; i < packets.length; i++) {
            var p = packets[i];
            if (rrRenderer.clipTest(p.pixelCoord[0], p.pixelCoord[1], p.sampleDepths[0], state.viewport.rect))
                clipped.push(p);
        }
        program.shadeFragments(clipped, shadingContext);

        rrRenderer.writeFragments2(state, renderTarget, clipped);
    }
};

/**
 * @param {rrRenderState.RenderState} state
 * @param {rrRenderer.RenderTarget} renderTarget
 * @param {sglrShaderProgram.ShaderProgram} program
 * @param {Array<rrVertexAttrib.VertexAttrib>} vertexAttribs
 * @param {rrRenderer.PrimitiveType} primitive
 * @param {(number|rrRenderer.DrawIndices)} first Index of first quad vertex
 * @param {number} count Number of indices
 * @param {number} instanceID
 */
rrRenderer.drawPoints = function(state, renderTarget, program, vertexAttribs, primitive, first, count, instanceID) {
    /**
     * @param {Array<rrVertexPacket.VertexPacket>} vertices
     * @param {Array<number>} indices
     * @return {Array<rrVertexPacket.VertexPacket>}
     */
    var selectVertices = function(vertices, indices) {
        var result = [];
        for (var i = 0; i < indices.length; i++)
            result.push(vertices[indices[i]]);
        return result;
    };

    var primitives = new rrRenderer.PrimitiveList(primitive, count, first);
    // Do not draw if nothing to draw
    if (primitives.getNumElements() == 0)
        return;

    // Prepare transformation
    var numVaryings = program.vertexShader.getOutputs().length;
    var vpalloc = new rrVertexPacket.VertexPacketAllocator(numVaryings);
    var vertexPackets = vpalloc.allocArray(primitives.getNumElements());
    var drawContext = new rrRenderer.DrawContext();
    drawContext.primitiveID = 0;

    var numberOfVertices = primitives.getNumElements();
    var numVertexPackets = 0;
    for (var elementNdx = 0; elementNdx < numberOfVertices; ++elementNdx) {

        // input
        vertexPackets[numVertexPackets].instanceNdx = instanceID;
        vertexPackets[numVertexPackets].vertexNdx = primitives.getIndex(elementNdx);

        // output
        vertexPackets[numVertexPackets].pointSize = state.point.pointSize; // default value from the current state
        vertexPackets[numVertexPackets].position = [0, 0, 0, 0]; // no undefined values

        ++numVertexPackets;

    }
    program.shadeVertices(vertexAttribs, vertexPackets, numVertexPackets);

    var zn = state.viewport.zn;
    var zf = state.viewport.zf;
    var depthScale = (zf - zn) / 2;
    var depthBias = (zf + zn) / 2;

    // For each primitive, we draw a point.
    for (var prim = primitives.getNextPrimitive(true); prim.length > 0; prim = primitives.getNextPrimitive()) {
        var pointPackets = selectVertices(vertexPackets, prim);

        var v0 = rrRenderer.transformGLToWindowCoords(state, pointPackets[0]);
        v0[2] = pointPackets[0].position[2];
        var pointSize = pointPackets[0].pointSize;

        var shadingContext = new rrShadingContext.FragmentShadingContext(
            pointPackets[0].outputs,
            null,
            null
        );
        var packets = [];

        var x = v0[0];
        var y = v0[1];
        var depth = v0[2];
        var b = [1, 0, 0];
        depth = depth * depthScale + depthBias;

        for (var i = Math.floor(x - pointSize / 2); i < x + pointSize / 2; i++) {
            for (var j = Math.floor(y - pointSize / 2); j < y + pointSize / 2; j++) {
                var centerX = i + 0.5;
                var centerY = j + 0.5;
                if (Math.abs(centerX - x) <= pointSize / 2 &&
                    Math.abs(centerY - y) <= pointSize / 2 &&
                    rrRenderer.clipTest(i, j, depth, state.viewport.rect))
                    packets.push(new rrFragmentOperations.Fragment(b, [i, j], depth));
            }
        }

        program.shadeFragments(packets, shadingContext);

        rrRenderer.writeFragments2(state, renderTarget, packets);
    }
};

});
