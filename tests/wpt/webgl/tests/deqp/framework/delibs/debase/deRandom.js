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

/**
 * This class allows one to create a random integer, floating point number or boolean (TODO, deRandom.choose random items from a list and deRandom.shuffle an array)
 */
'use strict';
goog.provide('framework.delibs.debase.deRandom');

goog.scope(function() {

var deRandom = framework.delibs.debase.deRandom;

/**
 * Array of pseudo random numbers based on seed
 * @constructor
 * @struct
 */
deRandom.deRandom = function() {
    /** @type {number} */ this.x = 0;
    /** @type {number} */ this.y = 0;
    /** @type {number} */ this.z = 0;
    /** @type {number} */ this.w = 0;
};

/**
 * deRandom.Random number generator init
 * @param {deRandom.deRandom} rnd Array to store random numbers
 * @param {number} seed Number for seed
 */
deRandom.deRandom_init = function(rnd, seed) {
    rnd.x = (-seed ^ 123456789);
    rnd.y = (362436069 * seed);
    rnd.z = (521288629 ^ (seed >> 7));
    rnd.w = (88675123 ^ (seed << 3));
};

/**
 * Function to get random int
 * @param {deRandom.deRandom} rnd Initialised array of random numbers
 * @param {Array<number>=} opts Min and max for range
 * @return {number} deRandom.Random int
 */
deRandom.deRandom_getInt = function(rnd, opts) {
    if (opts != undefined && opts[0] != undefined && opts[1] != undefined) {
        if (opts[0] == 0x80000000 && opts[1] == 0x7fffffff) {
            return deRandom.deRandom_getInt(rnd);
        } else {
            return opts[0] + (deRandom.deRandom_getInt(rnd) % (opts[1] - opts[0] + 1));
        }
    }
    var w = rnd.w;
    var t;

    t = rnd.x ^ (rnd.x << 11);
    rnd.x = rnd.y;
    rnd.y = rnd.z;
    rnd.z = w;
    rnd.w = w = (w ^ (w >> 19)) ^ (t ^ (t >> 8));
    return w;
};

/**
 * Function to get random float
 * @param {deRandom.deRandom} rnd Initialised array of random numbers
 * @param {Array<number>=} opts Min and max for range
 * @return {number} deRandom.Random float
 */
deRandom.deRandom_getFloat = function(rnd, opts) {
    if (opts != undefined && opts[0] != undefined && opts[1] != undefined) {
        if (opts[0] <= opts[1]) {
            return opts[0] + (opts[1] - opts[0]) * deRandom.deRandom_getFloat(rnd);
        }
    } else {
        return (deRandom.deRandom_getInt(rnd) & 0xFFFFFFF) / (0xFFFFFFF + 1);
    }
    throw new Error('Invalid arguments');
};

/**
 * Function to get random boolean
 * @param {deRandom.deRandom} rnd Initialised array of random numbers
 * @return {boolean} deRandom.Random boolean
 */
deRandom.deRandom_getBool = function(rnd) {
    var val;
    val = deRandom.deRandom_getInt(rnd);
    return ((val & 0xFFFFFF) < 0x800000);
};

/**
 * Function to get a common base seed
 * @return {number} constant
 */
deRandom.getBaseSeed = function() {
    return 42;
};

/**
 * TODO Function to deRandom.choose random items from a list
 * @template T
 * @param {deRandom.deRandom} rnd Initialised array of random numbers
 * @param {Array<T>} elements Array segment already defined
 * @param {Array<T>=} resultOut Array where to store the elements in. If undefined, default to array of (num) elements.
 * @param {number=} num Number of items to store in resultOut. If undefined, default to 1.
 * @return {Array<T>} Even though result is stored in resultOut, return it here as well.
 */
deRandom.choose = function(rnd, elements, resultOut, num) {
    var items = num || 1;
    var temp = elements.slice();
    if (!resultOut)
        resultOut = [];

    while (items-- > 0) {
        var index = deRandom.deRandom_getInt(rnd, [0, temp.length - 1]);
        resultOut.push(temp[index]);
        temp.splice(index, 1);
    }
    return resultOut;
};

/**
 * TODO Function to deRandom.choose weighted random items from a list
 * @param {deRandom.deRandom} rnd Initialised randomizer
 * @param {Array<number>} array Array to choose items from
 * @param {Array<number>} weights Weights array
 * @return {number} Result output
 */
deRandom.chooseWeighted = function(rnd, array, weights) {
    // Compute weight sum
    /** @type {number} */ var weightSum = 0.0;
    /** @type {number} */ var ndx;
    for (ndx = 0; ndx < array.length; ndx++)
        weightSum += weights[ndx];

    // Random point in 0..weightSum
    /** @type {number} */ var p = deRandom.deRandom_getFloat(rnd, [0.0, weightSum]);

    // Find item in range
    /** @type {number} */ var lastNonZero = array.length;
    /** @type {number} */ var curWeight = 0.0;
    for (ndx = 0; ndx != array.length; ndx++) {
        /** @type {number} */ var w = weights[ndx];

        curWeight += w;

        if (p < curWeight)
            return array[ndx];
        else if (w > 0.0)
            lastNonZero = ndx;
    }

    assertMsgOptions(lastNonZero != array.length, 'Index went out of bounds', false, true);
    return array[lastNonZero];
};

/**
 * TODO Function to deRandom.shuffle an array
 * @param {deRandom.deRandom} rnd Initialised array of random numbers
 * @param {Array} elements Array to deRandom.shuffle
 * @return {Array} Shuffled array
 */
deRandom.shuffle = function(rnd, elements) {
    var index = elements.length;

    while (index > 0) {
        var random = deRandom.deRandom_getInt(rnd, [0, index - 1]);
        index -= 1;
        var elem = elements[index];
        elements[index] = elements[random];
        elements[random] = elem;
    }
    return elements;
};

/**
 * This function is used to create the deRandom.Random object and
 * initialise the random number with a seed.
 * It contains functions for generating random numbers in a variety of formats
 * @constructor
 * @param {number} seed Number to use as a seed
 */
deRandom.Random = function(seed) {
    /**
     * Instance of array of pseudo random numbers based on seeds
    */
    this.m_rnd = new deRandom.deRandom();

    //initialise the random numbers based on seed
    deRandom.deRandom_init(this.m_rnd, seed);
};

/**
 * Function to get random boolean
 * @return {boolean} deRandom.Random boolean
 */
deRandom.Random.prototype.getBool = function() { return deRandom.deRandom_getBool(this.m_rnd) == true; };
/**
 * Function to get random float
 * @param {number=} min Min for range
 * @param {number=} max Max for range
 * @return {number} deRandom.Random float
 */
deRandom.Random.prototype.getFloat = function(min, max) { return deRandom.deRandom_getFloat(this.m_rnd, [min, max]) };
/**
 * Function to get random int
 * @param {number=} min Min for range
 * @param {number=} max Max for range
 * @return {number} deRandom.Random int
 */
deRandom.Random.prototype.getInt = function(min, max) {return deRandom.deRandom_getInt(this.m_rnd, [min, max])};
/**
 * TODO Function to deRandom.choose random items from a list
 * @template T
 * @param {Array<T>} elements Array segment already defined
 * @param {Array<T>=} resultOut Array where to store the elements in. If undefined, default to array of (num) elements.
 * @param {number=} num Number of items to store in resultOut. If undefined, default to 1.
 * @return {Array<T>} Even though result is stored in resultOut, return it here as well.
 */
deRandom.Random.prototype.choose = function(elements, resultOut, num) {return deRandom.choose(this.m_rnd, elements, resultOut, num)};
/**
 * choose weighted random items from a list
 * @param {Array<number>} array Array to choose items from
 * @param {Array<number>} weights Weights array
 * @return {number} Result output
 */
deRandom.Random.prototype.chooseWeighted = function(array, weights) {return deRandom.chooseWeighted(this.m_rnd, array, weights)};
/**
 * TODO Function to deRandom.shuffle an array
 * @param {Array} elements Array to deRandom.shuffle
 * @return {Array} Shuffled array
 */
deRandom.Random.prototype.shuffle = function(elements) {return deRandom.shuffle(this.m_rnd, elements)};

/**
 * Function to get a common base seed
 * @return {number} constant
 */
deRandom.Random.prototype.getBaseSeed = function() {
    return deRandom.getBaseSeed();
};

});
