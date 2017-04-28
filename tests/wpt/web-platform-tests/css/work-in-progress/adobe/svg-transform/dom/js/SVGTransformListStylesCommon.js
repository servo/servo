tList1 = document.getElementById("testRect1").transform;
tList2 = document.getElementById("testRect2").transform;

function testTransformStyleBasic()
{
	assert_equals(tList1.animVal.numberOfItems, 1, "There should be one transform on testRect1 - animVal");
	assert_equals(tList2.animVal.numberOfItems, 2, "There should be two transforms on testRect2 - animVal");
	assert_equals(tList1.baseVal.numberOfItems, 1, "There should be one transform on testRect1 - baseVal");
	assert_equals(tList2.baseVal.numberOfItems, 2, "There should be two transforms on testRect2 - baseVal");
}

