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
goog.provide('framework.common.tcuFloat');
goog.require('framework.delibs.debase.deMath');

goog.scope(function() {

var tcuFloat = framework.common.tcuFloat;
var deMath = framework.delibs.debase.deMath;

var DE_ASSERT = function(x) {
    if (!x)
        throw new Error('Assert failed');
};

tcuFloat.FloatFlags = {
    FLOAT_HAS_SIGN: (1 << 0),
    FLOAT_SUPPORT_DENORM: (1 << 1)
};

/**
 * Defines a tcuFloat.FloatDescription object, which is an essential part of the tcuFloat.deFloat type.
 * Holds the information that shapes the tcuFloat.deFloat.
 * @constructor
 */
tcuFloat.FloatDescription = function(exponentBits, mantissaBits, exponentBias, flags) {
    this.ExponentBits = exponentBits;
    this.MantissaBits = mantissaBits;
    this.ExponentBias = exponentBias;
    this.Flags = flags;

    this.totalBitSize = 1 + this.ExponentBits + this.MantissaBits;
    this.totalByteSize = Math.floor(this.totalBitSize / 8) + ((this.totalBitSize % 8) > 0 ? 1 : 0);
};

/**
 * Builds a zero float of the current binary description.
 * @param {number} sign
 * @return {tcuFloat.deFloat}
 */
tcuFloat.FloatDescription.prototype.zero = function(sign) {
    return tcuFloat.newDeFloatFromParameters(this.zeroNumber(sign), this);
};

tcuFloat.FloatDescription.prototype.zeroNumber = function(sign) {
    return deMath.shiftLeft((sign > 0 ? 0 : 1), (this.ExponentBits + this.MantissaBits));
};

/**
 * Builds an infinity float representation of the current binary description.
 * @param {number} sign
 * @return {tcuFloat.deFloat}
 */
tcuFloat.FloatDescription.prototype.inf = function(sign) {
    return tcuFloat.newDeFloatFromParameters(this.infNumber(sign), this);
};

tcuFloat.FloatDescription.prototype.infNumber = function(sign) {
    return ((sign > 0 ? 0 : 1) << (this.ExponentBits + this.MantissaBits)) |
        deMath.shiftLeft(((1 << this.ExponentBits) - 1), this.MantissaBits); //Unless using very large exponent types, native shift is safe here, i guess.
};

/**
 * Builds a NaN float representation of the current binary description.
 * @return {tcuFloat.deFloat}
 */
tcuFloat.FloatDescription.prototype.nan = function() {
    return tcuFloat.newDeFloatFromParameters(this.nanNumber(), this);
};

tcuFloat.FloatDescription.prototype.nanNumber = function() {
    return deMath.shiftLeft(1, (this.ExponentBits + this.MantissaBits)) - 1;
};

/**
 * Builds a tcuFloat.deFloat number based on the description and the given
 * sign, exponent and mantissa values.
 * @param {number} sign
 * @param {number} exponent
 * @param {number} mantissa
 * @return {tcuFloat.deFloat}
 */
tcuFloat.FloatDescription.prototype.construct = function(sign, exponent, mantissa) {
    // Repurpose this otherwise invalid input as a shorthand notation for zero (no need for caller to care about internal representation)
    /** @type {boolean} */ var isShorthandZero = exponent == 0 && mantissa == 0;

    // Handles the typical notation for zero (min exponent, mantissa 0). Note that the exponent usually used exponent (-ExponentBias) for zero/subnormals is not used.
    // Instead zero/subnormals have the (normally implicit) leading mantissa bit set to zero.

    /** @type {boolean} */ var isDenormOrZero = (exponent == 1 - this.ExponentBias) && (deMath.shiftRight(mantissa, this.MantissaBits) == 0);
    /** @type {number} */ var s = deMath.shiftLeft((sign < 0 ? 1 : 0), (this.ExponentBits + this.MantissaBits));
    /** @type {number} */ var exp = (isShorthandZero || isDenormOrZero) ? 0 : exponent + this.ExponentBias;

    DE_ASSERT(sign == +1 || sign == -1);
    DE_ASSERT(isShorthandZero || isDenormOrZero || deMath.shiftRight(mantissa, this.MantissaBits) == 1);
    DE_ASSERT((exp >> this.ExponentBits) == 0); //Native shift is safe

    return tcuFloat.newDeFloatFromParameters(
        deMath.binaryOp(
            deMath.binaryOp(
                s,
                deMath.shiftLeft(exp, this.MantissaBits),
                deMath.BinaryOp.OR
            ),
            deMath.binaryOp(
                mantissa,
                deMath.shiftLeft(1, this.MantissaBits) - 1,
                deMath.BinaryOp.AND
            ),
            deMath.BinaryOp.OR
        ),
        this
    );
};

/**
 * Builds a tcuFloat.deFloat number based on the description and the given
 * sign, exponent and binary mantissa values.
 * @param {number} sign
 * @param {number} exponent
 * @param {number} mantissaBits The raw binary representation.
 * @return {tcuFloat.deFloat}
 */
tcuFloat.FloatDescription.prototype.constructBits = function(sign, exponent, mantissaBits) {
    /** @type {number} */ var signBit = sign < 0 ? 1 : 0;
    /** @type {number} */ var exponentBits = exponent + this.ExponentBias;

    DE_ASSERT(sign == +1 || sign == -1);
    DE_ASSERT((exponentBits >> this.ExponentBits) == 0);
    DE_ASSERT(deMath.shiftRight(mantissaBits, this.MantissaBits) == 0);

    return tcuFloat.newDeFloatFromParameters(
        deMath.binaryOp(
            deMath.binaryOp(
                deMath.shiftLeft(
                    signBit,
                    this.ExponentBits + this.MantissaBits
                ),
                deMath.shiftLeft(exponentBits, this.MantissaBits),
                deMath.BinaryOp.OR
            ),
            mantissaBits,
            deMath.BinaryOp.OR
        ),
        this
    );
};

/**
 * Converts a tcuFloat.deFloat from it's own format description into the format described
 * by this description.
 * @param {tcuFloat.deFloat} other Other float to convert to this format description.
 * @return {tcuFloat.deFloat} converted tcuFloat.deFloat
 */
tcuFloat.FloatDescription.prototype.convert = function(other) {
    /** @type {number} */ var otherExponentBits = other.description.ExponentBits;
    /** @type {number} */ var otherMantissaBits = other.description.MantissaBits;
    /** @type {number} */ var otherExponentBias = other.description.ExponentBias;
    /** @type {number} */ var otherFlags = other.description.Flags;

    /** @type {number} */ var bitDiff;
    /** @type {number} */ var half;
    /** @type {number} */ var bias;

    if (!(this.Flags & tcuFloat.FloatFlags.FLOAT_HAS_SIGN) && other.sign() < 0) {
        // Negative number, truncate to zero.
        return this.zero(+1);
    } else if (other.isInf()) {
        return this.inf(other.sign());
    } else if (other.isNaN()) {
        return this.nan();
    } else if (other.isZero()) {
        return this.zero(other.sign());
    } else {
        /** @type {number} */ var eMin = 1 - this.ExponentBias;
        /** @type {number} */ var eMax = ((1 << this.ExponentBits) - 2) - this.ExponentBias;

        /** @type {number} */ var s = deMath.shiftLeft(other.signBit(), (this.ExponentBits + this.MantissaBits)); // \note Not sign, but sign bit.
        /** @type {number} */ var e = other.exponent();
        /** @type {number} */ var m = other.mantissa();

        // Normalize denormalized values prior to conversion.
        while (!deMath.binaryOp(m, deMath.shiftLeft(1, otherMantissaBits), deMath.BinaryOp.AND)) {
            m = deMath.shiftLeft(m, 1);
            e -= 1;
        }

        if (e < eMin) {
            // Underflow.
            if ((this.Flags & tcuFloat.FloatFlags.FLOAT_SUPPORT_DENORM) && (eMin - e - 1 <= this.MantissaBits)) {
                // Shift and round (RTE).
                bitDiff = (otherMantissaBits - this.MantissaBits) + (eMin - e);
                half = deMath.shiftLeft(1, (bitDiff - 1)) - 1;
                bias = deMath.binaryOp(deMath.shiftRight(m, bitDiff), 1, deMath.BinaryOp.AND);

                return tcuFloat.newDeFloatFromParameters(
                    deMath.binaryOp(
                        s,
                        deMath.shiftRight(
                            m + half + bias,
                            bitDiff
                        ),
                        deMath.BinaryOp.OR
                    ),
                    this
                );
            } else
                return this.zero(other.sign());
        } else {
            // Remove leading 1.
            m = deMath.binaryOp(m, deMath.binaryNot(deMath.shiftLeft(1, otherMantissaBits)), deMath.BinaryOp.AND);

            if (this.MantissaBits < otherMantissaBits) {
                // Round mantissa (round to nearest even).
                bitDiff = otherMantissaBits - this.MantissaBits;
                half = deMath.shiftLeft(1, (bitDiff - 1)) - 1;
                bias = deMath.binaryOp(deMath.shiftRight(m, bitDiff), 1, deMath.BinaryOp.AND);

                m = deMath.shiftRight(m + half + bias, bitDiff);

                if (deMath.binaryOp(m, deMath.shiftLeft(1, this.MantissaBits), deMath.BinaryOp.AND)) {
                    // Overflow in mantissa.
                    m = 0;
                    e += 1;
                }
            } else {
                bitDiff = this.MantissaBits - otherMantissaBits;
                m = deMath.shiftLeft(m, bitDiff);
            }

            if (e > eMax) {
                // Overflow.
                return this.inf(other.sign());
            } else {
                DE_ASSERT(deMath.deInRange32(e, eMin, eMax));
                DE_ASSERT(deMath.binaryOp((e + this.ExponentBias), deMath.binaryNot(deMath.shiftLeft(1, this.ExponentBits) - 1), deMath.BinaryOp.AND) == 0);
                DE_ASSERT(deMath.binaryOp(m, deMath.binaryNot(deMath.shiftLeft(1, this.MantissaBits) - 1), deMath.BinaryOp.AND) == 0);

                return tcuFloat.newDeFloatFromParameters(
                    deMath.binaryOp(
                        deMath.binaryOp(
                            s,
                            deMath.shiftLeft(
                                e + this.ExponentBias,
                                this.MantissaBits
                            ),
                            deMath.BinaryOp.OR
                        ),
                        m,
                        deMath.BinaryOp.OR
                    ),
                    this
                );
            }
        }
    }
};

/**
 * tcuFloat.deFloat class - Empty constructor, builds a 32 bit float by default
 * @constructor
 */
tcuFloat.deFloat = function() {
    this.description = tcuFloat.description32;

    this.m_buffer = null;
    this.m_array = null;
    this.bitValue = undefined;
    this.signValue = undefined;
    this.expValue = undefined;
    this.mantissaValue = undefined;

    this.m_value = 0;
};

/**
 * buffer - Get the deFloat's existing ArrayBuffer or create one if none exists.
 * @return {ArrayBuffer}
 */
tcuFloat.deFloat.prototype.buffer = function() {
    if (!this.m_buffer)
        this.m_buffer = new ArrayBuffer(this.description.totalByteSize);
    return this.m_buffer;
};

/**
 * array - Get the deFloat's existing Uint8Array or create one if none exists.
 * @return {Uint8Array}
 */
tcuFloat.deFloat.prototype.array = function() {
    if (!this.m_array)
        this.m_array = new Uint8Array(this.buffer());
    return this.m_array;
};

/**
 * deFloatNumber - To be used immediately after constructor
 * Builds a 32-bit tcuFloat.deFloat based on a 64-bit JS number.
 * @param {number} jsnumber
 * @return {tcuFloat.deFloat}
 */
tcuFloat.deFloat.prototype.deFloatNumber = function(jsnumber) {
    var view32 = new DataView(this.buffer());
    view32.setFloat32(0, jsnumber, true); //little-endian
    this.m_value = view32.getFloat32(0, true); //little-endian

    // Clear cached values
    this.bitValue = undefined;
    this.signValue = undefined;
    this.expValue = undefined;
    this.mantissaValue = undefined;

    return this;
};

/**
 * Convenience function to build a 32-bit tcuFloat.deFloat based on a 64-bit JS number
 * Builds a 32-bit tcuFloat.deFloat based on a 64-bit JS number.
 * @param {number} jsnumber
 * @return {tcuFloat.deFloat}
 */
tcuFloat.newDeFloatFromNumber = function(jsnumber) {
    return new tcuFloat.deFloat().deFloatNumber(jsnumber);
};

/**
 * deFloatBuffer - To be used immediately after constructor
 * Builds a tcuFloat.deFloat based on a buffer and a format description.
 * The buffer is assumed to contain data of the given description.
 * @param {ArrayBuffer} buffer
 * @param {tcuFloat.FloatDescription} description
 * @return {tcuFloat.deFloat}
 */
tcuFloat.deFloat.prototype.deFloatBuffer = function(buffer, description) {
    this.m_buffer = buffer;
    this.m_array = new Uint8Array(this.m_buffer);

    this.m_value = deMath.arrayToNumber(this.m_array);

    // Clear cached values
    this.bitValue = undefined;
    this.signValue = undefined;
    this.expValue = undefined;
    this.mantissaValue = undefined;

    return this;
};

/**
 * Convenience function to build a tcuFloat.deFloat based on a buffer and a format description
 * The buffer is assumed to contain data of the given description.
 * @param {ArrayBuffer} buffer
 * @param {tcuFloat.FloatDescription} description
 * @return {tcuFloat.deFloat}
 */
tcuFloat.newDeFloatFromBuffer = function(buffer, description) {
    return new tcuFloat.deFloat().deFloatBuffer(buffer, description);
};

/**
 * Set the tcuFloat.deFloat to the given number.
 * It does not perform any conversion; it assumes the number is compatible with
 * the previously set description.
 * @param {number} jsnumber
 * @return {tcuFloat.deFloat}
 **/
tcuFloat.deFloat.prototype.deFloatParametersNumber = function(jsnumber) {
    this.m_value = jsnumber;
    deMath.numberToArray(this.m_array, jsnumber);

    // Clear cached values
    this.bitValue = undefined;
    this.signValue = undefined;
    this.expValue = undefined;
    this.mantissaValue = undefined;

    return this;
};

/**
 * Initializes a tcuFloat.deFloat from the given number,
 * with the specified format description.
 * It does not perform any conversion; it assumes the number is compatible with
 * the given description.
 * @param {number} jsnumber
 * @param {tcuFloat.FloatDescription} description
 * @return {tcuFloat.deFloat}
 **/
tcuFloat.deFloat.prototype.deFloatParameters = function(jsnumber, description) {
    this.description = description;

    this.m_buffer = new ArrayBuffer(this.description.totalByteSize);
    this.m_array = new Uint8Array(this.m_buffer);

    return this.deFloatParametersNumber(jsnumber);
};

/**
 * Convenience function to create a tcuFloat.deFloat from the given number,
 * with the specified format description.
 * It does not perform any conversion; it assumes the number is compatible with
 * the given description.
 * @param {number} jsnumber
 * @param {tcuFloat.FloatDescription} description
 * @return {tcuFloat.deFloat}
 **/
tcuFloat.newDeFloatFromParameters = function(jsnumber, description) {
    return new tcuFloat.deFloat().deFloatParameters(jsnumber, description);
};

/**
 * Returns bit range [begin, end)
 * @param {number} begin
 * @param {number} end
 * @return {number}
 */
tcuFloat.deFloat.prototype.getBitRange = function(begin, end) {
    return deMath.getBitRange(this.bits(), begin, end);
};

/**
 * Returns the raw binary representation value of the tcuFloat.deFloat
 * @return {number}
 */
tcuFloat.deFloat.prototype.bits = function() {
    if (typeof this.bitValue === 'undefined')
        this.bitValue = deMath.arrayToNumber(this.array());
    return this.bitValue;
};

/**
 * Returns the raw binary sign bit
 * @return {number}
 */
tcuFloat.deFloat.prototype.signBit = function() {
    if (typeof this.signValue === 'undefined')
        this.signValue = this.getBitRange(this.description.totalBitSize - 1, this.description.totalBitSize);
    return this.signValue;
};

/**
 * Returns the raw binary exponent bits
 * @return {number}
 */
tcuFloat.deFloat.prototype.exponentBits = function() {
    if (typeof this.expValue === 'undefined')
        this.expValue = this.getBitRange(this.description.MantissaBits, this.description.MantissaBits + this.description.ExponentBits);
    return this.expValue;
};

/**
 * Returns the raw binary mantissa bits
 * @return {number}
 */
tcuFloat.deFloat.prototype.mantissaBits = function() {
    if (typeof this.mantissaValue === 'undefined')
        this.mantissaValue = this.getBitRange(0, this.description.MantissaBits);
    return this.mantissaValue;
};

/**
 * Returns the sign as a factor (-1 or 1)
 * @return {number}
 */
tcuFloat.deFloat.prototype.sign = function() {
    var sign = this.signBit();
    var signvalue = sign ? -1 : 1;
    return signvalue;
};

/**
 * Returns the real exponent, checking if it's a denorm or zero number or not
 * @return {number}
 */
tcuFloat.deFloat.prototype.exponent = function() {return this.isDenorm() ? 1 - this.description.ExponentBias : this.exponentBits() - this.description.ExponentBias;};

/**
 * Returns the (still raw) mantissa, checking if it's a denorm or zero number or not
 * Makes the normally implicit bit explicit.
 * @return {number}
 */
tcuFloat.deFloat.prototype.mantissa = function() {return this.isZero() || this.isDenorm() ? this.mantissaBits() : deMath.binaryOp(this.mantissaBits(), deMath.shiftLeft(1, this.description.MantissaBits), deMath.BinaryOp.OR);};

/**
 * Returns if the number is infinity or not.
 * @return {boolean}
 */
tcuFloat.deFloat.prototype.isInf = function() {return this.exponentBits() == ((1 << this.description.ExponentBits) - 1) && this.mantissaBits() == 0;};

/**
 * Returns if the number is NaN or not.
 * @return {boolean}
 */
tcuFloat.deFloat.prototype.isNaN = function() {return this.exponentBits() == ((1 << this.description.ExponentBits) - 1) && this.mantissaBits() != 0;};

/**
 * Returns if the number is zero or not.
 * @return {boolean}
 */
tcuFloat.deFloat.prototype.isZero = function() {return this.exponentBits() == 0 && this.mantissaBits() == 0;};

/**
 * Returns if the number is denormalized or not.
 * @return {boolean}
 */
tcuFloat.deFloat.prototype.isDenorm = function() {return this.exponentBits() == 0 && this.mantissaBits() != 0;};

/**
 * Builds a zero float of the current binary description.
 * @param {number} sign
 * @return {tcuFloat.deFloat}
 */
tcuFloat.deFloat.prototype.zero = function(sign) {
    return this.description.zero(sign);
};

/**
 * Builds an infinity float representation of the current binary description.
 * @param {number} sign
 * @return {tcuFloat.deFloat}
 */
tcuFloat.deFloat.prototype.inf = function(sign) {
    return this.description.inf(sign);
};

/**
 * Builds a NaN float representation of the current binary description.
 * @return {tcuFloat.deFloat}
 */
tcuFloat.deFloat.prototype.nan = function() {
    return this.description.nan();
};

/**
 * Builds a float of the current binary description.
 * Given a sign, exponent and mantissa.
 * @param {number} sign
 * @param {number} exponent
 * @param {number} mantissa
 * @return {tcuFloat.deFloat}
 */
tcuFloat.deFloat.prototype.construct = function(sign, exponent, mantissa) {
    return this.description.construct(sign, exponent, mantissa);
};

/**
 * Builds a float of the current binary description.
 * Given a sign, exponent and a raw binary mantissa.
 * @param {number} sign
 * @param {number} exponent
 * @param {number} mantissaBits Raw binary mantissa.
 * @return {tcuFloat.deFloat}
 */
tcuFloat.deFloat.prototype.constructBits = function(sign, exponent, mantissaBits) {
    return this.description.constructBits(sign, exponent, mantissaBits);
};

/**
 * Calculates the JS float number from the internal representation.
 * @return {number} The JS float value represented by this tcuFloat.deFloat.
 */
tcuFloat.deFloat.prototype.getValue = function() {
    if ((this.description.Flags | tcuFloat.FloatFlags.FLOAT_HAS_SIGN) === 0 && this.sign() < 0)
        return 0;
    if (this.isInf())
        return Number.Infinity;
    if (this.isNaN())
        return Number.NaN;
    if (this.isZero())
        return this.sign() * 0;
    /**@type {number} */ var mymantissa = this.mantissa();
    /**@type {number} */ var myexponent = this.exponent();
    /**@type {number} */ var sign = this.sign();

    /**@type {number} */ var value = mymantissa / Math.pow(2, this.description.MantissaBits) * Math.pow(2, myexponent);

    if (this.description.Flags | tcuFloat.FloatFlags.FLOAT_HAS_SIGN != 0)
        value = value * sign;

    return value;
};

tcuFloat.description10 = new tcuFloat.FloatDescription(5, 5, 15, 0);
tcuFloat.description11 = new tcuFloat.FloatDescription(5, 6, 15, 0);
tcuFloat.description16 = new tcuFloat.FloatDescription(5, 10, 15, tcuFloat.FloatFlags.FLOAT_HAS_SIGN);
tcuFloat.description32 = new tcuFloat.FloatDescription(8, 23, 127, tcuFloat.FloatFlags.FLOAT_HAS_SIGN | tcuFloat.FloatFlags.FLOAT_SUPPORT_DENORM);
tcuFloat.description64 = new tcuFloat.FloatDescription(11, 52, 1023, tcuFloat.FloatFlags.FLOAT_HAS_SIGN | tcuFloat.FloatFlags.FLOAT_SUPPORT_DENORM);

tcuFloat.convertFloat32Inline = (function() {
    var float32View = new Float32Array(1);
    var int32View = new Int32Array(float32View.buffer);

    return function(fval, description) {
        float32View[0] = fval;
        var fbits = int32View[0];

        var exponentBits = (fbits >> 23) & 0xff;
        var mantissaBits = fbits & 0x7fffff;
        var signBit = (fbits & 0x80000000) ? 1 : 0;
        var sign = signBit ? -1 : 1;

        var isZero = exponentBits == 0 && mantissaBits == 0;

        var bitDiff;
        var half;
        var bias;

        if (!(description.Flags & tcuFloat.FloatFlags.FLOAT_HAS_SIGN) && sign < 0) {
            // Negative number, truncate to zero.
            return description.zeroNumber(+1);
        } else if (exponentBits == ((1 << tcuFloat.description32.ExponentBits) - 1) && mantissaBits == 0) { // isInf
            return description.infNumber(sign);
        } else if (exponentBits == ((1 << tcuFloat.description32.ExponentBits) - 1) && mantissaBits != 0) { // isNaN
            return description.nanNumber();
        } else if (isZero) {
            return description.zeroNumber(sign);
        } else {
            var eMin = 1 - description.ExponentBias;
            var eMax = ((1 << description.ExponentBits) - 2) - description.ExponentBias;

            var isDenorm = exponentBits == 0 && mantissaBits != 0;

            var s = signBit << (description.ExponentBits + description.MantissaBits); // \note Not sign, but sign bit.
            var e = isDenorm ? 1 - tcuFloat.description32.ExponentBias : exponentBits - tcuFloat.description32.ExponentBias;// other.exponent();
            var m = isZero || isDenorm ? mantissaBits : mantissaBits | (1 << tcuFloat.description32.MantissaBits); // other.mantissa();

            // Normalize denormalized values prior to conversion.
            while (!(m & (1 << tcuFloat.description32.MantissaBits))) {
                m = deMath.shiftLeft(m, 1);
                e -= 1;
            }

            if (e < eMin) {
                // Underflow.
                if ((description.Flags & tcuFloat.FloatFlags.FLOAT_SUPPORT_DENORM) && (eMin - e - 1 <= description.MantissaBits)) {
                    // Shift and round (RTE).
                    bitDiff = (tcuFloat.description32.MantissaBits - description.MantissaBits) + (eMin - e);
                    half = (1 << (bitDiff - 1)) - 1;
                    bias = ((m >> bitDiff) & 1);
                    return (s | ((m + half + bias) >> bitDiff));
                } else
                    return description.zeroNumber(sign);
            } else {
                // Remove leading 1.
                m = (m & ~(1 << tcuFloat.description32.MantissaBits));

                if (description.MantissaBits < tcuFloat.description32.MantissaBits) {
                    // Round mantissa (round to nearest even).
                    bitDiff = tcuFloat.description32.MantissaBits - description.MantissaBits;
                    half = (1 << (bitDiff - 1)) - 1;
                    bias = ((m >> bitDiff) & 1);

                    m = (m + half + bias) >> bitDiff;

                    if ((m & (1 << description.MantissaBits))) {
                        // Overflow in mantissa.
                        m = 0;
                        e += 1;
                    }
                } else {
                    bitDiff = description.MantissaBits - tcuFloat.description32.MantissaBits;
                    m = (m << bitDiff);
                }

                if (e > eMax) {
                    // Overflow.
                    return description.infNumber(sign);
                } else {
                    DE_ASSERT(deMath.deInRange32(e, eMin, eMax));
                    DE_ASSERT(((e + description.ExponentBias) & ~((1 << description.ExponentBits) - 1)) == 0);
                    DE_ASSERT((m & ~((1 << description.MantissaBits) - 1)) == 0);

                    return (s | ((e + description.ExponentBias) << description.MantissaBits)) | m;
                }
            }
        }
    };
})();

/**
 * Builds a 10 bit tcuFloat.deFloat
 * @param {number} value (64-bit JS float)
 * @return {tcuFloat.deFloat}
 */
tcuFloat.newFloat10 = function(value) {
    /**@type {tcuFloat.deFloat} */ var other32 = new tcuFloat.deFloat().deFloatNumber(value);
    return tcuFloat.description10.convert(other32);
};

/**
 * Builds a 11 bit tcuFloat.deFloat
 * @param {number} value (64-bit JS float)
 * @return {tcuFloat.deFloat}
 */
tcuFloat.newFloat11 = function(value) {
    /**@type {tcuFloat.deFloat} */ var other32 = new tcuFloat.deFloat().deFloatNumber(value);
    return tcuFloat.description11.convert(other32);
};

/**
 * Builds a 16 bit tcuFloat.deFloat
 * @param {number} value (64-bit JS float)
 * @return {tcuFloat.deFloat}
 */
tcuFloat.newFloat16 = function(value) {
    /**@type {tcuFloat.deFloat} */ var other32 = new tcuFloat.deFloat().deFloatNumber(value);
    return tcuFloat.description16.convert(other32);
};

/**
 * Builds a 16 bit tcuFloat.deFloat from raw bits
 * @param {number} value (16-bit value)
 * @return {tcuFloat.deFloat}
 */
tcuFloat.newFloat32From16 = function(value) {
    var other16 = tcuFloat.newDeFloatFromParameters(value, tcuFloat.description16);
    return tcuFloat.description32.convert(other16);
};

/**
 * Builds a 16 bit tcuFloat.deFloat with no denorm support
 * @param {number} value (64-bit JS float)
 * @return {tcuFloat.deFloat}
 */
tcuFloat.newFloat16NoDenorm = function(value) {
    /**@type {tcuFloat.deFloat} */ var other32 = new tcuFloat.deFloat().deFloatNumber(value);
    return tcuFloat.description16.convert(other32);
};

/**
 * Builds a 32 bit tcuFloat.deFloat
 * @param {number} value (64-bit JS float)
 * @return {tcuFloat.deFloat}
 */
tcuFloat.newFloat32 = function(value) {
    return new tcuFloat.deFloat().deFloatNumber(value);
};

tcuFloat.numberToFloat11 = function(value) {
    return tcuFloat.convertFloat32Inline(value, tcuFloat.description11);
};

tcuFloat.float11ToNumber = (function() {
    var x = tcuFloat.newDeFloatFromParameters(0, tcuFloat.description11);
    return function(float11) {
        x.deFloatParametersNumber(float11);
        return x.getValue();
    };
})();

tcuFloat.numberToFloat10 = function(value) {
    return tcuFloat.convertFloat32Inline(value, tcuFloat.description10);
};

tcuFloat.float10ToNumber = (function() {
    var x = tcuFloat.newDeFloatFromParameters(0, tcuFloat.description10);
    return function(float10) {
        x.deFloatParametersNumber(float10);
        return x.getValue();
    };
})();

tcuFloat.numberToHalfFloat = function(value) {
    return tcuFloat.convertFloat32Inline(value, tcuFloat.description16);
};

tcuFloat.numberToHalfFloatNoDenorm = function(value) {
    return tcuFloat.newFloat16NoDenorm(value).bits();
};

tcuFloat.halfFloatToNumber = (function() {
    var x = tcuFloat.newDeFloatFromParameters(0, tcuFloat.description16);
    return function(half) {
        x.deFloatParametersNumber(half);
        return x.getValue();
    };
})();

tcuFloat.halfFloatToNumberNoDenorm = tcuFloat.halfFloatToNumber;

/**
 * Builds a 64 bit tcuFloat.deFloat
 * @param {number} value (64-bit JS float)
 * @return {tcuFloat.deFloat}
 */
tcuFloat.newFloat64 = function(value) {
    return new tcuFloat.deFloat().deFloatParameters(value, tcuFloat.description64);
};

});
