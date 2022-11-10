"use strict";

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
