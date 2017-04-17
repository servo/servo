add_start_callback(initTransform);
add_result_callback(initTransform);

test( testAnimValDistinct, "patternTansformAnimValDistinct", {assert: 'animVal is a distinctly different object than baseVal'} );
test( testAnimValReadOnly, "patternTransformAnimValReadOnly", {assert: 'animVal is read-only'} );
test( testAnimValNumberOfItemsReadOnly, "patternTransformAnimValNumberOfItemsReadOnly", {assert: 'animVal.numberOfItems is read-only'} );    
test( testAnimValInitialize, "patternTransformAnimValInitialize", {assert: 'animVal.initialize() throws NO_MODIFICATION_ALLOWED_ERR'} );    
test( testAnimValClear, "patternTransformAnimValClear", {assert: 'animVal.clear() throws NO_MODIFICATION_ALLOWED_ERR'} );    
test( testAnimValInsertItemBefore, "patternTransformAnimValInsertItemBefore", {assert: 'animVal.transformAnimValInsertItemBefore() throws NO_MODIFICATION_ALLOWED_ERR'} );    
test( testAnimValReplaceItem, "patternTransformAnimValReplaceItem", {assert: 'animVal.transformAnimValReplaceItem() throws NO_MODIFICATION_ALLOWED_ERR'} );   
test( testAnimValAppendItem, "patternTransformAnimValAppendItem", {assert: 'animVal.transformAnimValAppendItem() throws NO_MODIFICATION_ALLOWED_ERR'} );   
test( testAnimValRemoveItem, "patternTransformAnimValRemoveItem", {assert: 'animVal.transformAnimValRemoveItem() throws NO_MODIFICATION_ALLOWED_ERR'} );   
test( testAnimValCreateSVGTransformFromMatrix, "patternTransformAnimValCreateSVGTransformFromMatrix", {assert: 'animVal.createSVGTransformFromMatrix() creates a new SVGTransform object'} );    
test( testAnimValConsolidate, "patternTransformAnimValConsolidate", {assert: 'Validate animVal.transformAnimValConsolidate() throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValTransformTypeReadOnly, "patternTransformAnimValTransformTypeReadOnly", {assert: 'Validate transform.type on animVal is read-only'} );
test( testAnimValTransformMatrixReadOnly, "patternTransformAnimValTransformMatrixReadOnly", {assert: 'Validate transform.matrix on animVal is read-only'} );
test( testAnimValTransformAngleReadOnly, "patternTransformAnimValTransformAngleReadOnly", {assert: 'Validate transform.angle on animVal is read-only'} );
test( testAnimValSetMatrix, "patternTransformAnimValSetMatrix", {assert: 'Validate transform.setMatrix on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetTranslate, "patternTransformAnimValSetTranslate", {assert: 'Validate transform.setTranslate on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetRotate, "patternTransformAnimValSetRotate", {assert: 'Validate transform.setRotate on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetScale, "patternTransformAnimValSetScale", {assert: 'Validate transform.setScale on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetSkewX, "patternTransformAnimValSetSkewX", {assert: 'Validate transform.setSkewX on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetSkewY, "patternTransformAnimValSetSkewY", {assert: 'Validate transform.setSkewY on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testBaseValReadOnly, "patternTransformBaseValReadOnly", {assert: 'baseVal is read-only'} ); 
test( testBaseValNumberOfItemsReadOnly, "patternTransformBaseValNumberOfItemsReadOnly", {assert: 'baseVal.numberOfItems is read-only'} );    
test( testBaseValInitialize, "patternTransformBaseValInitialize", {assert: 'baseVal.initialize() clears and inserts a new item'} );    
test( testBaseValInsertItemBefore, "patternTransformBaseValInsertItemBefore", {assert: 'baseVal.insertItemBefore() inserts item before specified index'} );    
test( testBaseValReplaceItem, "patternTransformBaseValReplaceItem", {assert: 'baseVal.replaceItem() replaces the item at the specified index'} );    
test( testBaseValAppendItem, "patternTransformBaseValAppendItem", {assert: 'baseVal.appendItem() appends item to the list'} );
test( testBaseValRemoveItem, "patternTransformBaseValRemoveItem", {assert: 'baseVal.removeItem() removes item from the list'} );
test( testBaseValCreateSVGTransformFromMatrix, "patternTransformBaseValCreateSVGTransformFromMatrix", {assert: 'baseVal.createSVGTransformFromMatrix creates a new SVGTransform object'} );    
test( testBaseValConsolidate, "patternTransformBaseValConsolidate", {assert: 'baseVal.consolidate() consolidates the list into a single transfrom'} );
test( testBaseValConsolidateEmptyList, "patternTransformBaseValConsolidateEmptyList", {assert: 'baseVal.consolidate() on an empty list returns null'} );
test( testBaseValClear, "patternTransformBaseValClear", {assert: 'baseVal.clear() clears all transforms'} );   
test( testBaseValInitializeInvalid, "patternTransformBaseValInitializeInvalid", {assert: 'baseVal.initialize() throws exception when passed an arg not in SVGTransform'} );     
test( testBaseValGetItemInvalid, "patternTransformBaseValGetItemInvalid", {assert: 'baseVal.getItem() handles invalid arguments correctly'} );     
test( testBaseValInsertItemBeforeInvalid, "patternTransformBaseValInsertItemBeforeInvalid", {assert: 'baseVal.insertItemBefore() handles invalid arguments correctly'} );    
test( testBaseValReplaceItemInvalid, "patternTransformBaseValReplaceItemInvalid", {assert: 'baseVal.replaceItem() handles invalid arguments correctly'} );    
test( testBaseValAppendItemInvalid, "patternTransformBaseValAppendItemInvalid", {assert: 'baseVal.appendItem() handles invalid arguments correctly'} );    
test( testBaseValRemoveItemInvalid, "patternTransformBaseValRemoveItemInvalid", {assert: 'baseVal.removeItem() handles invalid arguments correctly'} );  
test( testBaseValCreateSVGTransformFromMatrixInvalid, "patternTransformBaseValCreateSVGTransformFromMatrixInvalid", {assert: 'baseVal.createSVGTransformFromMatrix handles invalid arguments correctly'} ); 
test( testBaseValTransformTypeReadOnly, "patternTransformBaseValTransformTypeReadOnly", {assert: 'Validate transform.type on baseVal is read-only'} );
test( testBaseValTransformMatrixReadOnly, "patternTransformBaseValTransformMatrixReadOnly", {assert: 'Validate transform.matrix on baseVal is read-only'} );
test( testBaseValTransformAngleReadOnly, "patternTransformBaseValTransformAngleReadOnly", {assert: 'Validate transform.angle on baseVal is read-only'} );


function initTransform() {
	
	tList = document.getElementById("greenRects").patternTransform;
	
	tList.baseVal.clear();
	tList.baseVal.initialize( document.getElementById("svg").createSVGTransform() );
	tList.baseVal.appendItem( document.getElementById("svg").createSVGTransform() );
	tList.baseVal.appendItem( document.getElementById("svg").createSVGTransform() );
}

