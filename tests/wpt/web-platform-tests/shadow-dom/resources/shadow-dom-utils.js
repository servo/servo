// Copyright 2012 Google Inc. All Rights Reserved.

/*
Distributed under both the W3C Test Suite License [1] and the W3C
3-clause BSD License [2]. To contribute to a W3C Test Suite, see the
policies and contribution forms [3].

[1] http://www.w3.org/Consortium/Legal/2008/04-testsuite-license
[2] http://www.w3.org/Consortium/Legal/2008/03-bsd-license
[3] http://www.w3.org/2004/10/27-testcases
*/

"use strict";

// custom element is also allowed.
var ATTACHSHADOW_SAFELISTED_ELEMENTS = [
    'article',
    'aside',
    'blockquote',
    'body',
    'div',
    'footer',
    'h1',
    'h2',
    'h3',
    'h4',
    'h5',
    'h6',
    'header',
    'nav',
    'p',
    'section',
    'span'
];

var HTML5_ELEMENT_NAMES = [
    'a', 'abbr', 'address', 'area', 'article', 'aside', 'audio',
    'b', 'base', 'bdi', 'bdo', 'blockquote', 'body', 'br', 'button',
    'canvas', 'caption', 'cite', 'code', 'col', 'colgroup', 'command',
    'datalist', 'dd', 'del', 'details', 'dfn', 'dialog', 'div', 'dl', 'dt',
    'em', 'embed',
    'fieldset', 'figcaption', 'figure', 'footer', 'form',
    'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'head', 'header', 'hgroup', 'hr',
    'html',
    'i', 'iframe', 'img', 'input', 'ins', 'kbd', 'keygen',
    'label', 'legend', 'li', 'link',
    'map', 'mark', 'menu', 'meta', 'meter',
    'nav', 'noscript',
    'object', 'ol', 'optgroup', 'option', 'output',
    'p', 'param', 'pre', 'progress',
    'q',
    'rp', 'rt', 'ruby',
    's', 'samp', 'script', 'section', 'select', 'small', 'source', 'span',
    'strong', 'style', 'sub',
    'table', 'tbody', 'td', 'textarea', 'tfoot', 'th', 'thead', 'time',
    'title', 'tr', 'track',
    'u', 'ul',
    'var', 'video',
    'wbr'
];

var ATTACHSHADOW_DISALLOWED_ELEMENTS = HTML5_ELEMENT_NAMES.filter(el => !ATTACHSHADOW_SAFELISTED_ELEMENTS.includes(el));

function unit(f) {
    return function () {
        var ctx = newContext();
        try {
            f(ctx);
        } finally {
            cleanContext(ctx);
        }
    }
}

function step_unit(f, ctx, t) {
    return function () {
        var done = false;
        try {
            f();
            done = true;
        } finally {
            if (done) {
                t.done();
            }
            cleanContext(ctx);
        }
    }
}

function assert_nodelist_contents_equal_noorder(actual, expected, message) {
    assert_equals(actual.length, expected.length, message);
    var used = [];
    for (var i = 0; i < expected.length; i++) {
        used.push(false);
    }
    for (i = 0; i < expected.length; i++) {
        var found = false;
        for (var j = 0; j < actual.length; j++) {
            if (used[j] == false && expected[i] == actual[j]) {
                used[j] = true;
                found = true;
                break;
            }
        }
        if (!found) {
            assert_unreached(message + ". Fail reason:  element not found: " + expected[i]);
        }
    }
}

//Example taken from http://www.w3.org/TR/shadow-dom/#event-retargeting-example
function createTestMediaPlayer(d) {
    d.body.innerHTML = '' +
    '<div id="player">' +
        '<input type="checkbox" id="outside-control">' +
        '<div id="player-shadow-host">' +
        '</div>' +
    '</div>';

    var playerShadowRoot = d.querySelector('#player-shadow-host').attachShadow({mode: 'open'});
    playerShadowRoot.innerHTML = '' +
        '<div id="controls">' +
            '<button class="play-button">PLAY</button>' +
            '<div tabindex="0" id="timeline">' +
                '<div id="timeline-shadow-host">' +
                '</div>' +
            '</div>' +
            '<div class="volume-slider-container" id="volume-slider-container">' +
                '<div tabindex="0" class="volume-slider" id="volume-slider">' +
                    '<div id="volume-shadow-host">' +
                    '</div>' +
                '</div>' +
            '</div>' +
        '</div>';

    var timeLineShadowRoot = playerShadowRoot.querySelector('#timeline-shadow-host').attachShadow({mode: 'open'});
    timeLineShadowRoot.innerHTML =  '<div class="slider-thumb" id="timeline-slider-thumb"></div>';

    var volumeShadowRoot = playerShadowRoot.querySelector('#volume-shadow-host').attachShadow({mode: 'open'});
    volumeShadowRoot.innerHTML = '<div class="slider-thumb" id="volume-slider-thumb"></div>';

    return {
        'playerShadowRoot': playerShadowRoot,
        'timeLineShadowRoot': timeLineShadowRoot,
        'volumeShadowRoot': volumeShadowRoot
        };
}

//FIXME This call of initKeyboardEvent works for WebKit-only.
//See https://bugs.webkit.org/show_bug.cgi?id=16735
// and https://bugs.webkit.org/show_bug.cgi?id=13368. Add check for browser here
function fireKeyboardEvent(doc, element, key) {
    var event = doc.createEvent('KeyboardEvent');
    event.initKeyboardEvent("keydown", true, true, doc.defaultView, key, 0, false, false, false, false);
    element.dispatchEvent(event);
}
