<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">

<html xmlns="http://www.w3.org/1999/xhtml">

 <head>

  <title>CSS Writing Modes Test: box offsets - relatively positioned boxes</title>

  <link rel="author" title="Gérard Talbot" href="http://www.gtalbot.org/BrowserBugsSection/css21testsuite/" />
  <link rel="help" title="7.1 Principles of Layout in Vertical Writing Modes" href="http://www.w3.org/TR/css-writing-modes-3/#vertical-layout" />
  <link rel="match" href="box-offsets-rel-pos-vlr-005-ref.xht" />

  <meta content="image" name="flags" />
  <meta content="Box offsets (bottom, left, right, top) expressed in absolute units (not with percentage unit) for relatively positioned boxes are with respect to the edges of the boxes themselves." name="assert" />

  <style type="text/css"><![CDATA[
  html
    {
      writing-mode: vertical-lr;
    }

  div#statically-positioned-box
    {
      background-color: yellow; /* padding box will be yellow */
      border: orange solid 50px; /* border box will be orange */
      height: 100px; /* a bright green square 100px by 100px image will serve as content box */
      margin-left: 8px;
      padding: 50px;
      position: static;
      width: 100px;
    }

  div.blue-relatively-positioned
    {
      background-color: blue;
      color: white;
      height: 25px;
      position: relative;
      width: 25px;
      writing-mode: horizontal-tb;
    }

  div#top-left
    {
      right: 250px;
      /*
      Calculation of right offset:
          50px (div#statically-positioned-box's border-right)
       +
         200px (div#statically-positioned-box's padding-box width)
       ==================
         250px
      */

      top: 50px;
      /*
      Calculation of top offset:
        50px (div#statically-positioned-box's border-top)
      ==================
        50px
      */
    }

  div#top-right
    {
      right: 100px;
      /*
      Calculation of right offset:
          25px (div#top-left's content width)
       +
          25px (div#top-right's content width)
       +
          50px (div#statically-positioned-box's border-right)
       ==================
         100px
      */

      top: 50px;
      /*
      Calculation of top offset:
        50px (div#statically-positioned-box's border-top)
      ==================
        50px
      */
    }

  div#bottom-left
    {
      top: 225px;
      /*
      Calculation of top offset:
          50px (div#statically-positioned-box's border-top)
       +
         200px (div#statically-positioned-box's padding-box height)
       -
          25px (div#bottom-left's content height)
      ==================
         225px
    */

      right: 300px;
      /*
      Calculation of right offset:
          25px (div#top-left's content width)
       +
          25px (div#top-right's content width)
       +
          50px (div#statically-positioned-box's border-right)
       +
         200px (div#statically-positioned-box's padding-box width)
       ==================
         300px
      */
    }

  div#bottom-right
    {
      top: 225px;
      /*
      Calculation of top offset:
          50px (div#statically-positioned-box's border-top)
       +
         200px (div#statically-positioned-box's padding-box height)
       -
          25px (div#bottom-right's content height)
      ==================
         225px
    */

      right: 150px;
      /*
      Calculation of right offset:
          25px (div#top-left's content width)
       +
          25px (div#top-right's content width)
       +
          25px (div#bottom-left's content width)
       +
          25px (div#bottom-right's content width)
       +
          50px (div#statically-positioned-box's border-left)
       ==================
         150px
      */
  }
  ]]></style>

 </head>

 <body>

  <p><img src="support/pass-cdts-box-offsets-rel-pos.png" width="304" height="35" alt="Image download support must be enabled" /></p>

  <!--
  The image says:
  "
  Test passes if there is a blue square
  at each corner of the yellow square.
  "
  -->

  <div id="statically-positioned-box"><img src="support/100x100-lime.png" alt="Image download support must be enabled" /></div>

  <div class="blue-relatively-positioned" id="top-left">TL</div>

  <div class="blue-relatively-positioned" id="top-right">TR</div>

  <div class="blue-relatively-positioned" id="bottom-left">BL</div>

  <div class="blue-relatively-positioned" id="bottom-right">BR</div>

 </body>
</html>