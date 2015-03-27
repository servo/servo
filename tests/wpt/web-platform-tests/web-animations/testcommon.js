/*
Distributed under both the W3C Test Suite License [1] and the W3C
3-clause BSD License [2]. To contribute to a W3C Test Suite, see the
policies and contribution forms [3].

[1] http://www.w3.org/Consortium/Legal/2008/04-testsuite-license
[2] http://www.w3.org/Consortium/Legal/2008/03-bsd-license
[3] http://www.w3.org/2004/10/27-testcases
 */

"use strict";

var ANIMATION_END_TIME = 1000;
var ANIMATION_TOP_DEFAULT = 300;
var ANIMATION_TOP_0 = 10;
var ANIMATION_TOP_0_5 = 100;
var ANIMATION_TOP_1 = 200;

var KEYFRAMES = [ {
    top : ANIMATION_TOP_0 + 'px',
    offset : 0
}, {
    top : ANIMATION_TOP_0_5 + 'px',
    offset : 1 / 2
}, {
    top : ANIMATION_TOP_1 + 'px',
    offset : 1
} ];

// creates new animation for given target
function newAnimation(animationTarget) {
    animationTarget.style.top = ANIMATION_TOP_DEFAULT + 'px';
    return new Animation(animationTarget, KEYFRAMES, ANIMATION_END_TIME);
}

// creates div element, appends it to the document body and
// add removing of the created element to test cleanup
function createDiv(test, doc) {
    if (!doc) {
        doc = document;
    }
    var div = doc.createElement('div');
    doc.body.appendChild(div);
    test.add_cleanup(function() {
        removeElement(div);
    });
    return div;
}

// Removes element
function removeElement(element) {
    element.parentNode.removeChild(element);
}

// Returns the type name of given object
function type(object) {
    return Object.prototype.toString.call(object).slice(8, -1);
}
