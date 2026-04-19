// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

var notExportedVar1;
let notExportedLet1;
const notExportedConst1 = null;
function notExportedFunc1() {}
function* notExportedGen1() {}
class notExportedClass1 {}

var localBindingId;

export var localVarDecl;
export let localLetDecl;
export const localConstDecl = null;
export function localFuncDecl() {}
export function* localGenDecl() {}
export class localClassDecl {}
export { localBindingId };
export { localBindingId as localIdName };
export { indirectIdName } from './instn-star-props-nrml-indirect_FIXTURE.js';
export { indirectIdName as indirectIdName2 } from './instn-star-props-nrml-indirect_FIXTURE.js';
export * as namespaceBinding from './instn-star-props-nrml-indirect_FIXTURE.js';

export * from './instn-star-props-nrml-star_FIXTURE.js';

