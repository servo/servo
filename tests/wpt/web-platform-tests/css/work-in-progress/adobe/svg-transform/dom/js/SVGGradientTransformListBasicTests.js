add_start_callback(initTransform);
add_result_callback(initTransform);

test( testAnimValDistinct, "gradientTansformAnimValDistinct", {assert: 'animVal is a distinctly different object than baseVal'} );
test( testAnimValReadOnly, "gradientTransformAnimValReadOnly", {assert: 'animVal is read-only'} );
test( testAnimValNumberOfItemsReadOnly, "gradientTransformAnimValNumberOfItemsReadOnly", {assert: 'animVal.numberOfItems is read-only'} );    
test( testAnimValInitialize, "gradientTransformAnimValInitialize", {assert: 'animVal.initialize() throws NO_MODIFICATION_ALLOWED_ERR'} );    
test( testAnimValClear, "gradientTransformAnimValClear", {assert: 'animVal.clear() throws NO_MODIFICATION_ALLOWED_ERR'} );    
test( testAnimValInsertItemBefore, "gradientTransformAnimValInsertItemBefore", {assert: 'animVal.transformAnimValInsertItemBefore() throws NO_MODIFICATION_ALLOWED_ERR'} );    
test( testAnimValReplaceItem, "gradientTransformAnimValReplaceItem", {assert: 'animVal.transformAnimValReplaceItem() throws NO_MODIFICATION_ALLOWED_ERR'} );   
test( testAnimValAppendItem, "gradientTransformAnimValAppendItem", {assert: 'animVal.transformAnimValAppendItem() throws NO_MODIFICATION_ALLOWED_ERR'} );   
test( testAnimValRemoveItem, "gradientTransformAnimValRemoveItem", {assert: 'animVal.transformAnimValRemoveItem() throws NO_MODIFICATION_ALLOWED_ERR'} );   
test( testAnimValCreateSVGTransformFromMatrix, "gradientTransformAnimValCreateSVGTransformFromMatrix", {assert: 'animVal.createSVGTransformFromMatrix() creates a new SVGTransform object'} );    
test( testAnimValConsolidate, "gradientTransformAnimValConsolidate", {assert: 'Validate animVal.transformAnimValConsolidate() throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValTransformTypeReadOnly, "gradientTransformAnimValTransformTypeReadOnly", {assert: 'Validate transform.type on animVal is read-only'} );
test( testAnimValTransformMatrixReadOnly, "gradientTransformAnimValTransformMatrixReadOnly", {assert: 'Validate transform.matrix on animVal is read-only'} );
test( testAnimValTransformAngleReadOnly, "gradientTransformAnimValTransformAngleReadOnly", {assert: 'Validate transform.angle on animVal is read-only'} );
test( testAnimValSetMatrix, "gradientTransformAnimValSetMatrix", {assert: 'Validate transform.setMatrix on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetTranslate, "gradientTransformAnimValSetTranslate", {assert: 'Validate transform.setTranslate on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetRotate, "gradientTransformAnimValSetRotate", {assert: 'Validate transform.setRotate on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetScale, "gradientTransformAnimValSetScale", {assert: 'Validate transform.setScale on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetSkewX, "gradientTransformAnimValSetSkewX", {assert: 'Validate transform.setSkewX on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetSkewY, "gradientTransformAnimValSetSkewY", {assert: 'Validate transform.setSkewY on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testBaseValReadOnly, "gradientTransformBaseValReadOnly", {assert: 'baseVal is read-only'} ); 
test( testBaseValNumberOfItemsReadOnly, "gradientTransformBaseValNumberOfItemsReadOnly", {assert: 'baseVal.numberOfItems is read-only'} );    
test( testBaseValInitialize, "gradientTransformBaseValInitialize", {assert: 'baseVal.initialize() clears and inserts a new item'} );    
test( testBaseValInsertItemBefore, "gradientTransformBaseValInsertItemBefore", {assert: 'baseVal.insertItemBefore() inserts item before specified index'} );    
test( testBaseValReplaceItem, "gradientTransformBaseValReplaceItem", {assert: 'baseVal.replaceItem() replaces the item at the specified index'} );    
test( testBaseValAppendItem, "gradientTransformBaseValAppendItem", {assert: 'baseVal.appendItem() appends item to the list'} );
test( testBaseValRemoveItem, "gradientTransformBaseValRemoveItem", {assert: 'baseVal.removeItem() removes item from the list'} );
test( testBaseValCreateSVGTransformFromMatrix, "gradientTransformBaseValCreateSVGTransformFromMatrix", {assert: 'baseVal.createSVGTransformFromMatrix creates a new SVGTransform object'} );    
test( testBaseValConsolidate, "gradientTransformBaseValConsolidate", {assert: 'baseVal.consolidate() consolidates the list into a single transfrom'} );
test( testBaseValConsolidateEmptyList, "gradientTransformBaseValConsolidateEmptyList", {assert: 'baseVal.consolidate() on an empty list returns null'} );
test( testBaseValClear, "gradientTransformBaseValClear", {assert: 'baseVal.clear() clears all transforms'} );   
test( testBaseValInitializeInvalid, "gradientTransformBaseValInitializeInvalid", {assert: 'baseVal.initialize() throws exception when passed an arg not in SVGTransform'} );     
test( testBaseValGetItemInvalid, "gradientTransformBaseValGetItemInvalid", {assert: 'baseVal.getItem() handles invalid arguments correctly'} );     
test( testBaseValInsertItemBeforeInvalid, "gradientTransformBaseValInsertItemBeforeInvalid", {assert: 'baseVal.insertItemBefore() handles invalid arguments correctly'} );    
test( testBaseValReplaceItemInvalid, "gradientTransformBaseValReplaceItemInvalid", {assert: 'baseVal.replaceItem() handles invalid arguments correctly'} );    
test( testBaseValAppendItemInvalid, "gradientTransformBaseValAppendItemInvalid", {assert: 'baseVal.appendItem() handles invalid arguments correctly'} );    
test( testBaseValRemoveItemInvalid, "gradientTransformBaseValRemoveItemInvalid", {assert: 'baseVal.removeItem() handles invalid arguments correctly'} );  
test( testBaseValCreateSVGTransformFromMatrixInvalid, "gradientTransformBaseValCreateSVGTransformFromMatrixInvalid", {assert: 'baseVal.createSVGTransformFromMatrix handles invalid arguments correctly'} ); 
test( testBaseValTransformTypeReadOnly, "gradientTransformBaseValTransformTypeReadOnly", {assert: 'Validate transform.type on baseVal is read-only'} );
test( testBaseValTransformMatrixReadOnly, "gradientTransformBaseValTransformMatrixReadOnly", {assert: 'Validate transform.matrix on baseVal is read-only'} );
test( testBaseValTransformAngleReadOnly, "gradientTransformBaseValTransformAngleReadOnly", {assert: 'Validate transform.angle on baseVal is read-only'} );


function initTransform() {
	
	tList = document.getElementById("grad").gradientTransform;
	
	tList.baseVal.clear();
	tList.baseVal.initialize( document.getElementById("svg").createSVGTransform() );
	tList.baseVal.appendItem( document.getElementById("svg").createSVGTransform() );
	tList.baseVal.appendItem( document.getElementById("svg").createSVGTransform() );
}

