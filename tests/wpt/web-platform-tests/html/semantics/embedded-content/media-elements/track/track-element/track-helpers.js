function enableAllTextTracks(textTracks) {
    for (var i = 0; i < textTracks.length; i++) {
        var track = textTracks[i];
        if (track.mode == "disabled")
            track.mode = "hidden";
    }
}

function assert_cues_equal(cues, expected) {
    assert_equals(cues.length, expected.length);
    for (var i = 0; i < cues.length; i++) {
        assert_equals(cues[i].id, expected[i].id);
        assert_equals(cues[i].startTime, expected[i].startTime);
        assert_equals(cues[i].endTime, expected[i].endTime);
        assert_equals(cues[i].text, expected[i].text);
    }
}

function assert_cues_match(cues, expected) {
    assert_equals(cues.length, expected.length);
    for (var i = 0; i < cues.length; i++) {
        var cue = cues[i];
        var expectedItem = expected[i];
        for (var property of Object.getOwnPropertyNames(expectedItem))
            assert_equals(cue[property], expectedItem[property]);
    }
}

function assert_cues_html_content(cues, expected) {
    assert_equals(cues.length, expected.length);
    for (var i = 0; i < cues.length; i++) {
        var expectedItem = expected[i];
        var property = Object.getOwnPropertyNames(expectedItem)[0];
        var propertyValue = expectedItem[property];
        assert_equals(propertyValue(cues[i]), expectedItem.expected);
    }
}

function check_cues_from_track(src, func) {
    async_test(function(t) {
        var video = document.createElement("video");
        var trackElement = document.createElement("track");
        trackElement.src = src;
        trackElement.default = true;
        video.appendChild(trackElement);

        trackElement.onload = t.step_func_done(function() {
            func(trackElement.track);
        });
    }, "Check cues from " + src);
}

function assert_cue_fragment(cue, children) {
    var fragment = createFragment(children);
    assert_true(fragment.isEqualNode(cue.getCueAsHTML()));
}

function assert_cue_fragment_as_textcontent(cue, children) {
    var fragment = createFragment(children);
    assert_equals(cue.getCueAsHTML().textContent, fragment.textContent);
}

function createFragment(children) {
    var fragment = document.createDocumentFragment();
    cloneChildrenToFragment(fragment, children);
    return fragment;
}

function cloneChildrenToFragment(root, children) {
    for (var child of children) {
        var childElement;
        if (child.type == "text") {
            childElement = document.createTextNode(child.value);
        } else {
            childElement = document.createElement(child.type);
            var styles = child.style || {};
            for (var attr of Object.getOwnPropertyNames(styles))
                childElement[attr] = styles[attr];
            cloneChildrenToFragment(childElement, child.value);
        }
        root.appendChild(childElement);
    }
}