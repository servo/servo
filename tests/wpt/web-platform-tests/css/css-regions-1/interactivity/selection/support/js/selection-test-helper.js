var testTimeout = 10000;

setup({timeout: testTimeout});

// This block is excecuted if running in WebKit's harness
if (window.testRunner)
{
	testRunner.dumpEditingCallbacks();
	testRunner.dumpAsText(false);
}

var DEFAULT_MOUSE_VALUE = true;

/*
 * Main test function that defines the testharness test() functions
 */
function runSelectionTest(inSelectionTests, useMouse)
{
	var selectionTests = inSelectionTests || [];
	var useMouse = (useMouse === undefined) ? DEFAULT_MOUSE_VALUE : useMouse;

	if( isRegionsEnabled() )
	{
		var selectionTest = async_test("Text was selected", {timeout: testTimeout});
		selectionTest.step( function()
		{
			var endSelect = document.getElementById("end-select");
			endSelect.onmouseup = selectionTest.step_func( function(evt)
			{
				/* Verify something got selected */
				var selectedText = getCurrentSelectedText();
				assert_not_equals(selectedText, "");

				/* Verify the selected text is everything between the start and end points */
				test( function() { verifySelectedText() }, "Selected text is correct" );

				/* Check for specific things in the selection */
				for(var i=0; i < selectionTests.length; i++)
				{
					if( selectionTests[i].nodeName )
					{
						var nodeName = selectionTests[i].nodeName
						var nodeExp = selectionTests[i].expected;
						var msg = nodeName + " is " + (nodeExp == true ? "" : "not ") + "in selection";
						test( function(){ assert_equals(isNodeInSelection(nodeName), nodeExp) }, msg);
					}
					else if( selectionTests[i].string )
					{
						var strToCheck = selectionTests[i].string;
						var strExp = selectionTests[i].expected;
						var msg = "'"+strToCheck+ "' is " + (strExp == true ? "" : "not ") + "in selection";
						test( function(){ assert_equals(isStringInSelection(strToCheck), strExp) }, msg);
					}
				}

				// Update the message stating the tests are complete
				var msg = document.getElementById("msg");

				var complete = document.createElement("p");
				complete.innerHTML = "Tests are complete. All results in the Details section below should PASS.";
				complete.style.color = "blue";
				msg.appendChild(complete);

				selectionTest.done();
			});

		  	setSelection("start-select", "end-select", useMouse);
		});
	}
	else
	{
		test( function(){ assert_true(false) }, "Regions are not enabled");
	}
}

/*
 * Set the document selection in an automated way
 * If running in Webkit's testRunner, uses internal WebKit APIs to simulate mouse movement.
 * Has option to bypass the mouse movement and set the selection range object directly
 * If not running in Webkit, the function exits, leaving the selection to be done manually.
 */
function setSelection(start, end, useMouse)
{
	if(window.testRunner)
	{
		// This block is executed if running in the Webkit harness
		var startNode = document.getElementById(start);
		var endNode = document.getElementById(end);

		var xStartPosition = startNode.getBoundingClientRect().left
		var yStartPosition = startNode.getBoundingClientRect().top

		var tmp = startNode.getBoundingClientRect();

		var xEndPosition = endNode.getBoundingClientRect().left
		var yEndPosition = endNode.getBoundingClientRect().top

		if( isTopToBottom(startNode, endNode) )
		{
			xEndPosition += endNode.getBoundingClientRect().width
		}
		else
		{
			xStartPosition += startNode.getBoundingClientRect().width
		}

		if(useMouse)
		{
			console.log("Selection set with the mouse");
			eventSender.mouseMoveTo(xStartPosition, yStartPosition);
			eventSender.mouseDown();

			eventSender.mouseMoveTo(xEndPosition, yEndPosition);
			eventSender.mouseUp();

			// Need to manually dispatch this event - it doesn't get
			// sent otherwise when running in testRunner
			var mouseUpEvt = document.createEvent('MouseEvents');
			mouseUpEvt.initMouseEvent(	'mouseup',true,true,window,1,0,0,
										xEndPosition,yEndPosition,
										false,false,false,false,1,null);

			endNode.dispatchEvent(mouseUpEvt);
		}
		else
		{
			console.log("Selection set without the mouse");
			var range = document.createRange();
			range.setStart(startNode, 0);
			range.setEnd(endNode, 0);

			var sel = window.getSelection();
			sel.removeAllRanges();
			sel.addRange(range);
		}
	}
}

function isRegionsEnabled()
{
	var style = document.getElementById("region").style

	if (typeof style["flow-from"] == 'string')
		return true;
	else
		return false;
}

/*
 * Determines whether range formed from the startPoint and endPoint
 * are in top to bottom order in the DOM
 */
function isTopToBottom(startPoint, endPoint)
{
	var start = document.createRange();
	start.setStart(startPoint, 0);
	start.setEnd(startPoint, 0);

	var end = document.createRange();
	end.setStart(endPoint, 0);
	end.setEnd(endPoint, 0);

	if( start.compareBoundaryPoints(Range.START_TO_END, end) < 0)
		return true;
	else
		return false;
}

/*
 * Returns just the text in the range specified by start and end, with newlines removed
 */
function getTextRange(start, end)
{
	var startNode = document.getElementById(start);
	var endNode = document.getElementById(end);

	var range = document.createRange();
	if(isTopToBottom(startNode, endNode))
	{
		range.setStart(startNode, 0);
		range.setEnd(endNode, 0);
	}
	else
	{
		range.setStart(endNode, 0);
		range.setEnd(startNode, 0);
	}

	return range.toString().replace(/\n/g,"");
}

/*
 * Returns just the text that is currently selected in the document, with newlines removed
 */
function getCurrentSelectedText()
{
	var currentSelection = "";

	var sel = window.getSelection();
	if (sel.rangeCount)
	{
		for (var i = 0, len = sel.rangeCount; i < len; ++i)
		{
			currRange = sel.getRangeAt(i);
			currentSelection += sel.getRangeAt(i).toString();
		}
	}

	return currentSelection.replace(/\n/g,"");
}

/*
 * Verifies the current selection text matches the text between the start-select and end-select elements
 */
function verifySelectedText()
{
	var expected = getTextRange("start-select", "end-select");
	var actual = getCurrentSelectedText();
	assert_equals(actual, expected);
}

/*
 * Returns true of strToCheck is in the current document selection, false if not
 */
function isStringInSelection(strToCheck)
{
	var sel = window.getSelection().getRangeAt(0);

	// If not, check for a substring
	if(sel.toString().indexOf(strToCheck) >= 0)
	 	return true;
	else
		return false;
}

/*
 * Returns true if the node toCheck is in the current document selection
 */
function isNodeInSelection(toCheck)
{
	var sel = window.getSelection().getRangeAt(0);

	// If it's a node in the document, check the start & end points
	var nodeToCheck = document.getElementById(toCheck);
	var range = document.createRange()
	range.setStart(nodeToCheck, 0);
	range.setEnd(nodeToCheck, nodeToCheck.childNodes.length);

	var startToStart = sel.compareBoundaryPoints(Range.START_TO_START, range);
	var startToEnd = sel.compareBoundaryPoints(Range.START_TO_END, range);
	var endToEnd = sel.compareBoundaryPoints(Range.END_TO_END, range);
	var endToStart = sel.compareBoundaryPoints(Range.END_TO_START, range);

	if(startToStart == startToEnd == endToEnd == endToStart)
		return false;
	else
		return true;
}



