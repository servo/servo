var tList = "";

function testAnimValDistinct()
{
	assert_not_equals(tList.animVal, tList.baseVal, "animVal should be distinctly different than baseVal");
}

function testAnimValReadOnly()
{
	assert_readonly(tList, "animVal", "animVal should be read-only");
}

function testAnimValNumberOfItemsReadOnly()
{
	assert_readonly(tList.animVal, "numberOfItems", "animVal.numberOfItems should be read-only");
}

function testAnimValInitialize()
{
	var t = document.getElementById("svg").createSVGTransform();
	assert_throws("NoModificationAllowedError", function(){tList.animVal.initialize(t)}, "animVal.initialize() should throw NO_MODIFICATION_ALLOWED_ERR");
}

function testAnimValClear()
{
	assert_throws("NoModificationAllowedError", function(){tList.animVal.clear()}, "animVal.clear() should throw NO_MODIFICATION_ALLOWED_ERR");
}

function testAnimValInsertItemBefore()
{
	var t = document.getElementById("svg").createSVGTransform();
	assert_throws("NoModificationAllowedError", function(){tList.animVal.insertItemBefore(t, 0)}, "animVal.insertItemBefore should throw NO_MODIFICATION_ALLOWED_ERR");
}

function testAnimValReplaceItem()
{
	var t = document.getElementById("svg").createSVGTransform();
	assert_throws("NoModificationAllowedError", function(){tList.animVal.replaceItem(t, 0)}, "animVal.replaceItem should throw NO_MODIFICATION_ALLOWED_ERR");
}

function testAnimValAppendItem()
{
	var t = document.getElementById("svg").createSVGTransform();
	assert_throws("NoModificationAllowedError", function(){tList.animVal.appendItem(t)}, "animVal.appendItem should throw NO_MODIFICATION_ALLOWED_ERR");
}

function testAnimValRemoveItem()
{
	assert_throws("NoModificationAllowedError", function(){tList.animVal.removeItem(0)}, "animVal.removeItem should throw NO_MODIFICATION_ALLOWED_ERR");
}

function testAnimValCreateSVGTransformFromMatrix()
{

	var matrix = document.getElementById("svg").createSVGTransform().matrix;
	var t = tList.animVal.createSVGTransformFromMatrix(matrix);

	assert_equals(matrixToString(t.matrix), matrixToString(matrix), "animVal.createSVGTransformFromMatrix should succeed");
}

function testAnimValConsolidate()
{
	assert_throws("NoModificationAllowedError", function(){tList.animVal.consolidate()}, "animVal.consolidate should throw NO_MODIFICATION_ALLOWED_ERR");
}

function testAnimValTransformTypeReadOnly() {

	for(var i = 0; i < tList.animVal.numberOfItems; i++)
	{
		assert_readonly(tList.animVal.getItem(i), "type", "transform.type should be read-only");
	}
}

function testAnimValTransformMatrixReadOnly() {

	for(var i = 0; i < tList.animVal.numberOfItems; i++)
	{
		assert_readonly(tList.animVal.getItem(i), "matrix", "transform.matrix should be read-only");
	}
}

function testAnimValTransformAngleReadOnly() {

	for(var i = 0; i < tList.animVal.numberOfItems; i++)
	{
		assert_readonly(tList.animVal.getItem(i), "angle", "transform.angle should be read-only");
	}
}

function testAnimValSetMatrix() {

	var m = document.getElementById("svg").createSVGTransform().matrix;
	for(var i = 0; i < tList.animVal.numberOfItems; i++)
	{
		assert_throws("NoModificationAllowedError", function(){tList.animVal.getItem(i).setMatrix(m)}, "transform.setMatrix on animVal should throw NO_MODIFICATION_ALLOWED_ERR");
	}
}

function testAnimValSetTranslate() {

	for(var i = 0; i < tList.animVal.numberOfItems; i++)
	{
		// assert_throws({name: 'NO_MODIFICATION_ALLOWED_ERR'}, function(){tList.animVal.getItem(i).setTranslate(50,50)}, "transform.setTranslate on animVal should throw NO_MODIFICATION_ALLOWED_ERR");
		var typeBefore = tList.animVal.getItem(i).type;
		tList.animVal.getItem(i).setTranslate(50,50);
		var typeAfter = tList.animVal.getItem(i).type;

		assert_equals(typeAfter, typeBefore, "the transform list for animVal should not be modified");
	}
}

function testAnimValSetRotate() {

	for(var i = 0; i < tList.animVal.numberOfItems; i++)
	{
		// assert_throws({name: 'NO_MODIFICATION_ALLOWED_ERR'}, function(){tList.animVal.getItem(i).setRotate(90,50,50)}, "transform.setRotate on animVal should throw NO_MODIFICATION_ALLOWED_ERR");
		var typeBefore = tList.animVal.getItem(i).type;
		tList.animVal.getItem(i).setRotate(90,50,50);
		var typeAfter = tList.animVal.getItem(i).type;

		assert_equals(typeAfter, typeBefore, "the transform list for animVal should not be modified");
	}
}

function testAnimValSetScale() {

	for(var i = 0; i < tList.animVal.numberOfItems; i++)
	{
		// assert_throws({name: 'NO_MODIFICATION_ALLOWED_ERR'}, function(){tList.animVal.getItem(i).setScale(0.5,0.5)}, "transform.setScale on animVal should throw NO_MODIFICATION_ALLOWED_ERR");
		var typeBefore = tList.animVal.getItem(i).type;
		tList.animVal.getItem(i).setScale(0.5,0.5);
		var typeAfter = tList.animVal.getItem(i).type;

		assert_equals(typeAfter, typeBefore, "the transform list for animVal should not be modified");
	}
}

function testAnimValSetSkewX() {

	for(var i = 0; i < tList.animVal.numberOfItems; i++)
	{
		// assert_throws({name: 'NO_MODIFICATION_ALLOWED_ERR'}, function(){tList.animVal.getItem(i).setSkewX(45)}, "transform.setSkewX on animVal should throw NO_MODIFICATION_ALLOWED_ERR");
		var typeBefore = tList.animVal.getItem(i).type;
		tList.animVal.getItem(i).setSkewX(45);
		var typeAfter = tList.animVal.getItem(i).type;

		assert_equals(typeAfter, typeBefore, "the transform list for animVal should not be modified");
	}
}

function testAnimValSetSkewY() {

	for(var i = 0; i < tList.animVal.numberOfItems; i++)
	{
		// assert_throws({name: 'NO_MODIFICATION_ALLOWED_ERR'}, function(){tList.animVal.getItem(i).setSkewY(45)}, "transform.setSkewY on animVal should throw NO_MODIFICATION_ALLOWED_ERR");
		var typeBefore = tList.animVal.getItem(i).type;
		tList.animVal.getItem(i).setSkewY(45);
		var typeAfter = tList.animVal.getItem(i).type;

		assert_equals(typeAfter, typeBefore, "the transform list for animVal should not be modified");
	}
}

function testBaseValReadOnly()
{
	assert_readonly(tList, "baseVal", "baseVal should be read-only");
}

function testBaseValNumberOfItemsReadOnly()
{
	assert_readonly(tList.baseVal, "numberOfItems", "baseVal.numberOfItems should be read-only");
}

function testBaseValInitialize()
{
	var t = document.getElementById("svg").createSVGTransform();
	assert_equals(tList.baseVal.initialize(t), t);
}

function testBaseValInsertItemBefore()
{
	var t1 = tList.baseVal.getItem(0);
	var t2 = document.getElementById("svg").createSVGTransform();

	var numItemsBefore = tList.baseVal.numberOfItems;

	var ret = tList.baseVal.insertItemBefore(t2, 0);
	var numItemsAfter = tList.baseVal.numberOfItems;

	assert_equals(ret, t2, "baseVal.insertItemBefore() should return the inserted item");
	assert_true(numItemsAfter == numItemsBefore + 1, "There should be 1 transform added to the list");
	assert_equals(tList.baseVal.getItem(0), t2, "Transform 2 was not inserted before Transform 1");
	assert_equals(tList.baseVal.getItem(1), t1, "Transform 1 is not in the correct position");
}

function testBaseValReplaceItem()
{
	var t1 = tList.baseVal.getItem(0);
	var t2 = document.getElementById("svg").createSVGTransform();

	var numItemsBefore = tList.baseVal.numberOfItems;

	var ret = tList.baseVal.replaceItem(t2, 0);
	var numItemsAfter = tList.baseVal.numberOfItems;

	assert_equals(ret, t2, "baseVal.replaceItem() should return the inserted item");
	assert_equals(numItemsAfter, numItemsBefore, "The list size should remain the same");
	assert_equals(tList.baseVal.getItem(0), t2, "Transform 2 did not replace Transform 1");
}

function testBaseValAppendItem()
{
	var t = document.getElementById("svg").createSVGTransform();

	var numItemsBefore = tList.baseVal.numberOfItems;

	var ret = tList.baseVal.appendItem(t);
	var numItemsAfter = tList.baseVal.numberOfItems;

	assert_equals(ret, t, "baseVal.appendItem() should return the inserted item");
	assert_true(numItemsAfter == numItemsBefore + 1, "There should be 1 transform added to the list");
	assert_equals(tList.baseVal.getItem(numItemsAfter-1), t, "Transform was not appended");
}

function testBaseValRemoveItem()
{
	var numItemsBefore = tList.baseVal.numberOfItems;
	var itemToRemove = tList.baseVal.getItem(numItemsBefore-1)

	var ret = tList.baseVal.removeItem(numItemsBefore-1);
	var numItemsAfter = tList.baseVal.numberOfItems;

	assert_equals(ret, itemToRemove, "baseVal.removeItem() should return the removed item");
	assert_true(numItemsAfter == numItemsBefore - 1, "There should be 1 transform removed from the list");
}

function testBaseValCreateSVGTransformFromMatrix()
{
	var matrix = document.getElementById("svg").createSVGTransform().matrix;
	var t = tList.baseVal.createSVGTransformFromMatrix(matrix);

	assert_equals(matrixToString(t.matrix), matrixToString(matrix), "animVal.createSVGTransformFromMatrix should succeed");
}

function testBaseValConsolidate()
{
	if(tList.baseVal.numberOfItems == 1)
	{
		var t = document.getElementById("svg").createSVGTransform();
		tList.baseVal.appendItem(t);
	}

	tList.baseVal.consolidate();
	assert_equals(tList.baseVal.numberOfItems, 1, "The list should be consolidated into 1 item");
}

function testBaseValConsolidateEmptyList()
{
	tList.baseVal.clear();
	assert_equals(tList.baseVal.consolidate(), null);
}

function testBaseValClear()
{
	assert_true(tList.baseVal.numberOfItems > 0, "There should be at least 1 transform on the testRect");
	tList.baseVal.clear();
	assert_equals(tList.baseVal.numberOfItems, 0, "There should be an empty transform list after the clear()");
	assert_throws("IndexSizeError", function(){tList.baseVal.getItem(0)}, "baseVal.getItem() on an empty list should throw an error");
}

function testBaseValInitializeInvalid()
{
	assert_throws(new TypeError, function(){tList.baseVal.initialize(null)}, "baseVal.initialize(null) should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.initialize(30)}, "baseVal.initialize(30) should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.initialize("someString")}, "initialize('someString') should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.initialize(new Object())}, "initialize(new Object()) should throw an error");
}

function testBaseValGetItemInvalid()
{
	assert_throws("IndexSizeError", function(){tList.baseVal.getItem(30)}, "baseVal.getItem(30) should throw an error");
	assert_equals(tList.baseVal.getItem(null), tList.baseVal.getItem(0), "baseVal.getItem(null) should return baseVal.getItem(0)");
	assert_equals(tList.baseVal.getItem("someString"), tList.baseVal.getItem(0), "baseVal.getItem('someString') should return baseVal.getItem(0)");
	assert_equals(tList.baseVal.getItem(new Object()), tList.baseVal.getItem(0), "baseVal.getItem(new Object()) should return baseVal.getItem(0)");
}

function testBaseValInsertItemBeforeInvalid()
{
	var t = document.getElementById("svg").createSVGTransform();

	// Specifying an index out of range should result in the item being appended
	var ret = tList.baseVal.insertItemBefore(t, tList.baseVal.numberOfItems+5);
	assert_equals(ret, t, "baseVal.insertItemBefore(t, 30) should return the inserted item");
	assert_equals(tList.baseVal.getItem(3), t, "baseVal.insertItemBefore(t, 30) should append the item to the list");

	assert_throws(new TypeError, function(){tList.baseVal.insertItemBefore(null, 0)}, "baseVal.insertItemBefore(null, 0) should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.insertItemBefore("someString", 0)}, "baseVal.insertItemBefore('someString', 0) should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.insertItemBefore(new Object(), 0)}, "baseVal.insertItemBefore(new Object(), 0) should throw an error");

	// Should an error be thrown if arg2 is invalid? Nothing in spec and no error is thrown so these currently fail as they are.
	/*
	assert_throws(new TypeError, function(){tList.baseVal.insertItemBefore(t, null)}, "baseVal.insertItemBefore(t, null) should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.insertItemBefore(t, "someString")}, "baseVal.insertItemBefore(t, 'someString') should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.insertItemBefore(t, new Object())}, "baseVal.insertItemBefore(t, new Object()) should throw an error");
	*/
}

function testBaseValReplaceItemInvalid()
{
	assert_throws(new TypeError, function(){tList.baseVal.replaceItem(null, 0)}, "baseVal.replaceItem(null, 0) should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.replaceItem("someString", 0)}, "baseVal.replaceItem('someString', 0) should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.replaceItem(new Object(), 0)}, "baseVal.replaceItem(new Object(), 0) should throw an error");

	// Should an error be thrown if arg2 is invalid? Nothing in spec and no error is thrown so these currently fail as they are.
	/*
	var t = document.getElementById("svg").createSVGTransform();
	assert_throws(new TypeError, function(){tList.baseVal.replaceItem(t, null)}, "baseVal.replaceItem(t, null) should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.replaceItem(t, "someString")}, "baseVal.replaceItem(t, 'someString') should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.replaceItem(t, new Object())}, "baseVal.replaceItem(t, new Object()) should throw an error");
	*/
}

function testBaseValAppendItemInvalid()
{
	assert_throws(new TypeError, function(){tList.baseVal.appendItem(null, 0)}, "baseVal.appendItem(null, 0) should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.appendItem("someString", 0)}, "baseVal.appendItem('someString', 0) should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.appendItem(new Object(), 0)}, "baseVal.appendItem(new Object(), 0) should throw an error");

	// Should an error be thrown if arg2 is invalid? Nothing in spec and no error is thrown so these currently fail as they are.
	/*
	var t = document.getElementById("svg").createSVGTransform();
	assert_throws(new TypeError, function(){tList.baseVal.appendItem(t, null)}, "baseVal.appendItem(t, null) should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.appendItem(t, "someString")}, "baseVal.appendItem(t, 'someString') should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.appendItem(t, new Object())}, "baseVal.appendItem(t, new Object()) should throw an error");
	*/
}

function testBaseValRemoveItemInvalid()
{
	assert_throws("IndexSizeError", function(){tList.baseVal.removeItem(10)}, "baseVal.removeItem() should throw an error on invalid index");
	assert_throws("IndexSizeError", function(){tList.baseVal.removeItem(-1)}, "baseVal.removeItem() should throw an error on invalid index");

	// Should an error be thrown if the arg is an invalid type? Nothing in spec and no error is thrown so these currently fail as they are.
	/*
	assert_throws(new TypeError, function(){tList.baseVal.removeItem(null)}, "baseVal.removeItem() should throw an error on invalid index");
	assert_throws(new TypeError, function(){tList.baseVal.removeItem("someString")}, "baseVal.removeItem() should throw an error on invalid index");
	assert_throws(new TypeError, function(){tList.baseVal.removeItem(new Object())}, "baseVal.removeItem() should throw an error on invalid index");
	*/
}

function testBaseValCreateSVGTransformFromMatrixInvalid()
{
	assert_throws(new TypeError, function(){tList.baseVal.createSVGTransformFromMatrix(null)}, "baseVal.createSVGTransformFromMatrix(null) should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.createSVGTransformFromMatrix("someString")}, "baseVal.createSVGTransformFromMatrix('someString', 0) should throw an error");
	assert_throws(new TypeError, function(){tList.baseVal.createSVGTransformFromMatrix(new Object())}, "baseVal.createSVGTransformFromMatrix(new Object(), 0) should throw an error");
}

function testBaseValTransformTypeReadOnly() {

	for(var i = 0; i < tList.baseVal.numberOfItems; i++)
	{
		assert_readonly(tList.baseVal.getItem(i), "type", "transform.type should be read-only");
	}
}

function testBaseValTransformMatrixReadOnly() {

	for(var i = 0; i < tList.baseVal.numberOfItems; i++)
	{
		assert_readonly(tList.baseVal.getItem(i), "matrix", "transform.type should be read-only");
	}
}

function testBaseValTransformAngleReadOnly() {

	for(var i = 0; i < tList.baseVal.numberOfItems; i++)
	{
		assert_readonly(tList.baseVal.getItem(i), "angle", "transform.type should be read-only");
	}
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