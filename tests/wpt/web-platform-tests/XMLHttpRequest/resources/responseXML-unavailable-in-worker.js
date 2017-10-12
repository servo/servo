self.importScripts('/resources/testharness.js');

test(function() {
    let xhr = new XMLHttpRequest();
    assert_not_exists(xhr, "responseXML", "responseXML should not be available on instances.");
    assert_not_exists(XMLHttpRequest.prototype, "responseXML", "responseXML should not be on the prototype.");
}, "XMLHttpRequest's responseXML property should not be exposed in workers.");

done();
