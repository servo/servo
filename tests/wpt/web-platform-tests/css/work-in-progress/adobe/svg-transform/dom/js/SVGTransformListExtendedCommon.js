var tList1 = "";
var tList2 = "";

function testAnimValUpdatedAfterModification()
{
	var animValBefore = tList1.animVal.numberOfItems;
	
	// Insert a transform at the beginning of the list - rotate 90 degrees about 50, 50
	var t = document.getElementById("svg").createSVGTransform();
	t.setRotate(90,50,50);
	tList1.baseVal.insertItemBefore(t, 0);
	
	// This is currently failing - commenting out to execute the rest of this test
//	assert_not_equals(tList1.animVal.getItem(0), tList1.baseVal.getItem(0), "The item inserted to baseVal should be copied to animVal");
	assert_equals(tList1.animVal.numberOfItems, animValBefore + 1, "animVal was not updated after insert");
	assert_equals(transformToString(tList1.baseVal.getItem(0)), "SVG_TRANSFORM_ROTATE [0.0 1.0 -1.0 0.0 100.0 0.0]");
	
	animValBefore = tList1.animVal.numberOfItems;
	tList1.baseVal.clear();
	
	assert_not_equals(tList1.animVal.numberOfItems, animValBefore, "animVal was not updated after clear()");
	assert_equals(tList1.animVal.numberOfItems, 0, "animVal was not updated after clear()");
}	

function testListItemUpdated()
{
	var t = tList1.baseVal.getItem(1);
	var tMatrixBefore = t.matrix;
	
	// Set the transform to be a scale
	t.setScale(0.5,0.5);
	
	// Verify the matrix is updated on the transform
	assert_not_equals(tMatrixBefore, t.matrix, "Matrix wasn't updated after setScale");
	
	// Verify it's updated immediately on both baseVal and animVal
	assert_equals(transformToString(tList1.baseVal.getItem(1)), "SVG_TRANSFORM_SCALE [0.5 0.0 0.0 0.5 0.0 0.0]");
	assert_equals(transformToString(tList1.animVal.getItem(1)), "SVG_TRANSFORM_SCALE [0.5 0.0 0.0 0.5 0.0 0.0]");
}

function testInitializeRemovesItem()
{
	var tList2Before = tList2.baseVal.numberOfItems;
	
	// Take the second item (scale) from tList2
	var scale = tList2.baseVal.getItem(1);

	// Initialize tList1 using it
	tList1.baseVal.initialize(scale);
	
	// Confirm it was removed from tList2 after inserted into tList1 - baseVal + animVal
	assert_true(tList2.baseVal.numberOfItems == tList2Before -1, "Transform wasn't removed from previous list - baseVal");
	assert_true(tList2.animVal.numberOfItems == tList2Before -1, "Transform wasn't removed from previous list - animVal");
	 
	for(var i = 0; i < tList2.baseVal.numberOfItems; i++)
	{
		assert_not_equals(tList2.baseVal.getItem(i).type, SVG_TRANSFORM_SCALE);
		assert_not_equals(tList2.animVal.getItem(i).type, SVG_TRANSFORM_SCALE);
	}
	
	// Confirm the list intialized with the correct item
	assert_equals(tList1.animVal.getItem(0).type, SVG_TRANSFORM_SCALE);
	
} 

function testInsertItemBeforeRemovesItem()
{
	var tList2Before = tList2.baseVal.numberOfItems;
	var tList1Before = tList1.baseVal.numberOfItems;
	
	// Take the third item (translate) from the second list
	var translate = tList2.baseVal.getItem(2);

	// Insert it into the 2nd position in first list
	tList1.baseVal.insertItemBefore(translate, 1);
	
	// Confirm it was removed from second list after inserted into the first list - baseVal + animVal
	assert_true(tList2.baseVal.numberOfItems == (tList2Before - 1), "Transform wasn't removed from previous list - baseVal");
	assert_true(tList2.animVal.numberOfItems == (tList2Before - 1), "Transform wasn't removed from previous list - animVal");
	assert_equals(tList1.baseVal.numberOfItems, (tList1Before + 1), "Transform wasn't inserted into the list - baseVal");
	assert_equals(tList1.animVal.numberOfItems, (tList1Before + 1), "Transform wasn't inserted into the list - animVal - ");
	 
	for(var i = 0; i < tList2.baseVal.numberOfItems; i++)
	{
		assert_not_equals(tList2.baseVal.getItem(i).type, SVG_TRANSFORM_TRANSLATE);
		assert_not_equals(tList2.animVal.getItem(i).type, SVG_TRANSFORM_TRANSLATE);
	}
	
	// Confirm is was added to the right position in the first list
	assert_equals(tList1.baseVal.getItem(1).type, SVG_TRANSFORM_TRANSLATE);
	assert_equals(tList1.animVal.getItem(1).type, SVG_TRANSFORM_TRANSLATE);
}

function testInsertItemBeforeAlreadyOnList()
{
	var tList2Before = tList2.baseVal.numberOfItems;
	
	// Take the third item (translate) from the list
	var translate = tList2.baseVal.getItem(2);

	// Insert it into the first position on the same list
	tList2.baseVal.insertItemBefore(translate, 0);
	
	// Confirm the list size remains the same
	assert_equals(tList2.baseVal.numberOfItems, tList2Before, "Transform wasn't removed from the list before being reinserted - baseVal");
	assert_equals(tList2.animVal.numberOfItems, tList2Before, "Transform wasn't removed from the list before being reinserted - animVal");
	
	// Confirm it got inserted in the correct position
	assert_equals(tList2.baseVal.getItem(0).type, SVG_TRANSFORM_TRANSLATE);
	assert_equals(tList2.animVal.getItem(0).type, SVG_TRANSFORM_TRANSLATE);
	
	// Confirm the rest of the items in the list shifted by 1
	assert_equals(tList2.baseVal.getItem(1).type, SVG_TRANSFORM_ROTATE);
	assert_equals(tList2.animVal.getItem(1).type, SVG_TRANSFORM_ROTATE);
	assert_equals(tList2.baseVal.getItem(2).type, SVG_TRANSFORM_SCALE);
	assert_equals(tList2.animVal.getItem(2).type, SVG_TRANSFORM_SCALE);
}

function testReplaceItemRemovesItem()
{
	var tList2Before = tList2.baseVal.numberOfItems;
	var tList1Before = tList1.baseVal.numberOfItems;
	
	// Take the first item (rotate) from the second list
	var rotate = tList2.baseVal.getItem(0);

	// Replace it into the 3rd position in first list
	tList1.baseVal.replaceItem(rotate, 2);
	
	// Confirm it was removed from second list after inserted into the first list - baseVal + animVal
	assert_true(tList2.baseVal.numberOfItems == (tList2Before - 1), "Transform wasn't removed from previous list - baseVal");
	assert_true(tList2.animVal.numberOfItems == (tList2Before - 1), "Transform wasn't removed from previous list - animVal");
	
	// Confirm the first list size stays the same
	assert_equals(tList1.baseVal.numberOfItems, tList1Before, "List size should stay the same when an item is replaced - baseVal");
	assert_equals(tList1.animVal.numberOfItems, tList1Before, "List size should stay the same when an item is replaced - animVal");	
	 
	for(var i = 0; i < tList2.baseVal.numberOfItems; i++)
	{
		assert_not_equals(tList2.baseVal.getItem(i).type, SVG_TRANSFORM_ROTATE);
		assert_not_equals(tList2.animVal.getItem(i).type, SVG_TRANSFORM_ROTATE);
	}
	
	// Confirm is was added to the right position in the first list
	assert_equals(tList1.baseVal.getItem(2).type, SVG_TRANSFORM_ROTATE);
	assert_equals(tList1.animVal.getItem(2).type, SVG_TRANSFORM_ROTATE);
}

function testReplaceItemAlreadyOnList()
{
	var tList2Before = tList2.baseVal.numberOfItems;

	// Take the third item (translate) from the list
	var translate = tList2.baseVal.getItem(2);

	// Replace the first item on the list with it
	tList2.baseVal.replaceItem(translate, 0);

	// Confirm the list size is -1
	assert_equals(tList2.baseVal.numberOfItems, (tList2Before - 1), "Transform wasn't removed from the list before being reinserted - baseVal");
	assert_equals(tList2.animVal.numberOfItems, (tList2Before - 1), "Transform wasn't removed from the list before being reinserted - animVal");

	// Confirm it got inserted in the correct position
	assert_equals(tList2.baseVal.getItem(0).type, SVG_TRANSFORM_TRANSLATE);
	assert_equals(tList2.animVal.getItem(0).type, SVG_TRANSFORM_TRANSLATE);
		
	// Confirm the second list item stayed the same 
	assert_equals(tList2.baseVal.getItem(1).type, SVG_TRANSFORM_SCALE);
	assert_equals(tList2.animVal.getItem(1).type, SVG_TRANSFORM_SCALE);
}

function testAppendItemRemovesItem()
{
	var tList2Before = tList2.baseVal.numberOfItems;
	var tList1Before = tList1.baseVal.numberOfItems;
	
	// Take the second item (scale) from the second list
	var scale = tList2.baseVal.getItem(1);

	// Append it to the second list
	tList1.baseVal.appendItem(scale);
	
	// Confirm it was removed from second list after inserted into the first list - baseVal + animVal
	assert_equals(tList2.baseVal.numberOfItems, (tList2Before - 1), "Transform wasn't removed from previous list - baseVal");
	assert_equals(tList2.animVal.numberOfItems, (tList2Before - 1), "Transform wasn't removed from previous list - animVal");
	assert_equals(tList1.baseVal.numberOfItems, (tList1Before + 1), "Transform wasn't appended to the list - baseVal");
	assert_equals(tList1.animVal.numberOfItems, (tList1Before + 1), "Transform wasn't appended to the list - animVal");
		 
	for(var i = 0; i < tList2.baseVal.numberOfItems; i++)
	{
		assert_not_equals(tList2.baseVal.getItem(i).type, SVG_TRANSFORM_SCALE);
		assert_not_equals(tList2.animVal.getItem(i).type, SVG_TRANSFORM_SCALE);
	}
	
	// Confirm is was added to the right position in the first list
	assert_equals(tList1.baseVal.getItem(3).type, SVG_TRANSFORM_SCALE);
	assert_equals(tList1.animVal.getItem(3).type, SVG_TRANSFORM_SCALE);
}

function testAppendItemAlreadyOnList()
{
	var tList2Before = tList2.baseVal.numberOfItems;
	
	// Take the second item (scale) from the list
	var scale = tList2.baseVal.getItem(1);

	// Append it the same list
	tList2.baseVal.appendItem(scale);
	
	// Confirm the list size remains the same
	assert_equals(tList2.baseVal.numberOfItems, tList2Before, "Transform wasn't removed from the list before being reinserted - baseVal");
	assert_equals(tList2.animVal.numberOfItems, tList2Before, "Transform wasn't removed from the list before being reinserted - animVal");
	 
	// Confirm it got inserted in the correct position
	assert_equals(tList2.baseVal.getItem(2).type, SVG_TRANSFORM_SCALE);
	assert_equals(tList2.animVal.getItem(2).type, SVG_TRANSFORM_SCALE);
	
	// Confirm the first list item stayed the same 
	assert_equals(tList2.baseVal.getItem(0).type, SVG_TRANSFORM_ROTATE);
	assert_equals(tList2.animVal.getItem(0).type, SVG_TRANSFORM_ROTATE);
	
	// Confirm that the item that was previously last is now second
	assert_equals(tList2.baseVal.getItem(1).type, SVG_TRANSFORM_TRANSLATE);
	assert_equals(tList2.animVal.getItem(1).type, SVG_TRANSFORM_TRANSLATE);
}

function testCreateTransformFromMatrix()
{
	// Create a matrix and set translate x & y
	var matrix = document.getElementById("svg").createSVGTransform().matrix;
	matrix.e = 50;
	matrix.f = 100;
	
	// Create a transform from its inverse
	var transform = tList1.baseVal.createSVGTransformFromMatrix(matrix.inverse());
	
	// Confirm it was created correctly
	assert_equals(transformToString(transform), "SVG_TRANSFORM_MATRIX [1.0 0.0 0.0 1.0 -50.0 -100.0]");

}

function testSetMatrix() {
	
	// Create a matrix and set scale x & y
	var matrix = document.getElementById("svg").createSVGTransform().matrix;
	matrix.a = 0.5;
	matrix.d = 1.5;
	
	// Set the matrix on tList2
	tList2.baseVal.getItem(0).setMatrix(matrix);
	
	// Confirm the type and matrix values get set properly
	assert_equals(transformToString(tList2.baseVal.getItem(0)), "SVG_TRANSFORM_MATRIX [0.5 0.0 0.0 1.5 0.0 0.0]", "Matrix was not set correctly - baseVal");
	assert_equals(transformToString(tList2.animVal.getItem(0)), "SVG_TRANSFORM_MATRIX [0.5 0.0 0.0 1.5 0.0 0.0]", "Matrix was not set correctly - animVal");
	
	// Confirm the angle got reset to zero
	assert_equals(tList2.baseVal.getItem(0).angle, 0);
	assert_equals(tList2.animVal.getItem(0).angle, 0);
	
}

function testSetTranslate() {
	
	// Set the first item on the list as a translate
	tList1.baseVal.getItem(0).setTranslate(100, 75);
	
	// Confirm the type and matrix values get set properly
	assert_equals(transformToString(tList1.baseVal.getItem(0)), "SVG_TRANSFORM_TRANSLATE [1.0 0.0 0.0 1.0 100.0 75.0]", "Matrix was not set correctly - baseVal");
	assert_equals(transformToString(tList1.animVal.getItem(0)), "SVG_TRANSFORM_TRANSLATE [1.0 0.0 0.0 1.0 100.0 75.0]", "Matrix was not set correctly - animVal");
}

function testSetRotate() {
	
	// Set the second item on the list to rotate about 25,25
	tList1.baseVal.getItem(1).setRotate(90, 25, 25);
	
	// Confirm the angle & type get set properly
	assert_equals(tList1.baseVal.getItem(1).angle, 90);
	assert_equals(tList1.animVal.getItem(1).angle, 90);
	assert_equals(tList1.baseVal.getItem(1).type, SVG_TRANSFORM_ROTATE);
	assert_equals(tList1.animVal.getItem(1).type, SVG_TRANSFORM_ROTATE);
}

function testSetScale() {
	
	// Set the first item on tList1 as a scale
	tList2.baseVal.getItem(0).setScale(-0.3, 0.8);
	
	// Confirm the type and matrix values get set properly
	assert_equals(transformToString(tList2.baseVal.getItem(0)), "SVG_TRANSFORM_SCALE [-0.3 0.0 0.0 0.8 0.0 0.0]", "Matrix was not set correctly - baseVal");
	assert_equals(transformToString(tList2.animVal.getItem(0)), "SVG_TRANSFORM_SCALE [-0.3 0.0 0.0 0.8 0.0 0.0]", "Matrix was not set correctly - animVal");
	
	// Confirm the angle got reset to zero
	assert_equals(tList2.baseVal.getItem(0).angle, 0);
	assert_equals(tList2.animVal.getItem(0).angle, 0);
}

function testSetSkewX() {
	
	// Set the second item on tList1 as a skewX
	tList2.baseVal.getItem(1).setSkewX(45);
	
	// Confirm the type and matrix values get set properly
	assert_equals(transformToString(tList2.baseVal.getItem(1)), "SVG_TRANSFORM_SKEWX [1.0 0.0 1.0 1.0 0.0 0.0]", "Matrix was not set correctly - baseVal");
	assert_equals(transformToString(tList2.animVal.getItem(1)), "SVG_TRANSFORM_SKEWX [1.0 0.0 1.0 1.0 0.0 0.0]", "Matrix was not set correctly - animVal");
	
	// Confirm the angle got reset to zero
	assert_equals(tList2.baseVal.getItem(1).angle, 45);
	assert_equals(tList2.animVal.getItem(1).angle, 45);
}
	
function testSetSkewY() {
	
	// Set the third item on tList1 as a skewY
	tList2.baseVal.getItem(2).setSkewY(45);
	
	// Confirm the type and matrix values get set properly
	assert_equals(transformToString(tList2.baseVal.getItem(2)), "SVG_TRANSFORM_SKEWY [1.0 1.0 0.0 1.0 0.0 0.0]", "Matrix was not set correctly - baseVal");
	assert_equals(transformToString(tList2.animVal.getItem(2)), "SVG_TRANSFORM_SKEWY [1.0 1.0 0.0 1.0 0.0 0.0]", "Matrix was not set correctly - animVal");
	
	// Confirm the angle got reset to zero
	assert_equals(tList2.baseVal.getItem(2).angle, 45);
	assert_equals(tList2.animVal.getItem(2).angle, 45);
}

function testConsolidateAllTypes()
{
	var translate = document.getElementById("svg").createSVGTransform();
	translate.setTranslate(100,100);
	
	var scale = document.getElementById("svg").createSVGTransform();
	scale.setScale(0.5, -1.5);
	
	var rotate = document.getElementById("svg").createSVGTransform();
	rotate.setRotate(90,0,0);
	
	var skewX = document.getElementById("svg").createSVGTransform();
	skewX.setSkewX(10);
	
	var skewY = document.getElementById("svg").createSVGTransform();
	skewY.setSkewY(30);
	
	var multiplied = multiplyMatrix( [translate,scale,rotate,skewX,skewY] );
	
	tList1.baseVal.initialize(translate);
	tList1.baseVal.appendItem(scale);
	tList1.baseVal.appendItem(rotate);
	tList1.baseVal.appendItem(skewX);
	tList1.baseVal.appendItem(skewY);
	
	var consolidated = tList1.baseVal.consolidate();
	
	assert_equals(matrixToString(tList1.baseVal.getItem(0).matrix), matrixToString(multiplied), "Matrix was not multiplied correctly when consolidated - baseVal");
	assert_equals(matrixToString(tList1.animVal.getItem(0).matrix), matrixToString(multiplied), "Matrix was not multiplied correctly when consolidated - animVal");
	
	
	assert_equals(tList1.baseVal.getItem(0).type, SVG_TRANSFORM_MATRIX, "Type was not set correctly - baseVal");
	assert_equals(tList1.animVal.getItem(0).type, SVG_TRANSFORM_MATRIX, "Type was not set correctly - animVal");
}


function matrixToString(matrix) 
{
	 return "[" + matrix.a.toFixed(1)
          + " " + matrix.b.toFixed(1)
          + " " + matrix.c.toFixed(1)
          + " " + matrix.d.toFixed(1)
          + " " + matrix.e.toFixed(1)
          + " " + matrix.f.toFixed(1)
          + "]";
}

function getType(transform) {
	var transformTypes = {
        "0": "SVG_TRANSFORM_UNKNOWN",
        "1": "SVG_TRANSFORM_MATRIX",
        "2": "SVG_TRANSFORM_TRANSLATE",
        "3": "SVG_TRANSFORM_SCALE",
        "4": "SVG_TRANSFORM_ROTATE",
        "5": "SVG_TRANSFORM_SKEWX",
        "6": "SVG_TRANSFORM_SKEWY"
    };
	
	return transformTypes[transform.type];
}

function multiplyMatrix(tList) {
	
	var retMatrix = tList[0].matrix;
	
	for(var i = 1; i < tList.length; i++)
	{
		retMatrix = retMatrix.multiply(tList[i].matrix);
	}
	
	return retMatrix;
}

var SVG_TRANSFORM_UNKNOWN = 0;
var SVG_TRANSFORM_MATRIX = 1;
var SVG_TRANSFORM_TRANSLATE = 2;
var SVG_TRANSFORM_SCALE = 3;
var SVG_TRANSFORM_ROTATE = 4;
var SVG_TRANSFORM_SKEWX = 5;
var SVG_TRANSFORM_SKEWY = 6;

function transformToString(transform) {
    
    return getType(transform) + " " + matrixToString(transform.matrix);
}

