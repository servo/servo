/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
//-----------------------------------------------------------------------------
var BUGNUMBER = 449627;
var summary = 'Crash with JIT in js_FillPropertyCache';
var actual = 'No Crash';
var expect = 'No Crash';


/************************ BROWSER DETECT (http://www.quirksmode.org/js/detect.html) ************************/

if (typeof navigator == 'undefined')
{
  var navigator = {
    userAgent: "Firefox",
    vendor: "Mozilla",
    platform: "Mac"
  };
}

var global = this;

var BrowserDetect = {
    init: function _init()
    {
      this.browser=this.searchString(this.dataBrowser) || "An unknown browser";

      this.OS= this.searchString(this.dataOS)||"an unknown OS";
    },
    searchString: function _searchString(a)
    {
      for(var i=0; i < a.length; i++)
      {
	var b=a[i].string;
	var c=a[i].prop;
	this.versionSearchString=a[i].versionSearch||a[i].identity;
	if(b)
	{
	  if(b.indexOf(a[i].subString)!=-1)
	    return a[i].identity;
        }
	else if(c)
	return a[i].identity;
      }
    },

    searchVersion:function _searchVersion(a)
    {
      var b=a.indexOf(this.versionSearchString);
      if(b==-1)
      	return;
      return parseFloat(a.substring(b+this.versionSearchString.length+1));
    },

    dataBrowser:[
      {
	string:navigator.userAgent,subString:"OmniWeb",versionSearch:"OmniWeb/",identity:"OmniWeb"
      },
      {
	string:navigator.vendor,subString:"Apple",identity:"Safari"
      },
      {
	prop:global.opera,identity:"Opera"
      },
      {
	string:navigator.vendor,subString:"iCab",identity:"iCab"
      },
      {
	string:navigator.vendor,subString:"KDE",identity:"Konqueror"
      },
      {
	string:navigator.userAgent,subString:"Firefox",identity:"Firefox"
      },
      {
	string:navigator.vendor,subString:"Camino",identity:"Camino"
      },
      {
	string:navigator.userAgent,subString:"Netscape",identity:"Netscape"
      },
      {
	string:navigator.userAgent,subString:"MSIE",identity:"Explorer",versionSearch:"MSIE"
      },
      {
	string:navigator.userAgent,subString:"Gecko",identity:"Mozilla",versionSearch:"rv"
      },
      {
	string:navigator.userAgent,subString:"Mozilla",identity:"Netscape",versionSearch:"Mozilla"
      }
    ],
    dataOS:[
      {
	string:navigator.platform,subString:"Win",identity:"Windows"
      },
      {
	string:navigator.platform,subString:"Mac",identity:"Mac"
      },
      {
	string:navigator.platform,subString:"Linux",identity:"Linux"
      }
    ]
  };

BrowserDetect.init();


assert.sameValue(expect, actual, summary);
