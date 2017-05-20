add_start_callback(initTransform);
add_result_callback(initTransform);

test( testAnimValUpdatedAfterModification, "patternTransformAnimValUpdatedAfterModification", {assert: 'animVal list is updated when baseVal is cleared'} );  
test( testListItemUpdated, "patternTransformListItemUpdated", {assert: 'List is updated immediately when modifications are made to list items'} );  
test( testInitializeRemovesItem, "patternTransformInitializeRemovesItem", {assert: 'If the item used to initialize a list is previously on a list, it is removed its previous list'} );  
test( testInsertItemBeforeRemovesItem, "patternTransformInsertItemBeforeRemovesItem", {assert: 'If the item being inserted is previously on a list, it is removed its previous list'} );  
test( testInsertItemBeforeAlreadyOnList, "patternTransformInsertItemBeforeAlreadyOnList", {assert: 'If the item being inserted is already on that list, it is removed from the list before being inserted'} );  
test( testReplaceItemRemovesItem, "patternTransformReplaceItemRemovesItem", {assert: 'If the replacement item being inserted is previously on a list, it is removed its previous list'} );  
test( testReplaceItemAlreadyOnList, "patternTransformReplaceItemAlreadyOnList", {assert: 'If the replacement item being inserted is already on that list, it is removed from the list before the replacement'} );  
test( testAppendItemRemovesItem, "patternTransformAppendItemRemovesItem", {assert: 'If the item being appended is previously on a list, it is removed its previous list before being appended'} );  
test( testAppendItemAlreadyOnList, "patternTransformAppendItemAlreadyOnList", {assert: 'If the item being appended is already on that list, it is removed from the list before being appended'} );  
test( testCreateTransformFromMatrix, "patternTransformCreateTransformFromMatrix", {assert: 'Matrix is created with transform type = SVG_TRANSFORM_MATRIX and the values from matrix parameter are copied'} );  
test( testSetMatrix, "patternTransformSetMatrix", {assert: 'setMatrix() modifications update the list correctly'} );  
test( testSetTranslate, "patternTransformSetTranslate", {assert: 'setTranslate() modifications update the list correctly'} );  
test( testSetRotate, "patternTransformSetRotate", {assert: 'setRotate() modifications update the list correctly'} );  
test( testSetScale, "patternTransformSetScale", {assert: 'setScale() modifications update the list correctly'} );  
test( testSetSkewX, "patternTransformSetSkewX", {assert: 'setSkewX() modifications update the list correctly'} );  
test( testSetSkewY, "patternTransformSetSkewY", {assert: 'setSkewY() modifications update the list correctly'} );  
test( testConsolidateAllTypes, "patternTransformConsolidateAllTypes", {assert: 'All types of transforms can be consolidated into a single matrix'} );  


function initTransform() {
	
	// Initialize tList1 with 3 matrix (default) transforms
	tList1 = document.getElementById("greenRects1").patternTransform;
	tList1.baseVal.clear();
	tList1.baseVal.initialize( document.getElementById("svg").createSVGTransform() );
	tList1.baseVal.appendItem( document.getElementById("svg").createSVGTransform() );
	tList1.baseVal.appendItem( document.getElementById("svg").createSVGTransform() );
	
	// Initialize tList2 with a rotate, a scale, and a translate transform
	tList2 = document.getElementById("greenRects2").patternTransform;
	tList2.baseVal.clear();
	
	var rotate = document.getElementById("svg").createSVGTransform()
	rotate.setRotate(90,0,0);
	
	var scale = document.getElementById("svg").createSVGTransform()
	scale.setScale(0.5,0.5);
	
	var translate = document.getElementById("svg").createSVGTransform()
	translate.setTranslate(50,50);
	
	tList2.baseVal.initialize(rotate);
	tList2.baseVal.appendItem(scale);
	tList2.baseVal.appendItem(translate);
}
