add_start_callback(initTransform);
add_result_callback(initTransform);

test( testAnimValDistinct, "transformAnimValDistinct", {assert: 'animVal is a distinctly different object than baseVal'} );
test( testAnimValReadOnly, "transformAnimValReadOnly", {assert: 'animVal is read-only'} );
test( testAnimValNumberOfItemsReadOnly, "transformAnimValNumberOfItemsReadOnly", {assert: 'animVal.numberOfItems is read-only'} );    
test( testAnimValInitialize, "transformAnimValInitialize", {assert: 'animVal.initialize() throws NO_MODIFICATION_ALLOWED_ERR'} );    
test( testAnimValClear, "transformAnimValClear", {assert: 'animVal.clear() throws NO_MODIFICATION_ALLOWED_ERR'} );    
test( testAnimValInsertItemBefore, "transformAnimValInsertItemBefore", {assert: 'animVal.transformAnimValInsertItemBefore() throws NO_MODIFICATION_ALLOWED_ERR'} );    
test( testAnimValReplaceItem, "transformAnimValReplaceItem", {assert: 'animVal.transformAnimValReplaceItem() throws NO_MODIFICATION_ALLOWED_ERR'} );   
test( testAnimValAppendItem, "transformAnimValAppendItem", {assert: 'animVal.transformAnimValAppendItem() throws NO_MODIFICATION_ALLOWED_ERR'} );   
test( testAnimValRemoveItem, "transformAnimValRemoveItem", {assert: 'animVal.transformAnimValRemoveItem() throws NO_MODIFICATION_ALLOWED_ERR'} );   
test( testAnimValCreateSVGTransformFromMatrix, "transformAnimValCreateSVGTransformFromMatrix", {assert: 'animVal.createSVGTransformFromMatrix() creates a new SVGTransform object'} );    
test( testAnimValConsolidate, "transformAnimValConsolidate", {assert: 'Validate animVal.transformAnimValConsolidate() throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValTransformTypeReadOnly, "transformAnimValTransformTypeReadOnly", {assert: 'Validate transform.type on animVal is read-only'} );
test( testAnimValTransformMatrixReadOnly, "transformAnimValTransformMatrixReadOnly", {assert: 'Validate transform.matrix on animVal is read-only'} );
test( testAnimValTransformAngleReadOnly, "transformAnimValTransformAngleReadOnly", {assert: 'Validate transform.angle on animVal is read-only'} );
test( testAnimValSetMatrix, "transformAnimValSetMatrix", {assert: 'Validate transform.setMatrix on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetTranslate, "transformAnimValSetTranslate", {assert: 'Validate transform.setTranslate on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetRotate, "transformAnimValSetRotate", {assert: 'Validate transform.setRotate on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetScale, "transformAnimValSetScale", {assert: 'Validate transform.setScale on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetSkewX, "transformAnimValSetSkewX", {assert: 'Validate transform.setSkewX on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testAnimValSetSkewY, "transformAnimValSetSkewY", {assert: 'Validate transform.setSkewY on animVal throws NO_MODIFICATION_ALLOWED_ERR'} );
test( testBaseValReadOnly, "transformBaseValReadOnly", {assert: 'baseVal is read-only'} );    
test( testBaseValNumberOfItemsReadOnly, "transformBaseValNumberOfItemsReadOnly", {assert: 'baseVal.numberOfItems is read-only'} );    
test( testBaseValInitialize, "transformBaseValInitialize", {assert: 'baseVal.initialize() clears and inserts a new item'} );    
test( testBaseValInsertItemBefore, "transformBaseValInsertItemBefore", {assert: 'baseVal.insertItemBefore() inserts item before specified index'} );    
test( testBaseValReplaceItem, "transformBaseValReplaceItem", {assert: 'baseVal.replaceItem() replaces the item at the specified index'} );    
test( testBaseValAppendItem, "transformBaseValAppendItem", {assert: 'baseVal.appendItem() appends item to the list'} );
test( testBaseValRemoveItem, "transformBaseValRemoveItem", {assert: 'baseVal.removeItem() removes item from the list'} );
test( testBaseValCreateSVGTransformFromMatrix, "transformBaseValCreateSVGTransformFromMatrix", {assert: 'baseVal.createSVGTransformFromMatrix creates a new SVGTransform object'} );    
test( testBaseValConsolidate, "transformBaseValConsolidate", {assert: 'baseVal.consolidate() consolidates the list into a single transfrom'} );
test( testBaseValConsolidateEmptyList, "transformBaseValConsolidateEmptyList", {assert: 'baseVal.consolidate() on an empty list returns null'} );
test( testBaseValClear, "transformBaseValClear", {assert: 'baseVal.clear() clears all transforms'} );   
test( testBaseValInitializeInvalid, "transformBaseValInitializeInvalid", {assert: 'baseVal.initialize() throws exception when passed an arg not in SVGTransform'} );     
test( testBaseValGetItemInvalid, "transformBaseValGetItemInvalid", {assert: 'baseVal.getItem() handles invalid arguments correctly'} );     
test( testBaseValInsertItemBeforeInvalid, "transformBaseValInsertItemBeforeInvalid", {assert: 'baseVal.insertItemBefore() handles invalid arguments correctly'} );    
test( testBaseValReplaceItemInvalid, "transformBaseValReplaceItemInvalid", {assert: 'baseVal.replaceItem() handles invalid arguments correctly'} );    
test( testBaseValAppendItemInvalid, "transformBaseValAppendItemInvalid", {assert: 'baseVal.appendItem() handles invalid arguments correctly'} );    
test( testBaseValRemoveItemInvalid, "transformBaseValRemoveItemInvalid", {assert: 'baseVal.removeItem() handles invalid arguments correctly'} );  
test( testBaseValCreateSVGTransformFromMatrixInvalid, "transformBaseValCreateSVGTransformFromMatrixInvalid", {assert: 'baseVal.createSVGTransformFromMatrix handles invalid arguments correctly'} ); 
test( testBaseValTransformTypeReadOnly, "transformBaseValTransformTypeReadOnly", {assert: 'Validate transform.type on baseVal is read-only'} );
test( testBaseValTransformMatrixReadOnly, "transformBaseValTransformMatrixReadOnly", {assert: 'Validate transform.matrix on baseVal is read-only'} );
test( testBaseValTransformAngleReadOnly, "transformBaseValTransformAngleReadOnly", {assert: 'Validate transform.angle on baseVal is read-only'} );




function initTransform() {
	
	tList = document.getElementById("testRect").transform;
	
	tList.baseVal.clear();
	tList.baseVal.initialize( document.getElementById("svg").createSVGTransform() );
	tList.baseVal.appendItem( document.getElementById("svg").createSVGTransform() );
	tList.baseVal.appendItem( document.getElementById("svg").createSVGTransform() );
}

