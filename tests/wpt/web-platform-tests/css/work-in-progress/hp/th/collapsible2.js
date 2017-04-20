/***************************************************************************
*
*	Collapsible Boxes - version 1.1
*	Edited for the Digitaria Site
*	Description: Object that allows you to have boxes that expand/collapse
*	Author: Michael Turnwall
*
*	Copyright 2007 Michael Turnwall - Some Rights Reserved
*	http://creativecommons.org/licenses/GPL/2.0/
*
*	This program is free software; you can redistribute it and/or modify
*	it under the terms of the GNU General Public License as published by
*	the Free Software Foundation (GPLv2)
*
*	This program is distributed in the hope that it will be useful,
*	but WITHOUT ANY WARRANTY; without even the implied warranty of
*	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
*
*	See http://creativecommons.org/licenses/GPL/2.0/ for more information.
*
***************************************************************************/

// function that allows getting elements by their class name. Returns the found elements in an array
// arguments: className - class name to search for, element - a starting point to start the search
// if no element is specified, all the nodes on the page are searched
document.getElementsByClassName = function(className,element)
{
	if(element)
	{
		var el = document.getElementById(element);
		var children = el.getElementsByTagName('*');
	}
	else
	{
		var children = document.getElementsByTagName('*') || document.all;
	}
	
	var elements = [];
	var regexp = new RegExp("\\b"+className+"\\b","g");
	for(i = 0; i < children.length; i++)
	{
		if(children[i].className.match(regexp))
			elements.push(children[i]);
	}
	if(elements.length > 0)
		return elements;
	else
		return false;
}
/*----------------------*/

function BoxObj(values)
{
	// grab the triggers and boxes
	this.triggers = document.getElementsByClassName(values.trigger);
	this.boxes = document.getElementsByClassName(values.box);
	// assign defaults for objName and collapsedHeight
	if(!values.objName)
		this.objName = "collapsable";
	else
		this.objName = values.objName;
	if(!values.collapsedHeight)
		this.collapseHeight = 0;
	else
		this.collapseHeight = values.collapsedHeight;
		
	this.openClass = values.openClass;
	this.closeClass = values.closeClass;
	// make sure number of triggers == number of boxes
	if(this.triggers.length != this.boxes.length)
	{
		alert("Seems like the amount of triggers doesn't equal the number of boxes.\nCheck your HTML.\nTriggers = " + this.triggers.length + " Boxes = " + this.boxes.length);
		return false;
	}
	var self = this;
	// assign click events to the triggers
	for(var i = 0; i < this.triggers.length; i++)
	{
		this.triggers[i].setAttribute("id","trigger"+i);
		this.triggers[i].onclick = function()
		{
			this.blur();
			self.boxNum = this.id.replace("trigger","");
			self.initEffect();
			return false;
		}
	}

	this.collection = new Array();
	// load the 2 dimensional array to relate each trigger with it's corresponding box
	alert ("box 0 offsetHeight is " + this.boxes[0].offsetHeight);	
	for(var i = 0; i < this.triggers.length; i++)
	{

		this.collection[i] = new Array();
		// [0] is the link, [1] is the box, [2] is the space from padding/borders, [3] is the height minus the padding & borders (targetHeight)
		this.collection[i][0] = this.triggers[i];
		this.collection[i][1] = this.boxes[i];
		this.boxes[i].style.display = "block";
		this.collection[i][2] = this.getWhiteSpace(this.boxes[i]);
		this.collection[i][3] = (this.boxes[i].offsetHeight);
		//alert(this.boxes[i].offsetHeight);
		this.boxes[i].style.height = this.collapseHeight + "px";
	}
	this.effectTimer;	// going to be used later for the timer
	// used to close the previously opened box. Added for Digi site.
	this.closeTimer;
	this.openedElement;
}
/*----------------------*/

// computes the extra space of the box from the padding and borders
// this needs to be heavily optimized. Too dirty but works for now
BoxObj.prototype.getWhiteSpace = function(el)
{
	if(el.currentStyle)
	{
		var padTop = el.currentStyle["paddingTop"];
		var padBottom = el.currentStyle["paddingBottom"];
		var borderTop = el.currentStyle["borderTopWidth"];
		var borderBottom = el.currentStyle["borderBottomWidth"];
	}
	else if(document.defaultView.getComputedStyle)
	{
		var padTop = document.defaultView.getComputedStyle(el,null).getPropertyValue("padding-top");
		var padBottom = document.defaultView.getComputedStyle(el,null).getPropertyValue("padding-bottom");
		var borderTop = document.defaultView.getComputedStyle(el,null).getPropertyValue("border-top-width");
		var borderBottom = document.defaultView.getComputedStyle(el,null).getPropertyValue("border-bottom-width");
	}
	padTop = (padTop.replace(/px/,"")) * 1;
	padBottom = (padBottom.replace(/px/,"")) * 1;
	borderTop = (borderTop.replace(/px/,"")) * 1;
	borderBottom = (borderBottom.replace(/px/,"")) * 1;
	if(isNaN(borderTop))
		borderTop = 0;
	if(isNaN(borderBottom))
		borderBottom = 0;
	return padTop + padBottom + borderTop + borderBottom;
}

// starts the effect and determines if the box needs to be collapsed or expanded
BoxObj.prototype.initEffect = function()
{
	// assign the array values to temp variables to reduce the amount of typing
	var extraSpace = this.collection[this.boxNum][2];
	var targetHeight = this.collection[this.boxNum][3];
	//added for digi site to close boxes that are already open
	/*if(this.openedElement)
	{
		if(this.openedElement != this.boxNum)
		{
			var oldExtraSpace = this.collection[this.openedElement][2];
			var oldTargetHeight = this.collection[this.openedElement][3];
			this.collapseBox(oldExtraSpace,oldTargetHeight,this.openedElement);
		}
	}*/
	alert(this.collection[this.boxNum][1].offsetHeight +"," +this.collection[this.boxNum][3]);
	if(this.collection[this.boxNum][1].offsetHeight < this.collection[this.boxNum][3])
	{
		if(this.openClass)
			this.collection[this.boxNum][1].className = this.collection[this.boxNum][1].className.replace(this.closeClass,this.openClass)
		//this.collection[this.boxNum][1].style.height = (Math.round(targetHeight/4)) + "px";
		this.adjustSize = Math.round(targetHeight/15);
		this.growBox(extraSpace,targetHeight);
	}
	else
	{
		//this.collection[this.boxNum][1].style.height = (Math.round(targetHeight/4)) + "px";
		this.collapseBox(extraSpace,targetHeight,this.boxNum);
	}
	this.openedElement = this.boxNum
}
/*----------------------*/

BoxObj.prototype.growBox = function(extraSpace,targetHeight)
{
	clearTimeout(this.effectTimer);
	var box = this.collection[this.boxNum][1];
	box.style.height = (box.offsetHeight-extraSpace + this.adjustSize) + "px";
	if((box.offsetHeight-extraSpace) < targetHeight/2)
	{
		//box.style.height = (box.offsetHeight-extraSpace + this.adjustSize) + "px";
		this.effectTimer = window.setTimeout(this.objName+".growBox("+extraSpace+","+targetHeight+")",10);
	}
	else if((box.offsetHeight-extraSpace) < targetHeight)
	{
		//var newHeight = ((box.offsetHeight-extraSpace) + this.adjustSize);
		//box.style.height = (box.offsetHeight-extraSpace + this.adjustSize) + "px";
		this.effectTimer = window.setTimeout(this.objName+".growBox("+extraSpace+","+targetHeight+")",20);
	}
	else
	{
		//alert(targetHeight)
		//box.style.height = targetHeight + "px";
	}
}
/*----------------------*/

BoxObj.prototype.collapseBox = function(extraSpace,targetHeight,boxNum)
{
	clearTimeout(this.closeTimer);
	var box = this.collection[boxNum][1];

	if((box.offsetHeight-extraSpace) > targetHeight/2)
	{
		box.style.height = ((box.offsetHeight-extraSpace) - this.adjustSize) + "px";
		this.closeTimer = window.setTimeout(this.objName+".collapseBox("+extraSpace+","+targetHeight+","+boxNum+")",10);
	}
	else if((box.offsetHeight-extraSpace) > this.collapseHeight)
	{
		var newHeight = ((box.offsetHeight-extraSpace) - this.adjustSize);
		if(newHeight > this.collapseHeight)
		{
			box.style.height = newHeight + "px";
			this.closeTimer = window.setTimeout(this.objName+".collapseBox("+extraSpace+","+targetHeight+","+boxNum+")",20);
		}
		else if(newHeight == this.collapseHeight || newHeight < this.collapseHeight)
		{
			box.style.height = this.collapseHeight + "px";
			if(this.closeClass)
				this.collection[boxNum][1].className = this.collection[boxNum][1].className.replace(this.openClass,this.closeClass)
		}
	}
	else
	{
		if(this.closeClass)
			this.collection[boxNum][1].className = this.collection[boxNum][1].className.replace(this.openClass,this.closeClass)
		box.style.height = this.collapseHeight + "px";
	}
}
/*----------------------*/
