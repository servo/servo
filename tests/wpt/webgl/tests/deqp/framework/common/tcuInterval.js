/*-------------------------------------------------------------------------
 * drawElements Quality Program Tester Core
 * ----------------------------------------
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
 *//*!
 * \file
 * \brief Interval arithmetic and floating point precisions.
 *//*--------------------------------------------------------------------*/
 'use strict';
 goog.provide('framework.common.tcuInterval');
 goog.require('framework.delibs.debase.deMath');

 goog.scope(function() {

     var tcuInterval = framework.common.tcuInterval;
     var deMath = framework.delibs.debase.deMath;

    /**
     * @typedef {function(number):number}
     */
    tcuInterval.DoubleFunc1;

    /**
     * @typedef {function(number, number):number}
     */
    tcuInterval.DoubleFunc2;

    /**
     * @typedef {function(number,number,number):number}
     */
    tcuInterval.DoubleFunc3;

    /**
     * @typedef {function(number):tcuInterval.Interval}
     */
    tcuInterval.DoubleIntervalFunc1;

    /**
     * @typedef {function(number,number):tcuInterval.Interval}
     */
    tcuInterval.DoubleIntervalFunc2;

    /**
     * @typedef {function(number,number,number):tcuInterval.Interval}
     */
    tcuInterval.DoubleIntervalFunc3;

    /**
     * @param {function(number): number} func
     * @param {tcuInterval.Interval} arg0
     * @return {tcuInterval.Interval}
     */
    tcuInterval.applyMonotone1p = function(func, arg0) {
        /**
         * @param {number=} x
         * @param {number=} y
         * @return {number}
         */
        var body = function(x, y) {
            x = x || 0;
            return func(x);
        };
        return tcuInterval.applyMonotone1(arg0,
            function(x) { return tcuInterval.setInterval(body, x); });
    };

    /**
     * @param {function(number): tcuInterval.Interval} func
     * @param {tcuInterval.Interval} arg0
     * @return {tcuInterval.Interval}
     */
    tcuInterval.applyMonotone1i = function(func, arg0) {
        return tcuInterval.withIntervals(func(arg0.lo()), func(arg0.hi()));
    };

    /**
     * @param {function(number, number): number} func
     * @param {tcuInterval.Interval} arg0
     * @param {tcuInterval.Interval} arg1
     * @return {tcuInterval.Interval}
     */
    tcuInterval.applyMonotone2p = function(func, arg0, arg1) {
        /**
         * @param {number=} x
         * @param {number=} y
         * @return {number}
         */
        var body = function(x, y) {
            x = x || 0;
            y = y || 0;
            return func(x, y);
        };
        return tcuInterval.applyMonotone2(arg0, arg1,
            function(x, y) { return tcuInterval.setInterval(body, x, y); });
    };

    /**
     * @param {function(number, number): tcuInterval.Interval} func
     * @param {tcuInterval.Interval} arg0
     * @param {tcuInterval.Interval} arg1
     * @return {tcuInterval.Interval}
     */
    tcuInterval.applyMonotone2i = function(func, arg0, arg1) {
        /** @type {number} */ var lo0 = arg0.lo();
        /** @type {number} */ var hi0 = arg0.hi();
        /** @type {number} */ var lo1 = arg1.lo();
        /** @type {number} */ var hi1 = arg1.hi();
        var a = tcuInterval.withIntervals(func(lo0, lo1), func(lo0, hi1));
        var b = tcuInterval.withIntervals(func(hi0, lo1), func(hi0, hi1));
        return tcuInterval.withIntervals(a, b);
    };

    /**
     * @constructor
     * @param {number=} val
     */
    tcuInterval.Interval = function(val) {
        if (val === undefined) {
            this.m_hasNaN = false;
            this.m_lo = Number.POSITIVE_INFINITY;
            this.m_hi = Number.NEGATIVE_INFINITY;
        } else {
            this.m_hasNaN = isNaN(val);
            this.m_lo = this.m_hasNaN ? Number.POSITIVE_INFINITY : val;
            this.m_hi = this.m_hasNaN ? Number.NEGATIVE_INFINITY : val;
        }
    };

    tcuInterval.Interval.prototype.toString = function() {
        var str = 'Interval(' + this.m_lo + ', ' + this.m_hi;
        if (this.m_hasNaN)
            str += ', hasNaN';
        str += ')';
        return str;
    };

    /**
     * @param {tcuInterval.Interval} a
     * @param {tcuInterval.Interval} b
     * @return {tcuInterval.Interval}
     */
    tcuInterval.withIntervals = function(a, b) {
        /** @type {tcuInterval.Interval} */ var interval = new tcuInterval.Interval();
        interval.m_hasNaN = (a.m_hasNaN || b.m_hasNaN);
        interval.m_lo = Math.min(a.m_lo, b.m_lo);
        interval.m_hi = Math.max(a.m_hi, b.m_hi);
        return interval;
    };

    /**
     * @param {number} a
     * @param {number} b
     * @return {tcuInterval.Interval}
     */
    tcuInterval.withNumbers = function(a, b) {
        var x = new tcuInterval.Interval(a);
        var y = new tcuInterval.Interval(b);
        return tcuInterval.withIntervals(x, y);
    };

    /**
     * @param {boolean} hasNaN_
     * @param {number} lo_
     * @param {number} hi_
     * @return {tcuInterval.Interval}
     */
    tcuInterval.withParams = function(hasNaN_, lo_, hi_) {
        /** @type {tcuInterval.Interval} */ var interval = new tcuInterval.Interval();
        interval.m_hasNaN = hasNaN_;
        interval.m_lo = lo_;
        interval.m_hi = hi_;
        return interval;
    };

    /**
     * @return {number}
     */
    tcuInterval.Interval.prototype.length = function() {
        return this.m_hi - this.m_lo;
    };

    /**
     * @return {number}
     */
    tcuInterval.Interval.prototype.lo = function() {
        return this.m_lo;
    };

    /**
     * @return {number}
     */
    tcuInterval.Interval.prototype.hi = function() {
        return this.m_hi;
    };

    /**
     * @return {boolean}
     */
    tcuInterval.Interval.prototype.hasNaN = function() {
        return this.m_hasNaN;
    };

    /**
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.prototype.nan = function() {
        return this.m_hasNaN ? new tcuInterval.Interval(NaN) : new tcuInterval.Interval();
    };

    /**
     * @return {boolean}
     */
    tcuInterval.Interval.prototype.empty = function() {
        return this.m_lo > this.m_hi;
    };

    /**
     * @return {boolean}
     */
    tcuInterval.Interval.prototype.isFinite = function() {
        return isFinite(this.m_lo) && isFinite(this.m_hi);
    };

    /**
     * @return {boolean}
     */
    tcuInterval.Interval.prototype.isOrdinary = function() {
        return !this.hasNaN() && !this.empty() && this.isFinite();
    };

    /**
     * @param {tcuInterval.Interval} other
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.prototype.operatorOrBinary = function(other) {
        /** @type {tcuInterval.Interval} */ var temp = new tcuInterval.Interval();
        temp.m_hasNaN = this.m_hasNaN || other.m_hasNaN;
        temp.m_lo = Math.min(this.m_lo, other.m_lo);
        temp.m_hi = Math.max(this.m_hi, other.m_hi);
        return temp;
    };

    /**
     * @param {tcuInterval.Interval} other
     */
    tcuInterval.Interval.prototype.operatorOrAssignBinary = function(other) {
        /** @type {tcuInterval.Interval} */ var temp = this.operatorOrBinary(other);
        this.m_hasNaN = temp.m_hasNaN;
        this.m_lo = temp.m_lo;
        this.m_hi = temp.m_hi;
    };

    /**
     * @param {tcuInterval.Interval} other
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.prototype.operatorAndBinary = function(other) {
        /** @type {tcuInterval.Interval} */ var temp = new tcuInterval.Interval();
        temp.m_hasNaN = this.m_hasNaN && other.m_hasNaN;
        temp.m_lo = Math.max(this.m_lo, other.m_lo);
        temp.m_hi = Math.min(this.m_hi, other.m_hi);
        return temp;
    };

    /**
     * @param {tcuInterval.Interval} other
     */
    tcuInterval.Interval.prototype.operatorAndAssignBinary = function(other) {
        /** @type {tcuInterval.Interval} */ var temp = this.operatorAndBinary(other);
        this.m_hasNaN = temp.m_hasNaN;
        this.m_lo = temp.m_lo;
        this.m_hi = temp.m_hi;
    };

    /**
     * @param {tcuInterval.Interval} other
     * @return {boolean}
     */
    tcuInterval.Interval.prototype.contains = function(other) {
        return (other.lo() >= this.lo() && other.hi() <= this.hi() &&
                (!other.hasNaN() || this.hasNaN()));
    };

    /**
     * @param {tcuInterval.Interval} other
     * @return {boolean}
     */
    tcuInterval.Interval.prototype.intersects = function(other) {
        return ((other.hi() >= this.lo() && other.lo() >= this.hi()) ||
                (other.hasNaN() && this.hasNaN()));
    };

    /**
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.prototype.operatorNegative = function() {
        /** @type {tcuInterval.Interval} */ var temp = new tcuInterval.Interval();
        temp.m_hasNaN = this.m_hasNaN;
        temp.m_lo = -this.m_hi;
        temp.m_hi = -this.m_lo;
        return temp;
    };

    /**
     * @param {boolean=} nan
     * @return {tcuInterval.Interval}
     */
    tcuInterval.unbounded = function(nan) {
        if (nan === undefined)
            nan = false;
        return tcuInterval.withParams(nan, Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY);
    };

    /**
     * @return {number}
     */
    tcuInterval.Interval.prototype.midpoint = function() {
        return 0.5 * (this.hi() + this.lo()); // returns NaN when not bounded
    };

    /**
     * @param {tcuInterval.Interval} other
     * @return {boolean}
     */
    tcuInterval.Interval.prototype.operatorCompare = function(other) {
        return ((this.m_hasNaN == other.m_hasNaN) &&
                ((this.empty() && other.empty()) ||
                 (this.m_lo == other.m_lo && this.m_hi == other.m_hi)));
    };

    /**
     * @param {tcuInterval.Interval} x
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.operatorPositive = function(x) {
        return x;
    };

    /**
     * @param {tcuInterval.Interval} x
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.exp2 = function(x) {
        // std::pow
        return tcuInterval.applyMonotone2p(Math.pow, new tcuInterval.Interval(2), x);
    };

    /**
     * @param {tcuInterval.Interval} x
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.exp = function(x) {
        // std::exp
        return tcuInterval.applyMonotone1p(Math.exp, x);
    };

    /**
     * @param {tcuInterval.Interval} x
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.sign = function(x) {
        // TODO
        throw new Error('Unimplemented');
    };

    /**
     * @param {tcuInterval.Interval} x
     * @param {tcuInterval.Interval} y
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.operatorSum = function(x, y) {
        /** @type {tcuInterval.Interval} */ var ret = new tcuInterval.Interval();

        if (!x.empty() && !y.empty())
            ret = tcuInterval.setIntervalBounds(function(dummy) {return x.lo() + y.lo();}, function(dummy) {return x.hi() + y.hi();});
        if (x.hasNaN() || y.hasNaN())
            ret.operatorOrAssignBinary(new tcuInterval.Interval(NaN));

        return ret;
    };

    /**
     * @param {tcuInterval.Interval} x
     * @param {tcuInterval.Interval} y
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.operatorSub = function(x, y) {
        /** @type {tcuInterval.Interval} */ var ret = new tcuInterval.Interval();

        /**
         * @param {number=} x
         * @param {number=} y
         * @return {tcuInterval.Interval}
         */
        var body = function(x, y) {
            return new tcuInterval.Interval(x - y);
        };

        ret = tcuInterval.applyMonotone2(x, y, body);
        return ret;
    };

    /**
     * @param {tcuInterval.Interval} x
     * @param {tcuInterval.Interval} y
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.operatorMul = function(x, y) {
        /** @type {tcuInterval.Interval} */ var ret = new tcuInterval.Interval();

        /**
         * @param {number=} x
         * @param {number=} y
         * @return {tcuInterval.Interval}
         */
        var body = function(x, y) {
            return new tcuInterval.Interval(x * y);
        };

        ret = tcuInterval.applyMonotone2(x, y, body);

        return ret;
    };

    /**
     * @param {tcuInterval.Interval} nom
     * @param {tcuInterval.Interval} den
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.operatorDiv = function(nom, den) {
        if (den.contains(new tcuInterval.Interval(0))) {
            // \todo [2014-03-21 lauri] Non-inf endpoint when one den endpoint is
            // zero and nom doesn't cross zero?
            return tcuInterval.unbounded();
        } else {
            /** @type {tcuInterval.Interval} */ var ret = new tcuInterval.Interval();
            /**
             * @param {number=} x
             * @param {number=} y
             * @return {tcuInterval.Interval}
             */
            var body = function(x, y) {
                return new tcuInterval.Interval(x / y);
            };

            ret = tcuInterval.applyMonotone2(nom, den, body);

            return ret;
        }
    };

    /**
     * @param {tcuInterval.Interval} x
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.prototype.abs = function(x) {
        //std::abs
        /** @type {tcuInterval.Interval} */ var mono = tcuInterval.applyMonotone1p(Math.abs, x);
        var zero = new tcuInterval.Interval(0);
        if (x.contains(zero))
            return tcuInterval.withIntervals(zero, mono);

        return mono;
    };

    /**
     * @param {tcuInterval.Interval} x
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.sqrt = function(x) {
        return tcuInterval.applyMonotone1p(Math.sqrt, x);
    };

    /**
     * @param {tcuInterval.Interval} x
     * @return {tcuInterval.Interval}
     */
    tcuInterval.Interval.inverseSqrt = function(x) {
        var ret = new tcuInterval.Interval(1);
        ret = tcuInterval.Interval.operatorDiv(ret, tcuInterval.Interval.sqrt(x));
        return ret;
    };

/**
 * @param {function(number=, number=): number} setLow
 * @param {function(number=, number=): number} setHigh
 * @param {number=} arg0
 * @param {number=} arg1
 * @return {tcuInterval.Interval}
 */
tcuInterval.setIntervalBounds = function(setLow, setHigh, arg0, arg1) {
    // TODO: No support for rounding modes. Originally, setLow() was rounded down and setHigh() rounded up
    var lo = new tcuInterval.Interval(setLow(arg0, arg1));
    var hi = new tcuInterval.Interval(setHigh(arg0, arg1));
    return lo.operatorOrBinary(hi);
};

/**
 * @param {function(number=, number=): number} set
 * @param {number=} arg0
 * @param {number=} arg1
 * @return {tcuInterval.Interval}
 */
tcuInterval.setInterval = function(set, arg0, arg1) {
    return tcuInterval.setIntervalBounds(set, set, arg0, arg1);
};

/**
 * @param {tcuInterval.Interval} arg
 * @param {function(number): tcuInterval.Interval} body
 * @return {tcuInterval.Interval}
 */
tcuInterval.applyMonotone1 = function(arg, body) {
    var ret = new tcuInterval.Interval();

    if (!arg.empty()) {
        var lo = body(arg.lo());
        var hi = body(arg.hi());
        ret = lo.operatorOrBinary(hi);
    }

    if (arg.hasNaN()) {
        ret = ret.operatorOrBinary(new tcuInterval.Interval(NaN));
    }

    return ret;
};

/**
 * TODO: Check if this function works properly
 * @param {tcuInterval.Interval} arg0
 * @param {tcuInterval.Interval} arg1
 * @param {function(number, number): tcuInterval.Interval} body
 * @return {tcuInterval.Interval}
 */
tcuInterval.applyMonotone2 = function(arg0, arg1, body) {
    var ret = new tcuInterval.Interval();

    if (!arg0.empty() && !arg1.empty()) {
        var lo0 = body(arg0.lo(), arg1.lo());
        var lo1 = body(arg0.lo(), arg1.hi());
        var hi0 = body(arg0.hi(), arg1.lo());
        var hi1 = body(arg0.hi(), arg1.hi());
        var a = lo0.operatorOrBinary(hi0);
        var b = lo1.operatorOrBinary(hi1);
        ret = a.operatorOrBinary(b);
    }

    if (arg0.hasNaN() || arg1.hasNaN()) {
        ret = ret.operatorOrBinary(new tcuInterval.Interval(NaN));
    }

    return ret;
};

/**
 * TODO: Check if this function works properly
 * @param {tcuInterval.Interval} arg0
 * @param {tcuInterval.Interval} arg1
 * @param {tcuInterval.Interval} arg2
 * @param {function(number, number, number): tcuInterval.Interval} body
 * @return {tcuInterval.Interval}
 */
tcuInterval.applyMonotone3 = function(arg0, arg1, arg2, body) {
    var ret = new tcuInterval.Interval();

    if (!arg0.empty() && !arg1.empty() && !arg2.empty()) {
        var i0 = body(arg0.lo(), arg1.lo(), arg2.lo());
        var i1 = body(arg0.lo(), arg1.lo(), arg2.hi());
        var i2 = body(arg0.lo(), arg1.hi(), arg2.lo());
        var i3 = body(arg0.lo(), arg1.hi(), arg2.hi());
        var i4 = body(arg0.hi(), arg1.lo(), arg2.lo());
        var i5 = body(arg0.hi(), arg1.lo(), arg2.hi());
        var i6 = body(arg0.hi(), arg1.hi(), arg2.lo());
        var i7 = body(arg0.hi(), arg1.hi(), arg2.hi());

        var low = Math.min(i0.lo(), i1.lo(), i2.lo(), i3.lo(), i4.lo(), i5.lo(), i6.lo(), i7.lo());
        var high = Math.max(i0.hi(), i1.hi(), i2.hi(), i3.hi(), i4.hi(), i5.hi(), i6.hi(), i7.hi());
        var hasNaN = i0.hasNaN() || i1.hasNaN() || i2.hasNaN() || i3.hasNaN() || i4.hasNaN() || i5.hasNaN() || i6.hasNaN() || i7.hasNaN();

        ret = tcuInterval.withParams(hasNaN, low, high);
    }

    if (arg0.hasNaN() || arg1.hasNaN() || arg2.hasNaN()) {
        ret = ret.operatorOrBinary(new tcuInterval.Interval(NaN));
    }

    return ret;
};

/** @const */ tcuInterval.POSITIVE_INFINITY = new tcuInterval.Interval(Infinity);
/** @const */ tcuInterval.NEGATIVE_INFINITY = new tcuInterval.Interval(-Infinity);
/** @const */ tcuInterval.ZERO = new tcuInterval.Interval(0);
/** @const */ tcuInterval.NAN = new tcuInterval.Interval(NaN);
});
