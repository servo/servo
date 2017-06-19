// Copyright 2016 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

onmessage = function(e) {
    var compiled_module = e.data;
    var instance = new WebAssembly.Instance(compiled_module);
    if (instance === undefined) {
        postMessage("error!");
        return;
    }
    var entrypoint = instance.exports["increment"];

    if (typeof entrypoint !== "function") {
        postMessage("error!");
        return;
    }

    var ret = entrypoint(42);
    postMessage(ret);
}
