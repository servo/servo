// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: >
    Tests that the options numberingSystem and calendar can be  set through
    either the locale or the options.
author: Norbert Lindenberg, Daniel Ehrenberg
---*/

let defaultLocale = new Intl.DateTimeFormat().resolvedOptions().locale;

let supportedNumberingSystems = ["latn", "arab"].filter(nu =>
  new Intl.DateTimeFormat(defaultLocale + "-u-nu-" + nu)
    .resolvedOptions().numberingSystem === nu
);

let supportedCalendars = ["gregory", "chinese"].filter(ca =>
  new Intl.DateTimeFormat(defaultLocale + "-u-ca-" + ca)
    .resolvedOptions().calendar === ca
);

let options = [
    {key: "nu", property: "numberingSystem", type: "string", values: supportedNumberingSystems},
    {key: "ca", property: "calendar", type: "string", values: supportedCalendars}
];

options.forEach(function (option) {
    let dateTimeFormat, opt, result;
    
    // find out which values are supported for a property in the default locale
    let supportedValues = [];
    option.values.forEach(function (value) {
        opt = {};
        opt[option.property] = value;
        dateTimeFormat = new Intl.DateTimeFormat([defaultLocale], opt);
        result = dateTimeFormat.resolvedOptions()[option.property];
        if (result !== undefined && supportedValues.indexOf(result) === -1) {
            supportedValues.push(result);
        }
    });
    
    // verify that the supported values can also be set through the locale
    supportedValues.forEach(function (value) {
        dateTimeFormat = new Intl.DateTimeFormat([defaultLocale + "-u-" + option.key + "-" + value]);
        result = dateTimeFormat.resolvedOptions()[option.property];
        assert.sameValue(result, value, "Property " + option.property + " couldn't be set through locale extension key " + option.key + ".");
    });
    
    // verify that the options setting overrides the locale setting
    supportedValues.forEach(function (value) {
        let otherValue;
        option.values.forEach(function (possibleValue) {
            if (possibleValue !== value) {
                otherValue = possibleValue;
            }
        });
        if (otherValue !== undefined) {
            opt = {};
            opt[option.property] = value;
            dateTimeFormat = new Intl.DateTimeFormat([defaultLocale + "-u-" + option.key + "-" + otherValue], opt);
            result = dateTimeFormat.resolvedOptions()[option.property];
            assert.sameValue(result, value, "Options value for property " + option.property + " doesn't override locale extension key " + option.key + ".");
        }
    });
});
