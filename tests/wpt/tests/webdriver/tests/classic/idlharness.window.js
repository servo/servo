// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/webdriver/

"use strict";

idl_test(
  ["webdriver"],
  ["html"],
  idl_array => {
    idl_array.add_objects({
      Navigator: ["navigator"]
    });
  }
);
