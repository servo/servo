add_start_callback(initTransform);
add_result_callback(initTransform);

test( testAnimValUpdatedAfterModification, "transformAnimValUpdatedAfterModification", {assert: 'animVal list is updated when baseVal is cleared'} );  
test( testListItemUpdated, "transformListItemUpdated", {assert: 'List is updated immediately when modifications are made to list items'} );  
test( testInitializeRemovesItem, "transformInitializeRemovesItem", {assert: 'If the item used to initialize a list is previously on a list, it is removed its previous list'} );  
test( testInsertItemBeforeRemovesItem, "transformInsertItemBeforeRemovesItem", {assert: 'If the item being inserted is previously on a list, it is removed its previous list'} );  
test( testInsertItemBeforeAlreadyOnList, "transformInsertItemBeforeAlreadyOnList", {assert: 'If the item being inserted is already on that list, it is removed from the list before being inserted'} );  
test( testReplaceItemRemovesItem, "transformReplaceItemRemovesItem", {assert: 'If the replacement item being inserted is previously on a list, it is removed its previous list'} );  
test( testReplaceItemAlreadyOnList, "transformReplaceItemAlreadyOnList", {assert: 'If the replacement item being inserted is already on that list, it is removed from the list before the replacement'} );  
test( testAppendItemRemovesItem, "transformAppendItemRemovesItem", {assert: 'If the item being appended is previously on a list, it is removed its previous list before being appended'} );  
test( testAppendItemAlreadyOnList, "transformAppendItemAlreadyOnList", {assert: 'If the item being appended is already on that list, it is removed from the list before being appended'} );  
test( testCreateTransformFromMatrix, "transformCreateTransformFromMatrix", {assert: 'Matrix is created with transform type = SVG_TRANSFORM_MATRIX and the values from matrix parameter are copied'} );  
test( testSetMatrix, "transformSetMatrix", {assert: 'setMatrix() modifications update the list correctly'} );  
test( testSetTranslate, "transformSetTranslate", {assert: 'setTranslate() modifications update the list correctly'} );  
test( testSetRotate, "transformSetRotate", {assert: 'setRotate() modifications update the list correctly'} );  
test( testSetScale, "transformSetScale", {assert: 'setScale() modifications update the list correctly'} );  
test( testSetSkewX, "transformSetSkewX", {assert: 'setSkewX() modifications update the list correctly'} );  
test( testSetSkewY, "transformSetSkewY", {assert: 'setSkewY() modifications update the list correctly'} );  
test( testConsolidateAllTypes, "transformConsolidateAllTypes", {assert: 'All types of transforms can be consolidated into a single matrix'} );  
test( testModifyConsolidated, "transformModifyConsolidated", {assert: 'Modifications can be made to a consolidated matrix'} );  
test( testConsolidateConsolidated, "transformConsolidateConsolidated", {assert: 'Consolidated matrix can be consolidated again'} );  



function initTransform() {
	
	// Initialize tList1 with 3 matrix (default) transforms
	tList1 = document.getElementById("testRect1").transform;
	tList1.baseVal.clear();
	tList1.baseVal.initialize( document.getElementById("svg").createSVGTransform() );
	tList1.baseVal.appendItem( document.getElementById("svg").createSVGTransform() );
	tList1.baseVal.appendItem( document.getElementById("svg").createSVGTransform() );
	
	// Initialize tList2 with a rotate, a scale, and a translate transform
	tList2 = document.getElementById("testRect2").transform;
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

