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

/*--------------------------------------------------------------------*//*!
 * \brief Map tcu::TextureFormat to GL pixel transfer format.
 *
 * Maps generic texture format description to GL pixel transfer format.
 * If no mapping is found, throws tcu::InternalError.
 *
 * \param texFormat Generic texture format.
 * \return GL pixel transfer format.
 *//*--------------------------------------------------------------------*/
'use strict';
goog.provide('framework.common.tcuCompressedTexture');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');

goog.scope(function() {

var tcuCompressedTexture = framework.common.tcuCompressedTexture;
var tcuTexture = framework.common.tcuTexture;
var deMath = framework.delibs.debase.deMath;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

/**
 * @enum
 */
tcuCompressedTexture.Format = {
    ETC1_RGB8: 0,
    EAC_R11: 1,
    EAC_SIGNED_R11: 2,
    EAC_RG11: 3,
    EAC_SIGNED_RG11: 4,
    ETC2_RGB8: 5,
    ETC2_SRGB8: 6,
    ETC2_RGB8_PUNCHTHROUGH_ALPHA1: 7,
    ETC2_SRGB8_PUNCHTHROUGH_ALPHA1: 8,
    ETC2_EAC_RGBA8: 9,
    ETC2_EAC_SRGB8_ALPHA8: 10,

    ASTC_4x4_RGBA: 11,
    ASTC_5x4_RGBA: 12,
    ASTC_5x5_RGBA: 13,
    ASTC_6x5_RGBA: 14,
    ASTC_6x6_RGBA: 15,
    ASTC_8x5_RGBA: 16,
    ASTC_8x6_RGBA: 17,
    ASTC_8x8_RGBA: 18,
    ASTC_10x5_RGBA: 19,
    ASTC_10x6_RGBA: 20,
    ASTC_10x8_RGBA: 21,
    ASTC_10x10_RGBA: 22,
    ASTC_12x10_RGBA: 23,
    ASTC_12x12_RGBA: 24,
    ASTC_4x4_SRGB8_ALPHA8: 25,
    ASTC_5x4_SRGB8_ALPHA8: 26,
    ASTC_5x5_SRGB8_ALPHA8: 27,
    ASTC_6x5_SRGB8_ALPHA8: 28,
    ASTC_6x6_SRGB8_ALPHA8: 29,
    ASTC_8x5_SRGB8_ALPHA8: 30,
    ASTC_8x6_SRGB8_ALPHA8: 31,
    ASTC_8x8_SRGB8_ALPHA8: 32,
    ASTC_10x5_SRGB8_ALPHA8: 33,
    ASTC_10x6_SRGB8_ALPHA8: 34,
    ASTC_10x8_SRGB8_ALPHA8: 35,
    ASTC_10x10_SRGB8_ALPHA8: 36,
    ASTC_12x10_SRGB8_ALPHA8: 37,
    ASTC_12x12_SRGB8_ALPHA8: 38
};

tcuCompressedTexture.divRoundUp = function(a, b) {
    return Math.floor(a / b) + ((a % b) ? 1 : 0);
};

tcuCompressedTexture.isEtcFormat = function(fmt) {
    // WebGL2 supports ETC2 and EAC formats
    switch (fmt) {
        // case tcuCompressedTexture.Format.ETC1_RGB8:
        case tcuCompressedTexture.Format.EAC_R11:
        case tcuCompressedTexture.Format.EAC_SIGNED_R11:
        case tcuCompressedTexture.Format.EAC_RG11:
        case tcuCompressedTexture.Format.EAC_SIGNED_RG11:
        case tcuCompressedTexture.Format.ETC2_RGB8:
        case tcuCompressedTexture.Format.ETC2_SRGB8:
        case tcuCompressedTexture.Format.ETC2_RGB8_PUNCHTHROUGH_ALPHA1:
        case tcuCompressedTexture.Format.ETC2_SRGB8_PUNCHTHROUGH_ALPHA1:
        case tcuCompressedTexture.Format.ETC2_EAC_RGBA8:
        case tcuCompressedTexture.Format.ETC2_EAC_SRGB8_ALPHA8:
            return true;

        default:
            return false;
    }
};

tcuCompressedTexture.etcDecompressInternal = function() {

var ETC2_BLOCK_WIDTH = 4;
var ETC2_BLOCK_HEIGHT = 4;
var ETC2_UNCOMPRESSED_PIXEL_SIZE_A8 = 1;
var ETC2_UNCOMPRESSED_PIXEL_SIZE_R11 = 2;
var ETC2_UNCOMPRESSED_PIXEL_SIZE_RG11 = 4;
var ETC2_UNCOMPRESSED_PIXEL_SIZE_RGB8 = 3;
var ETC2_UNCOMPRESSED_PIXEL_SIZE_RGBA8 = 4;
var ETC2_UNCOMPRESSED_BLOCK_SIZE_A8 = ETC2_BLOCK_WIDTH * ETC2_BLOCK_HEIGHT * ETC2_UNCOMPRESSED_PIXEL_SIZE_A8;
var ETC2_UNCOMPRESSED_BLOCK_SIZE_R11 = ETC2_BLOCK_WIDTH * ETC2_BLOCK_HEIGHT * ETC2_UNCOMPRESSED_PIXEL_SIZE_R11;
var ETC2_UNCOMPRESSED_BLOCK_SIZE_RG11 = ETC2_BLOCK_WIDTH * ETC2_BLOCK_HEIGHT * ETC2_UNCOMPRESSED_PIXEL_SIZE_RG11;
var ETC2_UNCOMPRESSED_BLOCK_SIZE_RGB8 = ETC2_BLOCK_WIDTH * ETC2_BLOCK_HEIGHT * ETC2_UNCOMPRESSED_PIXEL_SIZE_RGB8;
var ETC2_UNCOMPRESSED_BLOCK_SIZE_RGBA8 = ETC2_BLOCK_WIDTH * ETC2_BLOCK_HEIGHT * ETC2_UNCOMPRESSED_PIXEL_SIZE_RGBA8;

/**
 * @param {ArrayBuffer} src Source ArrayBuffer
 * @return {Uint8Array}
 */
var get64BitBlock = function(src, blockNdx) {
    var block = new Uint8Array(src, blockNdx * 8, 8);
    return block;
};

/**
 * @param {ArrayBuffer} src Source ArrayBuffer
 * Return the first 64 bits of a 128 bit block.
 */
var get128BitBlockStart = function(src, blockNdx) {
    return get64BitBlock(src, 2 * blockNdx);
};

/**
 * @param {ArrayBuffer} src Source ArrayBuffer
 * Return the last 64 bits of a 128 bit block.
 */
var get128BitBlockEnd = function(src, blockNdx) {
    return get64BitBlock(src, 2 * blockNdx + 1);
};

var mask8 = function(src, low, high) {
    if (low > 7 || high < 0)
        return {
            value: 0,
            bits: 0
        };

    var numBits = high - low + 1;
    var mask = (1 << numBits) - 1;

    return {
        value: (src >> low) & mask,
        bits: numBits
    };
};

var getBits64 = function(src, low, high) {
    var result = 0;
    var bits = 0;
    var lowIndex = low;
    var highIndex = high;
    for (var i = 7; i >= 0; i--) {
        var v = mask8(src[i], Math.max(0, lowIndex), Math.min(7, highIndex));
        lowIndex = lowIndex - 8;
        highIndex = highIndex - 8;
        result = result | (v.value << bits);
        bits = v.bits;
    }
    return result;
};

var getBit64 = function(src, bit) {
    return getBits64(src, bit, bit);
};

var extendSigned3To8 = function(src) {
    var isNeg = (src & (1 << 2)) != 0;
    var val = isNeg ? src - 8 : src;
    return val;
};

var extend4To8 = function(src) {
    return src * 255 / 15;
};

var extend5To8 = function(src) {
    return src * 255 / 31;
};

var extend6To8 = function(src) {
    return src * 255 / 63;
};

var extend7To8 = function(src) {
    return src * 255 / 127;
};

var extend11To16 = function(src) {
    return src * 32.015144;
};

var extend11To16WithSign = function(src) {
    if (src < 0)
        return -extend11To16(-src);
    else
        return extend11To16(src);
};

/**
 * @param { (Uint16Array|Int16Array) } dst
 * @param {Uint8Array} src
 * @param {boolean} signedMode
 */
var decompressEAC11Block = function(dst, src, signedMode) {
    var modifierTable = [
        [-3, -6, -9, -15, 2, 5, 8, 14],
        [-3, -7, -10, -13, 2, 6, 9, 12],
        [-2, -5, -8, -13, 1, 4, 7, 12],
        [-2, -4, -6, -13, 1, 3, 5, 12],
        [-3, -6, -8, -12, 2, 5, 7, 11],
        [-3, -7, -9, -11, 2, 6, 8, 10],
        [-4, -7, -8, -11, 3, 6, 7, 10],
        [-3, -5, -8, -11, 2, 4, 7, 10],
        [-2, -6, -8, -10, 1, 5, 7, 9],
        [-2, -5, -8, -10, 1, 4, 7, 9],
        [-2, -4, -8, -10, 1, 3, 7, 9],
        [-2, -5, -7, -10, 1, 4, 6, 9],
        [-3, -4, -7, -10, 2, 3, 6, 9],
        [-1, -2, -3, -10, 0, 1, 2, 9],
        [-4, -6, -8, -9, 3, 5, 7, 8],
        [-3, -5, -7, -9, 2, 4, 6, 8]
    ];

    var multiplier = getBits64(src, 52, 55);
    var tableNdx = getBits64(src, 48, 51);
    var baseCodeword = getBits64(src, 56, 63);

    if (signedMode) {
        if (baseCodeword > 127)
            baseCodeword -= 256;
        if (baseCodeword == -128)
            baseCodeword = -127;
    }

    var pixelNdx = 0;
    for (var x = 0; x < ETC2_BLOCK_WIDTH; x++) {
        for (var y = 0; y < ETC2_BLOCK_HEIGHT; y++) {
             var dstOffset = (y * ETC2_BLOCK_WIDTH + x);
             var pixelBitNdx = 45 - 3 * pixelNdx;
             var modifierNdx = (getBit64(src, pixelBitNdx + 2) << 2) | (getBit64(src, pixelBitNdx + 1) << 1) | getBit64(src, pixelBitNdx);
             var modifier = modifierTable[tableNdx][modifierNdx];

            if (signedMode) {
                if (multiplier != 0)
                    dst[dstOffset] = deMath.clamp(baseCodeword * 8 + multiplier * modifier * 8, -1023, 1023);
                else
                    dst[dstOffset] = deMath.clamp(baseCodeword * 8 + modifier, -1023, 1023);
            } else {
                if (multiplier != 0)
                    dst[dstOffset] = deMath.clamp(baseCodeword * 8 + 4 + multiplier * modifier * 8, 0, 2047);
                else
                    dst[dstOffset] = deMath.clamp(baseCodeword * 8 + 4 + modifier, 0, 2047);
            }
            pixelNdx++;
        }
    }
};

var decompressEAC_R11 = function(/*const tcu::PixelBufferAccess&*/ dst, width, height, src, signedMode) {
    /** @const */ var numBlocksX = tcuCompressedTexture.divRoundUp(width, 4);
    /** @const */ var numBlocksY = tcuCompressedTexture.divRoundUp(height, 4);
    var dstPtr;
    var dstRowPitch = dst.getRowPitch();
    var dstPixelSize = ETC2_UNCOMPRESSED_PIXEL_SIZE_R11;
    var uncompressedBlockArray = new ArrayBuffer(ETC2_UNCOMPRESSED_BLOCK_SIZE_R11);
    var uncompressedBlock16;
    if (signedMode) {
        dstPtr = new Int16Array(dst.m_data);
        uncompressedBlock16 = new Int16Array(uncompressedBlockArray);
    } else {
        dstPtr = new Uint16Array(dst.m_data);
        uncompressedBlock16 = new Uint16Array(uncompressedBlockArray);
    }

    for (var blockY = 0; blockY < numBlocksY; blockY++) {
        for (var blockX = 0; blockX < numBlocksX; blockX++) {
            /*const deUint64*/ var compressedBlock = get64BitBlock(src, blockY * numBlocksX + blockX);

            // Decompress.
            decompressEAC11Block(uncompressedBlock16, compressedBlock, signedMode);

            // Write to dst.
            var baseX = blockX * ETC2_BLOCK_WIDTH;
            var baseY = blockY * ETC2_BLOCK_HEIGHT;
            for (var y = 0; y < Math.min(ETC2_BLOCK_HEIGHT, height - baseY); y++) {
                for (var x = 0; x < Math.min(ETC2_BLOCK_WIDTH, width - baseX); x++) {
                    DE_ASSERT(ETC2_UNCOMPRESSED_PIXEL_SIZE_R11 == 2);

                    if (signedMode) {
                        var srcIndex = y * ETC2_BLOCK_WIDTH + x;
                        var dstIndex = (baseY + y) * dstRowPitch / dstPixelSize + baseX + x;

                        dstPtr[dstIndex] = extend11To16WithSign(uncompressedBlock16[srcIndex]);
                    } else {
                        var srcIndex = y * ETC2_BLOCK_WIDTH + x;
                        var dstIndex = (baseY + y) * dstRowPitch / dstPixelSize + baseX + x;

                        dstPtr[dstIndex] = extend11To16(uncompressedBlock16[srcIndex]);
                    }
                }
            }
        }
    }
};

var decompressEAC_RG11 = function(/*const tcu::PixelBufferAccess&*/ dst, width, height, src, signedMode) {
    /** @const */ var numBlocksX = tcuCompressedTexture.divRoundUp(width, 4);
    /** @const */ var numBlocksY = tcuCompressedTexture.divRoundUp(height, 4);
    var dstPtr;
    var dstRowPitch = dst.getRowPitch();
    var dstPixelSize = ETC2_UNCOMPRESSED_PIXEL_SIZE_RG11;
    var uncompressedBlockArrayR = new ArrayBuffer(ETC2_UNCOMPRESSED_BLOCK_SIZE_R11);
    var uncompressedBlockArrayG = new ArrayBuffer(ETC2_UNCOMPRESSED_BLOCK_SIZE_R11);
    var uncompressedBlockR16;
    var uncompressedBlockG16;
    if (signedMode) {
        dstPtr = new Int16Array(dst.m_data);
        uncompressedBlockR16 = new Int16Array(uncompressedBlockArrayR);
        uncompressedBlockG16 = new Int16Array(uncompressedBlockArrayG);
    } else {
        dstPtr = new Uint16Array(dst.m_data);
        uncompressedBlockR16 = new Uint16Array(uncompressedBlockArrayR);
        uncompressedBlockG16 = new Uint16Array(uncompressedBlockArrayG);
    }

    for (var blockY = 0; blockY < numBlocksY; blockY++) {
        for (var blockX = 0; blockX < numBlocksX; blockX++) {
            /*const deUint64*/ var compressedBlockR = get128BitBlockStart(src, blockY * numBlocksX + blockX);
            /*const deUint64*/ var compressedBlockG = get128BitBlockEnd(src, blockY * numBlocksX + blockX);

            // Decompress.
            decompressEAC11Block(uncompressedBlockR16, compressedBlockR, signedMode);
            decompressEAC11Block(uncompressedBlockG16, compressedBlockG, signedMode);

            // Write to dst.
            var baseX = blockX * ETC2_BLOCK_WIDTH;
            var baseY = blockY * ETC2_BLOCK_HEIGHT;
            for (var y = 0; y < Math.min(ETC2_BLOCK_HEIGHT, height - baseY); y++) {
                for (var x = 0; x < Math.min(ETC2_BLOCK_WIDTH, width - baseX); x++) {
                    DE_ASSERT(ETC2_UNCOMPRESSED_PIXEL_SIZE_RG11 == 4);

                    if (signedMode) {
                        var srcIndex = y * ETC2_BLOCK_WIDTH + x;
                        var dstIndex = 2 * ((baseY + y) * dstRowPitch / dstPixelSize + baseX + x);

                        dstPtr[dstIndex] = extend11To16WithSign(uncompressedBlockR16[srcIndex]);
                        dstPtr[dstIndex + 1] = extend11To16WithSign(uncompressedBlockG16[srcIndex]);
                    } else {
                        var srcIndex = y * ETC2_BLOCK_WIDTH + x;
                        var dstIndex = 2 * ((baseY + y) * dstRowPitch / dstPixelSize + baseX + x);

                        dstPtr[dstIndex] = extend11To16(uncompressedBlockR16[srcIndex]);
                        dstPtr[dstIndex + 1] = extend11To16(uncompressedBlockG16[srcIndex]);
                    }
                }
            }
        }
    }
};

// if alphaMode is true, do PUNCHTHROUGH and store alpha to alphaDst; otherwise do ordinary ETC2 RGB8.
/**
 * @param {Uint8Array} dst Destination array
 * @param {Uint8Array} src Source array
 * @param {Uint8Array} alphaDst Optional Alpha output channel
 */
var decompressETC2Block = function(dst, src, alphaDst, alphaMode) {
    /**
     * enum
     */
    var Etc2Mode = {
        MODE_INDIVIDUAL: 0,
        MODE_DIFFERENTIAL: 1,
        MODE_T: 2,
        MODE_H: 3,
        MODE_PLANAR: 4
    };

    var diffOpaqueBit = getBit64(src, 33);
    var selBR = getBits64(src, 59, 63); // 5 bits.
    var selBG = getBits64(src, 51, 55);
    var selBB = getBits64(src, 43, 47);
    var selDR = extendSigned3To8(getBits64(src, 56, 58)); // 3 bits.
    var selDG = extendSigned3To8(getBits64(src, 48, 50));
    var selDB = extendSigned3To8(getBits64(src, 40, 42));

    var mode;

    if (!alphaMode && diffOpaqueBit == 0)
        mode = Etc2Mode.MODE_INDIVIDUAL;
    else if (!deMath.deInRange32(selBR + selDR, 0, 31))
        mode = Etc2Mode.MODE_T;
    else if (!deMath.deInRange32(selBG + selDG, 0, 31))
        mode = Etc2Mode.MODE_H;
    else if (!deMath.deInRange32(selBB + selDB, 0, 31))
        mode = Etc2Mode.MODE_PLANAR;
    else
        mode = Etc2Mode.MODE_DIFFERENTIAL;

    if (mode == Etc2Mode.MODE_INDIVIDUAL || mode == Etc2Mode.MODE_DIFFERENTIAL) {
        // Individual and differential modes have some steps in common, handle them here.
        var modifierTable = [
        //      00 01 10 11
            [2, 8, -2, -8],
            [5, 17, -5, -17],
            [9, 29, -9, -29],
            [13, 42, -13, -42],
            [18, 60, -18, -60],
            [24, 80, -24, -80],
            [33, 106, -33, -106],
            [47, 183, -47, -183]
        ];

         var flipBit = getBit64(src, 32);
         var table = [getBits64(src, 37, 39), getBits64(src, 34, 36)];
        var baseR = [];
        var baseG = [];
        var baseB = [];

        if (mode == Etc2Mode.MODE_INDIVIDUAL) {
            // Individual mode, initial values.
            baseR[0] = extend4To8(getBits64(src, 60, 63));
            baseR[1] = extend4To8(getBits64(src, 56, 59));
            baseG[0] = extend4To8(getBits64(src, 52, 55));
            baseG[1] = extend4To8(getBits64(src, 48, 51));
            baseB[0] = extend4To8(getBits64(src, 44, 47));
            baseB[1] = extend4To8(getBits64(src, 40, 43));
        } else {
            // Differential mode, initial values.
            baseR[0] = extend5To8(selBR);
            baseG[0] = extend5To8(selBG);
            baseB[0] = extend5To8(selBB);

            baseR[1] = extend5To8((selBR + selDR));
            baseG[1] = extend5To8((selBG + selDG));
            baseB[1] = extend5To8((selBB + selDB));
        }

        // Write final pixels for individual or differential mode.
        var pixelNdx = 0;
        for (var x = 0; x < ETC2_BLOCK_WIDTH; x++) {
            for (var y = 0; y < ETC2_BLOCK_HEIGHT; y++, pixelNdx++) {
                 var dstOffset = (y * ETC2_BLOCK_WIDTH + x) * ETC2_UNCOMPRESSED_PIXEL_SIZE_RGB8;
                 var subBlock = ((flipBit ? y : x) >= 2) ? 1 : 0;
                 var tableNdx = table[subBlock];
                 var modifierNdx = (getBit64(src, 16 + pixelNdx) << 1) | getBit64(src, pixelNdx);
                 var alphaDstOffset = (y * ETC2_BLOCK_WIDTH + x) * ETC2_UNCOMPRESSED_PIXEL_SIZE_A8; // Only needed for PUNCHTHROUGH version.

                // If doing PUNCHTHROUGH version (alphaMode), opaque bit may affect colors.
                if (alphaMode && diffOpaqueBit == 0 && modifierNdx == 2) {
                    dst[dstOffset + 0] = 0;
                    dst[dstOffset + 1] = 0;
                    dst[dstOffset + 2] = 0;
                    alphaDst[alphaDstOffset] = 0;
                } else {
                    var modifier;

                    // PUNCHTHROUGH version and opaque bit may also affect modifiers.
                    if (alphaMode && diffOpaqueBit == 0 && (modifierNdx == 0 || modifierNdx == 2))
                        modifier = 0;
                    else
                        modifier = modifierTable[tableNdx][modifierNdx];

                    dst[dstOffset + 0] = deMath.clamp(baseR[subBlock] + modifier, 0, 255);
                    dst[dstOffset + 1] = deMath.clamp(baseG[subBlock] + modifier, 0, 255);
                    dst[dstOffset + 2] = deMath.clamp(baseB[subBlock] + modifier, 0, 255);

                    if (alphaMode)
                        alphaDst[alphaDstOffset] = 255;
                }
            }
        }
    } else if (mode == Etc2Mode.MODE_T || mode == Etc2Mode.MODE_H) {
        // T and H modes have some steps in common, handle them here.
        var distTable = [3, 6, 11, 16, 23, 32, 41, 64];

        var paintR = [];
        var paintG = [];
        var paintB = [];

        if (mode == Etc2Mode.MODE_T) {
            // T mode, calculate paint values.
             var R1a = getBits64(src, 59, 60);
             var R1b = getBits64(src, 56, 57);
             var G1 = getBits64(src, 52, 55);
             var B1 = getBits64(src, 48, 51);
             var R2 = getBits64(src, 44, 47);
             var G2 = getBits64(src, 40, 43);
             var B2 = getBits64(src, 36, 39);
             var distNdx = (getBits64(src, 34, 35) << 1) | getBit64(src, 32);
             var dist = distTable[distNdx];

            paintR[0] = extend4To8((R1a << 2) | R1b);
            paintG[0] = extend4To8(G1);
            paintB[0] = extend4To8(B1);
            paintR[2] = extend4To8(R2);
            paintG[2] = extend4To8(G2);
            paintB[2] = extend4To8(B2);
            paintR[1] = deMath.clamp(paintR[2] + dist, 0, 255);
            paintG[1] = deMath.clamp(paintG[2] + dist, 0, 255);
            paintB[1] = deMath.clamp(paintB[2] + dist, 0, 255);
            paintR[3] = deMath.clamp(paintR[2] - dist, 0, 255);
            paintG[3] = deMath.clamp(paintG[2] - dist, 0, 255);
            paintB[3] = deMath.clamp(paintB[2] - dist, 0, 255);
        } else {
            // H mode, calculate paint values.
            var R1 = getBits64(src, 59, 62);
            var G1a = getBits64(src, 56, 58);
            var G1b = getBit64(src, 52);
            var B1a = getBit64(src, 51);
            var B1b = getBits64(src, 47, 49);
            var R2 = getBits64(src, 43, 46);
            var G2 = getBits64(src, 39, 42);
            var B2 = getBits64(src, 35, 38);
            var baseR = [];
            var baseG = [];
            var baseB = [];
            var baseValue = [];
            var distNdx;
            var dist;

            baseR[0] = extend4To8(R1);
            baseG[0] = extend4To8((G1a << 1) | G1b);
            baseB[0] = extend4To8((B1a << 3) | B1b);
            baseR[1] = extend4To8(R2);
            baseG[1] = extend4To8(G2);
            baseB[1] = extend4To8(B2);
            baseValue[0] = ((baseR[0]) << 16) | ((baseG[0]) << 8) | baseB[0];
            baseValue[1] = ((baseR[1]) << 16) | ((baseG[1]) << 8) | baseB[1];
            distNdx = (getBit64(src, 34) << 2) | (getBit64(src, 32) << 1);
            if (baseValue[0] >= baseValue[1])
                distNdx += 1;
            dist = distTable[distNdx];

            paintR[0] = deMath.clamp(baseR[0] + dist, 0, 255);
            paintG[0] = deMath.clamp(baseG[0] + dist, 0, 255);
            paintB[0] = deMath.clamp(baseB[0] + dist, 0, 255);
            paintR[1] = deMath.clamp(baseR[0] - dist, 0, 255);
            paintG[1] = deMath.clamp(baseG[0] - dist, 0, 255);
            paintB[1] = deMath.clamp(baseB[0] - dist, 0, 255);
            paintR[2] = deMath.clamp(baseR[1] + dist, 0, 255);
            paintG[2] = deMath.clamp(baseG[1] + dist, 0, 255);
            paintB[2] = deMath.clamp(baseB[1] + dist, 0, 255);
            paintR[3] = deMath.clamp(baseR[1] - dist, 0, 255);
            paintG[3] = deMath.clamp(baseG[1] - dist, 0, 255);
            paintB[3] = deMath.clamp(baseB[1] - dist, 0, 255);
        }

        // Write final pixels for T or H mode.
        var pixelNdx = 0;
        for (var x = 0; x < ETC2_BLOCK_WIDTH; x++) {
            for (var y = 0; y < ETC2_BLOCK_HEIGHT; y++, pixelNdx++) {
                 var dstOffset = (y * ETC2_BLOCK_WIDTH + x) * ETC2_UNCOMPRESSED_PIXEL_SIZE_RGB8;
                 var paintNdx = (getBit64(src, 16 + pixelNdx) << 1) | getBit64(src, pixelNdx);
                 var alphaDstOffset = (y * ETC2_BLOCK_WIDTH + x) * ETC2_UNCOMPRESSED_PIXEL_SIZE_A8; // Only needed for PUNCHTHROUGH version.

                if (alphaMode && diffOpaqueBit == 0 && paintNdx == 2) {
                    dst[dstOffset + 0] = 0;
                    dst[dstOffset + 1] = 0;
                    dst[dstOffset + 2] = 0;
                    alphaDst[alphaDstOffset] = 0;
                } else {
                    dst[dstOffset + 0] = deMath.clamp(paintR[paintNdx], 0, 255);
                    dst[dstOffset + 1] = deMath.clamp(paintG[paintNdx], 0, 255);
                    dst[dstOffset + 2] = deMath.clamp(paintB[paintNdx], 0, 255);

                    if (alphaMode)
                        alphaDst[alphaDstOffset] = 255;
                }
            }
        }
    } else {
        // Planar mode.
        var GO1 = getBit64(src, 56);
        var GO2 = getBits64(src, 49, 54);
        var BO1 = getBit64(src, 48);
        var BO2 = getBits64(src, 43, 44);
        var BO3 = getBits64(src, 39, 41);
        var RH1 = getBits64(src, 34, 38);
        var RH2 = getBit64(src, 32);
        var RO = extend6To8(getBits64(src, 57, 62));
        var GO = extend7To8((GO1 << 6) | GO2);
        var BO = extend6To8((BO1 << 5) | (BO2 << 3) | BO3);
        var RH = extend6To8((RH1 << 1) | RH2);
        var GH = extend7To8(getBits64(src, 25, 31));
        var BH = extend6To8(getBits64(src, 19, 24));
        var RV = extend6To8(getBits64(src, 13, 18));
        var GV = extend7To8(getBits64(src, 6, 12));
        var BV = extend6To8(getBits64(src, 0, 5));

        // Write final pixels for planar mode.
        for (var y = 0; y < 4; y++) {
            for (var x = 0; x < 4; x++) {
                 var dstOffset = (y * ETC2_BLOCK_WIDTH + x) * ETC2_UNCOMPRESSED_PIXEL_SIZE_RGB8;
                 var unclampedR = (x * (RH - RO) + y * (RV - RO) + 4 * RO + 2) / 4;
                 var unclampedG = (x * (GH - GO) + y * (GV - GO) + 4 * GO + 2) / 4;
                 var unclampedB = (x * (BH - BO) + y * (BV - BO) + 4 * BO + 2) / 4;
                 var alphaDstOffset = (y * ETC2_BLOCK_WIDTH + x) * ETC2_UNCOMPRESSED_PIXEL_SIZE_A8; // Only needed for PUNCHTHROUGH version.

                dst[dstOffset + 0] = deMath.clamp(unclampedR, 0, 255);
                dst[dstOffset + 1] = deMath.clamp(unclampedG, 0, 255);
                dst[dstOffset + 2] = deMath.clamp(unclampedB, 0, 255);

                if (alphaMode)
                    alphaDst[alphaDstOffset] = 255;
            }
        }
    }
};

var decompressEAC8Block = function(dst, src) {
    var modifierTable = [
        [-3, -6, -9, -15, 2, 5, 8, 14],
        [-3, -7, -10, -13, 2, 6, 9, 12],
        [-2, -5, -8, -13, 1, 4, 7, 12],
        [-2, -4, -6, -13, 1, 3, 5, 12],
        [-3, -6, -8, -12, 2, 5, 7, 11],
        [-3, -7, -9, -11, 2, 6, 8, 10],
        [-4, -7, -8, -11, 3, 6, 7, 10],
        [-3, -5, -8, -11, 2, 4, 7, 10],
        [-2, -6, -8, -10, 1, 5, 7, 9],
        [-2, -5, -8, -10, 1, 4, 7, 9],
        [-2, -4, -8, -10, 1, 3, 7, 9],
        [-2, -5, -7, -10, 1, 4, 6, 9],
        [-3, -4, -7, -10, 2, 3, 6, 9],
        [-1, -2, -3, -10, 0, 1, 2, 9],
        [-4, -6, -8, -9, 3, 5, 7, 8],
        [-3, -5, -7, -9, 2, 4, 6, 8]
    ];

    var baseCodeword = getBits64(src, 56, 63);
    var multiplier = getBits64(src, 52, 55);
    var tableNdx = getBits64(src, 48, 51);

    var pixelNdx = 0;
    for (var x = 0; x < ETC2_BLOCK_WIDTH; x++) {
        for (var y = 0; y < ETC2_BLOCK_HEIGHT; y++, pixelNdx++) {
            var dstOffset = (y * ETC2_BLOCK_WIDTH + x);
            var pixelBitNdx = 45 - 3 * pixelNdx;
            var modifierNdx = (getBit64(src, pixelBitNdx + 2) << 2) | (getBit64(src, pixelBitNdx + 1) << 1) | getBit64(src, pixelBitNdx);
            var modifier = modifierTable[tableNdx][modifierNdx];

            dst[dstOffset] = deMath.clamp(baseCodeword + multiplier * modifier, 0, 255);
        }
    }
};

var decompressETC2 = function(/*const tcu::PixelBufferAccess&*/ dst, width, height, src) {
    var numBlocksX = tcuCompressedTexture.divRoundUp(width, 4);
    var numBlocksY = tcuCompressedTexture.divRoundUp(height, 4);
    var dstPtr = new Uint8Array(dst.m_data);
    var dstRowPitch = dst.getRowPitch();
    var dstPixelSize = ETC2_UNCOMPRESSED_PIXEL_SIZE_RGB8;
    var uncompressedBlockArray = new ArrayBuffer(ETC2_UNCOMPRESSED_BLOCK_SIZE_RGB8);
    var uncompressedBlock = new Uint8Array(uncompressedBlockArray);

    for (var blockY = 0; blockY < numBlocksY; blockY++) {
        for (var blockX = 0; blockX < numBlocksX; blockX++) {
            var compressedBlock = get64BitBlock(src, blockY * numBlocksX + blockX);

            // Decompress.
            decompressETC2Block(uncompressedBlock, compressedBlock, null, false);

            // Write to dst.
            var baseX = blockX * ETC2_BLOCK_WIDTH;
            var baseY = blockY * ETC2_BLOCK_HEIGHT;
            for (var y = 0; y < Math.min(ETC2_BLOCK_HEIGHT, height - baseY); y++) {
                for (var x = 0; x < Math.min(ETC2_BLOCK_WIDTH, width - baseX); x++) {
                    var srcIndex = (y * ETC2_BLOCK_WIDTH + x) * ETC2_UNCOMPRESSED_PIXEL_SIZE_RGB8;
                    var dstIndex = (baseY + y) * dstRowPitch + (baseX + x) * dstPixelSize;

                    for (var i = 0; i < ETC2_UNCOMPRESSED_PIXEL_SIZE_RGB8; i++)
                        dstPtr[dstIndex + i] = uncompressedBlock[srcIndex + i];
                }
            }
        }
    }
};

var decompressETC2_EAC_RGBA8 = function(/*const tcu::PixelBufferAccess&*/ dst, width, height, src) {
    var numBlocksX = tcuCompressedTexture.divRoundUp(width, 4);
    var numBlocksY = tcuCompressedTexture.divRoundUp(height, 4);
    var dstPtr = new Uint8Array(dst.m_data);
    var dstRowPitch = dst.getRowPitch();
    var dstPixelSize = ETC2_UNCOMPRESSED_PIXEL_SIZE_RGBA8;
    var uncompressedBlockArray = new ArrayBuffer(ETC2_UNCOMPRESSED_BLOCK_SIZE_RGB8);
    var uncompressedBlock = new Uint8Array(uncompressedBlockArray);
    var uncompressedBlockAlphaArray = new ArrayBuffer(ETC2_UNCOMPRESSED_BLOCK_SIZE_A8);
    var uncompressedBlockAlpha = new Uint8Array(uncompressedBlockAlphaArray);

    for (var blockY = 0; blockY < numBlocksY; blockY++) {
        for (var blockX = 0; blockX < numBlocksX; blockX++) {
            var compressedBlockAlpha = get128BitBlockStart(src, blockY * numBlocksX + blockX);
            var compressedBlockRGB = get128BitBlockEnd(src, blockY * numBlocksX + blockX);

            // Decompress.
            decompressETC2Block(uncompressedBlock, compressedBlockRGB, null, false);
            decompressEAC8Block(uncompressedBlockAlpha, compressedBlockAlpha);

            // Write to dst.
            var baseX = blockX * ETC2_BLOCK_WIDTH;
            var baseY = blockY * ETC2_BLOCK_HEIGHT;
            for (var y = 0; y < Math.min(ETC2_BLOCK_HEIGHT, height - baseY); y++) {
                for (var x = 0; x < Math.min(ETC2_BLOCK_WIDTH, width - baseX); x++) {
                    var srcIndex = (y * ETC2_BLOCK_WIDTH + x) * ETC2_UNCOMPRESSED_PIXEL_SIZE_RGB8;
                    var srcAlphaIndex = (y * ETC2_BLOCK_WIDTH + x) * ETC2_UNCOMPRESSED_PIXEL_SIZE_A8;
                    var dstIndex = (baseY + y) * dstRowPitch + (baseX + x) * dstPixelSize;

                    for (var i = 0; i < ETC2_UNCOMPRESSED_PIXEL_SIZE_RGBA8 - 1; i++)
                        dstPtr[dstIndex + i] = uncompressedBlock[srcIndex + i];
                    dstPtr[dstIndex + ETC2_UNCOMPRESSED_PIXEL_SIZE_RGBA8 - 1] = uncompressedBlockAlpha[srcAlphaIndex];

                }
            }
        }
    }
};

var decompressETC2_RGB8_PUNCHTHROUGH_ALPHA1 = function(/*const tcu::PixelBufferAccess&*/ dst, width, height, src) {
    var numBlocksX = tcuCompressedTexture.divRoundUp(width, 4);
    var numBlocksY = tcuCompressedTexture.divRoundUp(height, 4);
    var dstPtr = new Uint8Array(dst.m_data);
    var dstRowPitch = dst.getRowPitch();
    var dstPixelSize = ETC2_UNCOMPRESSED_PIXEL_SIZE_RGBA8;
    var uncompressedBlockArray = new ArrayBuffer(ETC2_UNCOMPRESSED_BLOCK_SIZE_RGB8);
    var uncompressedBlock = new Uint8Array(uncompressedBlockArray);
    var uncompressedBlockAlphaArray = new ArrayBuffer(ETC2_UNCOMPRESSED_BLOCK_SIZE_A8);
    var uncompressedBlockAlpha = new Uint8Array(uncompressedBlockAlphaArray);

    for (var blockY = 0; blockY < numBlocksY; blockY++) {
        for (var blockX = 0; blockX < numBlocksX; blockX++) {
            var compressedBlock = get64BitBlock(src, blockY * numBlocksX + blockX);

            // Decompress.
            decompressETC2Block(uncompressedBlock, compressedBlock, uncompressedBlockAlpha, true);

            // Write to dst.
            var baseX = blockX * ETC2_BLOCK_WIDTH;
            var baseY = blockY * ETC2_BLOCK_HEIGHT;
            for (var y = 0; y < Math.min(ETC2_BLOCK_HEIGHT, height - baseY); y++) {
                for (var x = 0; x < Math.min(ETC2_BLOCK_WIDTH, width - baseX); x++) {
                    var srcIndex = (y * ETC2_BLOCK_WIDTH + x) * ETC2_UNCOMPRESSED_PIXEL_SIZE_RGB8;
                    var srcAlphaIndex = (y * ETC2_BLOCK_WIDTH + x) * ETC2_UNCOMPRESSED_PIXEL_SIZE_A8;
                    var dstIndex = (baseY + y) * dstRowPitch + (baseX + x) * dstPixelSize;

                    for (var i = 0; i < ETC2_UNCOMPRESSED_PIXEL_SIZE_RGBA8 - 1; i++)
                        dstPtr[dstIndex + i] = uncompressedBlock[srcIndex + i];
                    dstPtr[dstIndex + ETC2_UNCOMPRESSED_PIXEL_SIZE_RGBA8 - 1] = uncompressedBlockAlpha[srcAlphaIndex];

                }
            }
        }
    }
};

return {
    decompressEAC_R11: decompressEAC_R11,
    decompressEAC_RG11: decompressEAC_RG11,
    decompressETC2: decompressETC2,
    decompressETC2_RGB8_PUNCHTHROUGH_ALPHA1: decompressETC2_RGB8_PUNCHTHROUGH_ALPHA1,
    decompressETC2_EAC_RGBA8: decompressETC2_EAC_RGBA8
};

}();

/**
 * @constructor
 * @param {tcuCompressedTexture.Format} format
 * @param {number} width
 * @param {number} height
 * @param {number=} depth
 */
tcuCompressedTexture.CompressedTexture = function(format, width, height, depth) {
    depth = depth === undefined ? 1 : depth;
    this.setStorage(format, width, height, depth);
    /** @type {Uint8Array} */ this.m_data;
};

/**
 * @return {number}
 */
tcuCompressedTexture.CompressedTexture.prototype.getDataSize = function() {
     return this.m_data.length;
};

/**
  * @return {Uint8Array}
  */
tcuCompressedTexture.CompressedTexture.prototype.getData = function() {
      return this.m_data;
};

/**
  * @return {number}
  */
tcuCompressedTexture.CompressedTexture.prototype.getWidth = function() {
      return this.m_width;
};

/**
  * @return {number}
  */
tcuCompressedTexture.CompressedTexture.prototype.getHeight = function() {
      return this.m_height;
};

/**
  * @return {tcuCompressedTexture.Format}
  */
tcuCompressedTexture.CompressedTexture.prototype.getFormat = function() {
      return this.m_format;
};

tcuCompressedTexture.CompressedTexture.prototype.setStorage = function(format, width, height, depth) {
    depth = depth === undefined ? 1 : depth;
    this.m_format = format;
    this.m_width = width;
    this.m_height = height;
    this.m_depth = depth;

    if (tcuCompressedTexture.isEtcFormat(this.m_format)) {
        DE_ASSERT(this.m_depth == 1);

        var blockSizeMultiplier = 0; // How many 64-bit parts each compressed block contains.

        switch (this.m_format) {
            case tcuCompressedTexture.Format.ETC1_RGB8: blockSizeMultiplier = 1; break;
            case tcuCompressedTexture.Format.EAC_R11: blockSizeMultiplier = 1; break;
            case tcuCompressedTexture.Format.EAC_SIGNED_R11: blockSizeMultiplier = 1; break;
            case tcuCompressedTexture.Format.EAC_RG11: blockSizeMultiplier = 2; break;
            case tcuCompressedTexture.Format.EAC_SIGNED_RG11: blockSizeMultiplier = 2; break;
            case tcuCompressedTexture.Format.ETC2_RGB8: blockSizeMultiplier = 1; break;
            case tcuCompressedTexture.Format.ETC2_SRGB8: blockSizeMultiplier = 1; break;
            case tcuCompressedTexture.Format.ETC2_RGB8_PUNCHTHROUGH_ALPHA1: blockSizeMultiplier = 1; break;
            case tcuCompressedTexture.Format.ETC2_SRGB8_PUNCHTHROUGH_ALPHA1: blockSizeMultiplier = 1; break;
            case tcuCompressedTexture.Format.ETC2_EAC_RGBA8: blockSizeMultiplier = 2; break;
            case tcuCompressedTexture.Format.ETC2_EAC_SRGB8_ALPHA8: blockSizeMultiplier = 2; break;

            default:
                throw new Error('Unsupported format ' + format);
                break;
        }

        this.m_array = new ArrayBuffer(blockSizeMultiplier * 8 * tcuCompressedTexture.divRoundUp(this.m_width, 4) * tcuCompressedTexture.divRoundUp(this.m_height, 4));
        this.m_data = new Uint8Array(this.m_array);
    }
    // else if (isASTCFormat(this.m_format))
    // {
    //     if (this.m_depth > 1)
    //         throw tcu::InternalError("3D ASTC textures not currently supported");

    //     const IVec3 blockSize = getASTCBlockSize(this.m_format);
    //     this.m_data.resize(ASTC_BLOCK_SIZE_BYTES * tcuCompressedTexture.divRoundUp(this.m_width, blockSize[0]) * tcuCompressedTexture.divRoundUp(this.m_height, blockSize[1]) * tcuCompressedTexture.divRoundUp(this.m_depth, blockSize[2]));
    // }
    // else
    // {
    //     DE_ASSERT(this.m_format == FORMAT_LAST);
    //     DE_ASSERT(this.m_width == 0 && this.m_height == 0 && this.m_depth == 0);
    //     this.m_data.resize(0);
    // }
};

/*--------------------------------------------------------------------*//*!
 * \brief Get uncompressed texture format
 *//*--------------------------------------------------------------------*/
tcuCompressedTexture.CompressedTexture.prototype.getUncompressedFormat = function() {
    if (tcuCompressedTexture.isEtcFormat(this.m_format)) {
        switch (this.m_format) {
            case tcuCompressedTexture.Format.ETC1_RGB8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNORM_INT8);
            case tcuCompressedTexture.Format.EAC_R11: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.UNORM_INT16);
            case tcuCompressedTexture.Format.EAC_SIGNED_R11: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.SNORM_INT16);
            case tcuCompressedTexture.Format.EAC_RG11: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.UNORM_INT16);
            case tcuCompressedTexture.Format.EAC_SIGNED_RG11: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.SNORM_INT16);
            case tcuCompressedTexture.Format.ETC2_RGB8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNORM_INT8);
            case tcuCompressedTexture.Format.ETC2_SRGB8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.sRGB, tcuTexture.ChannelType.UNORM_INT8);
            case tcuCompressedTexture.Format.ETC2_RGB8_PUNCHTHROUGH_ALPHA1: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8);
            case tcuCompressedTexture.Format.ETC2_SRGB8_PUNCHTHROUGH_ALPHA1: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.sRGBA, tcuTexture.ChannelType.UNORM_INT8);
            case tcuCompressedTexture.Format.ETC2_EAC_RGBA8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8);
            case tcuCompressedTexture.Format.ETC2_EAC_SRGB8_ALPHA8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.sRGBA, tcuTexture.ChannelType.UNORM_INT8);
            default:
                throw new Error('Unsupported format ' + this.m_format);
        }
    }
    // else if (isASTCFormat(m_format))
    // {
    //     if (isASTCSRGBFormat(m_format))
    //         return TextureFormat(tcuTexture.ChannelType.sRGBA, tcuTexture.ChannelType.UNORM_INT8);
    //     else
    //         return TextureFormat(tcuTexture.ChannelType.RGBA, tcuTexture.ChannelType.HALF_FLOAT);
    // }
    // else
    // {
    //     DE_ASSERT(false);
    //     return TextureFormat();
    // }
};

/**
 * Decode to uncompressed pixel data
 * @param {tcuTexture.PixelBufferAccess} dst Destination buffer
 */
tcuCompressedTexture.CompressedTexture.prototype.decompress = function(dst) {
    DE_ASSERT(dst.getWidth() == this.m_width && dst.getHeight() == this.m_height && dst.getDepth() == 1);
    var format = this.getUncompressedFormat();
    if (dst.getFormat().order != format.order || dst.getFormat().type != format.type)
        throw new Error('Formats do not match.');

    if (tcuCompressedTexture.isEtcFormat(this.m_format)) {
        switch (this.m_format) {
            // case tcuCompressedTexture.Format.ETC1_RGB8: decompressETC1 (dst, this.m_width, this.m_height, this.m_data); break;
            case tcuCompressedTexture.Format.EAC_R11: tcuCompressedTexture.etcDecompressInternal.decompressEAC_R11(dst, this.m_width, this.m_height, this.m_array, false); break;
            case tcuCompressedTexture.Format.EAC_SIGNED_R11: tcuCompressedTexture.etcDecompressInternal.decompressEAC_R11(dst, this.m_width, this.m_height, this.m_array, true); break;
            case tcuCompressedTexture.Format.EAC_RG11: tcuCompressedTexture.etcDecompressInternal.decompressEAC_RG11(dst, this.m_width, this.m_height, this.m_array, false); break;
            case tcuCompressedTexture.Format.EAC_SIGNED_RG11: tcuCompressedTexture.etcDecompressInternal.decompressEAC_RG11(dst, this.m_width, this.m_height, this.m_array, true); break;
            case tcuCompressedTexture.Format.ETC2_RGB8: tcuCompressedTexture.etcDecompressInternal.decompressETC2(dst, this.m_width, this.m_height, this.m_array); break;
            case tcuCompressedTexture.Format.ETC2_SRGB8: tcuCompressedTexture.etcDecompressInternal.decompressETC2(dst, this.m_width, this.m_height, this.m_array); break;
            case tcuCompressedTexture.Format.ETC2_RGB8_PUNCHTHROUGH_ALPHA1: tcuCompressedTexture.etcDecompressInternal.decompressETC2_RGB8_PUNCHTHROUGH_ALPHA1(dst, this.m_width, this.m_height, this.m_array); break;
            case tcuCompressedTexture.Format.ETC2_SRGB8_PUNCHTHROUGH_ALPHA1: tcuCompressedTexture.etcDecompressInternal.decompressETC2_RGB8_PUNCHTHROUGH_ALPHA1(dst, this.m_width, this.m_height, this.m_array); break;
            case tcuCompressedTexture.Format.ETC2_EAC_RGBA8: tcuCompressedTexture.etcDecompressInternal.decompressETC2_EAC_RGBA8(dst, this.m_width, this.m_height, this.m_array); break;
            case tcuCompressedTexture.Format.ETC2_EAC_SRGB8_ALPHA8: tcuCompressedTexture.etcDecompressInternal.decompressETC2_EAC_RGBA8(dst, this.m_width, this.m_height, this.m_array); break;

            default:
                throw new Error('Unsupported format ' + this.m_format);
                break;
        }
    }
    // else if (isASTCFormat(m_format))
    // {
    //     const tcu::IVec3 blockSize = getASTCBlockSize(m_format);
    //     const bool isSRGBFormat = isASTCSRGBFormat(m_format);

    //     if (blockSize[2] > 1)
    //         throw tcu::InternalError("3D ASTC textures not currently supported");

    //     decompressASTC(dst, m_width, m_height, &m_data[0], blockSize[0], blockSize[1], isSRGBFormat, isSRGBFormat || params.isASTCModeLDR);
    // } /**/
    else
        throw new Error('Unsupported format ' + this.m_format);
};

 });
