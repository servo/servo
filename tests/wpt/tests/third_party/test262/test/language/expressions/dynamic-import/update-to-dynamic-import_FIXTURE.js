// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

Function('return this;')().test262Update = name => x = name;

export default function() {
    return import('./update-to-dynamic-import-other_FIXTURE.js');
}

export var x = 'first';
