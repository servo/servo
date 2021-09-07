"use strict";

var HTML5_ELEMENTS = [ 'a', 'abbr', 'address', 'area', 'article', 'aside',
        'audio', 'b', 'base', 'bdi', 'bdo', 'blockquote', 'body', 'br',
        'button', 'canvas', 'caption', 'cite', 'code', 'col', 'colgroup',
        'command', 'datalist', 'dd', 'del', 'details', 'dfn', 'dialog', 'div',
        'dl', 'dt', 'em', 'embed', 'fieldset', 'figcaption', 'figure',
        'footer', 'form', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'head', 'header',
        'hgroup', 'hr', 'html', 'i', 'iframe', 'img', 'input', 'ins', 'kbd',
        'keygen', 'label', 'legend', 'li', 'link', 'map', 'mark', 'menu',
        'meta', 'meter', 'nav', 'noscript', 'object', 'ol', 'optgroup',
        'option', 'output', 'p', 'param', 'pre', 'progress', 'q', 'rp', 'rt',
        'ruby', 's', 'samp', 'script', 'section', 'select', 'small', 'source',
        'span', 'strong', 'style', 'sub', 'table', 'tbody', 'td', 'textarea',
        'tfoot', 'th', 'thead', 'time', 'title', 'tr', 'track', 'u', 'ul',
        'var', 'video', 'wbr' ];

// only void (without end tag) HTML5 elements
var HTML5_VOID_ELEMENTS = [ 'area', 'base', 'br', 'col', 'command', 'embed',
        'hr', 'img', 'input', 'keygen', 'link', 'meta', 'param', 'source',
        'track', 'wbr' ];

// https://html.spec.whatwg.org/multipage/multipage/forms.html#form-associated-element
var HTML5_FORM_ASSOCIATED_ELEMENTS = [ 'button', 'fieldset', 'input',
        'object', 'output', 'select', 'textarea' ];

function newDocument() {
    var d = document.implementation.createDocument();
    return d;
}

function newHTMLDocument() {
    var d = document.implementation.createHTMLDocument('Test Document');
    return d;
}

function newXHTMLDocument() {
    var doctype = document.implementation.createDocumentType('html',
            '-//W3C//DTD XHTML 1.0 Transitional//EN',
            'http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd');

    var d = document.implementation.createDocument(
            'http://www.w3.org/1999/xhtml', 'html', doctype);
    return d;
}

function newIFrame(context, src) {
    if (typeof (context) === 'undefined'
            || typeof (context.iframes) !== 'object') {
        assert_unreached('Illegal context object in newIFrame');
    }

    var iframe = document.createElement('iframe');

    if (typeof (src) != 'undefined') {
        iframe.src = src;
    }
    document.body.appendChild(iframe);
    context.iframes.push(iframe);

    assert_true(typeof (iframe.contentWindow) != 'undefined'
            && typeof (iframe.contentWindow.document) != 'undefined'
            && iframe.contentWindow.document != document,
            'Failed to create new rendered document');
    return iframe;
}

function newRenderedHTMLDocument(context) {
    var frame = newIFrame(context);
    var d = frame.contentWindow.document;
    return d;
}

function newContext() {
    return {
        iframes : []
    };
}

function cleanContext(context) {
    context.iframes.forEach(function(e) {
        e.parentNode.removeChild(e);
    });
}

// run given test function in context
// the context is cleaned up after test completes.
function inContext(f) {
    return function() {
        var context = newContext();
        try {
            f(context);
        } finally {
            cleanContext(context);
        }
    };
}

// new context and iframe are created and url (if supplied) is asigned to
// iframe.src
// function f is bound to the iframe onload event or executed directly after
// iframe creation
// the context is passed to function as argument
function testInIFrame(url, f, testName, testProps) {
    if (url) {
        var t = async_test(testName);
        t.step(function() {
            var context = newContext();
            var iframe = newIFrame(context, url);
            iframe.onload = t.step_func(function() {
                try {
                    f(context);
                    t.done();
                } finally {
                    cleanContext(context);
                }
            });
        });
    } else {
        test(inContext(function(context) {
            newRenderedHTMLDocument(context);
            f(context);
        }), testName);
    }
}

function assert_nodelist_contents_equal_noorder(actual, expected, message) {
    assert_equals(actual.length, expected.length, message);
    var used = [];
    for ( var i = 0; i < expected.length; i++) {
        used.push(false);
    }
    for (i = 0; i < expected.length; i++) {
        var found = false;
        for ( var j = 0; j < actual.length; j++) {
            if (used[j] == false && expected[i] == actual[j]) {
                used[j] = true;
                found = true;
                break;
            }
        }
        if (!found) {
            assert_unreached(message + ". Fail reason:  element not found: "
                    + expected[i]);
        }
    }
}

function isVoidElement(elementName) {
    return HTML5_VOID_ELEMENTS.indexOf(elementName) >= 0;
}

function checkTemplateContent(d, obj, html, id, nodeName) {

    obj.innerHTML = '<template id="tmpl">' + html + '</template>';

    var t = d.querySelector('#tmpl');

    if (id != null) {
        assert_equals(t.content.childNodes.length, 1, 'Element ' + nodeName
                + ' should present among template nodes');
        assert_equals(t.content.firstChild.id, id, 'Wrong element ID');
    }
    if (nodeName != null) {
        assert_equals(t.content.firstChild.nodeName, nodeName.toUpperCase(),
                'Wrong node name');
    }
}

function checkBodyTemplateContent(d, html, id, nodeName) {
    checkTemplateContent(d, d.body, html, id, nodeName);
}

function checkHeadTemplateContent(d, html, id, nodeName) {
    checkTemplateContent(d, d.head, html, id, nodeName);
}
