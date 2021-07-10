// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

class MyElement extends HTMLElement {};
customElements.define('my-element', MyElement);

idl_test(
  ['custom-state-pseudo-class'],
  ['html', 'wai-aria'],
  idl_array => {
    idl_array.add_objects({
      CustomStateSet: [ 'customStateSet' ],
    });

    const myElement = document.createElement('my-element');
    self.customStateSet = myElement.attachInternals().states;
  }
);
