// META: timeout=long
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/mathml-core/

'use strict';

let objects = {};

const elements = [
  'math',
  'a',
  'annotation',
  'annotation-xml',
  'maction',
  'math',
  'merror',
  'mfrac',
  'mi',
  'mmultiscripts',
  'mn',
  'mo',
  'mover',
  'mpadded',
  'mphantom',
  'mprescripts',
  'mroot',
  'mrow',
  'ms',
  'mspace',
  'msqrt',
  'mstyle',
  'msub',
  'msubsup',
  'msup',
  'mtable',
  'mtd',
  'mtext',
  'mtr',
  'munder',
  'munderover',
  'semantics'
];

idl_test(
  ['mathml-core'],
  ['cssom', 'html', 'dom'],
  idlArray => {
    const mathmlUrl = 'http://www.w3.org/1998/Math/MathML';
    for (const element of elements) {
      try {
        objects[element] = document.createElementNS(mathmlUrl, element);
      } catch (e) {
        // Will be surfaced by idlharness.js's test_object below.
      }
    }

    idlArray.add_objects({
      MathMLElement: [
        'objects.annotation',
        'objects["annotation-xml"]',
        'objects.maction',
        'objects.math',
        'objects.merror',
        'objects.mfrac',
        'objects.mi',
        'objects.mmultiscripts',
        'objects.mn',
        'objects.mo',
        'objects.mover',
        'objects.mpadded',
        'objects.mphantom',
        'objects.mprescripts',
        'objects.mroot',
        'objects.mrow',
        'objects.ms',
        'objects.mspace',
        'objects.msqrt',
        'objects.mstyle',
        'objects.msub',
        'objects.msubsup',
        'objects.msup',
        'objects.mtable',
        'objects.mtd',
        'objects.mtext',
        'objects.mtr',
        'objects.munder',
        'objects.munderover',
        'objects.semantics'
      ],
      MathMLAnchorElement: ['objects.a'],
    });
    idlArray.prevent_multiple_testing('MathMLElement');
  }
);
