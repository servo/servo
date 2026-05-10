// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializenumberformat
description: >
    Tests that the options numberingSystem and calendar can be  set through
    either the locale or the options.
author: Norbert Lindenberg, Daniel Ehrenberg
---*/

let defaultLocale = new Intl.NumberFormat().resolvedOptions().locale;

let supportedNumberingSystems = ["latn", "arab"].filter(nu =>
  new Intl.NumberFormat(defaultLocale + "-u-nu-" + nu)
    .resolvedOptions().numberingSystem === nu
);

let options = [
    {key: "nu", property: "numberingSystem", type: "string", values: supportedNumberingSystems},
];

options.forEach(function (option) {
    let numberFormat, opt, result;
    
    // find out which values are supported for a property in the default locale
    let supportedValues = [];
    option.values.forEach(function (value) {
        opt = {};
        opt[option.property] = value;
        numberFormat = new Intl.NumberFormat([defaultLocale], opt);
        result = numberFormat.resolvedOptions()[option.property];
        if (result !== undefined && supportedValues.indexOf(result) === -1) {
            supportedValues.push(result);
        }
    });
    
    // verify that the supported values can also be set through the locale
    supportedValues.forEach(function (value) {
        numberFormat = new Intl.NumberFormat([defaultLocale + "-u-" + option.key + "-" + value]);
        result = numberFormat.resolvedOptions()[option.property];
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
            numberFormat = new Intl.NumberFormat([defaultLocale + "-u-" + option.key + "-" + otherValue], opt);
            result = numberFormat.resolvedOptions()[option.property];
            assert.sameValue(result, value, "Options value for property " + option.property + " doesn't override locale extension key " + option.key + ".");
        }
    });
});

