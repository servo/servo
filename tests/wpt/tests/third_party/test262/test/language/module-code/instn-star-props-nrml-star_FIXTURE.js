// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

var notExportedVar2;
let notExportedLet2;
const notExportedConst2 = null;
function notExportedFunc2() {}
function* notExportedGen2() {}
class notExportedClass2 {}

var starBindingId;

export var starVarDecl;
export let starLetDecl;
export const starConstDecl = null;
export function starFuncDecl() {}
export function* starGenDecl() {}
export class starClassDecl {}
export { starBindingId };
export { starBindingId as starIdName };
export { starIndirectIdName } from './instn-star-props-nrml-indirect_FIXTURE.js';
export { starIndirectIdName as starIndirectIdName2 } from './instn-star-props-nrml-indirect_FIXTURE.js';
export * as starIndirectNamespaceBinding from './instn-star-props-nrml-indirect_FIXTURE.js';
