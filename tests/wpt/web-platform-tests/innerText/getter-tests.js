testText("<div>abc", "abc", "Simplest possible test");

/**** white-space:normal ****/

testText("<div> abc", "abc", "Leading whitespace removed");
testText("<div>abc ", "abc", "Trailing whitespace removed");
testText("<div>abc  def", "abc def", "Internal whitespace compressed");
testText("<div>abc\ndef", "abc def", "\\n converted to space");
testText("<div>abc\rdef", "abc def", "\\r converted to space");
testText("<div>abc\tdef", "abc def", "\\t converted to space");
testText("<div>abc <br>def", "abc\ndef", "Trailing whitespace before hard line break removed");

/**** <pre> ****/

testText("<pre> abc", " abc", "Leading whitespace preserved");
testText("<pre>abc ", "abc ", "Trailing whitespace preserved");
testText("<pre>abc  def", "abc  def", "Internal whitespace preserved");
testText("<pre>abc\ndef", "abc\ndef", "\\n preserved");
testText("<pre>abc\rdef", "abc\ndef", "\\r converted to newline");
testText("<pre>abc\tdef", "abc\tdef", "\\t preserved");

/**** <div style="white-space:pre"> ****/

testText("<div style='white-space:pre'> abc", " abc", "Leading whitespace preserved");
testText("<div style='white-space:pre'>abc ", "abc ", "Trailing whitespace preserved");
testText("<div style='white-space:pre'>abc  def", "abc  def", "Internal whitespace preserved");
testText("<div style='white-space:pre'>abc\ndef", "abc\ndef", "\\n preserved");
testText("<div style='white-space:pre'>abc\rdef", "abc\ndef", "\\r converted to newline");
testText("<div style='white-space:pre'>abc\tdef", "abc\tdef", "\\t preserved");

/**** <span style="white-space:pre"> ****/

testText("<span style='white-space:pre'> abc", " abc", "Leading whitespace preserved");
testText("<span style='white-space:pre'>abc ", "abc ", "Trailing whitespace preserved");
testText("<span style='white-space:pre'>abc  def", "abc  def", "Internal whitespace preserved");
testText("<span style='white-space:pre'>abc\ndef", "abc\ndef", "\\n preserved");
testText("<span style='white-space:pre'>abc\rdef", "abc\ndef", "\\r converted to newline");
testText("<span style='white-space:pre'>abc\tdef", "abc\tdef", "\\t preserved");

/**** <div style="white-space:pre-line"> ****/

testText("<div style='white-space:pre-line'> abc", "abc", "Leading whitespace removed");
testText("<div style='white-space:pre-line'>abc ", "abc", "Trailing whitespace removed");
testText("<div style='white-space:pre-line'>abc  def", "abc def", "Internal whitespace collapsed");
testText("<div style='white-space:pre-line'>abc\ndef", "abc\ndef", "\\n preserved");
testText("<div style='white-space:pre-line'>abc\rdef", "abc\ndef", "\\r converted to newline");
testText("<div style='white-space:pre-line'>abc\tdef", "abc def", "\\t converted to space");

/**** Collapsing whitespace across element boundaries ****/

testText("<div><span>abc </span> def", "abc def", "Whitespace collapses across element boundaries");
testText("<div><span>abc </span><span></span> def", "abc def", "Whitespace collapses across element boundaries");
testText("<div><span>abc </span><span style='white-space:pre'></span> def", "abc def", "Whitespace collapses across element boundaries");

/**** Soft line breaks ****/

testText("<div style='width:0'>abc def", "abc def", "Soft line breaks ignored");

/**** first-line/first-letter ****/

testText("<div class='first-line-uppercase' style='width:0'>abc def", "ABC def", "::first-line styles applied");
testText("<div class='first-letter-uppercase' style='width:0'>abc def", "Abc def", "::first-letter styles applied");
testText("<div class='first-letter-float' style='width:0'>abc def", "abc def", "::first-letter float ignored");

/**** &nbsp; ****/

testText("<div>&nbsp;", "\xA0", "&nbsp; preserved");

/**** display:none ****/

testText("<div style='display:none'>abc", "abc", "display:none container");
testText("<div style='display:none'>abc  def", "abc  def", "No whitespace compression in display:none container");
testText("<div style='display:none'> abc def ", " abc def ", "No removal of leading/trailing whitespace in display:none container");
testText("<div>123<span style='display:none'>abc", "123", "display:none child not rendered");
testText("<div style='display:none'><span id='target'>abc", "abc", "display:none container with non-display-none target child");
testTextInSVG("<div id='target'>abc", "", "non-display-none child of svg");
testTextInSVG("<div style='display:none' id='target'>abc", "abc", "display:none child of svg");
testTextInSVG("<div style='display:none'><div id='target'>abc", "abc", "child of display:none child of svg");

/**** display:contents ****/

if (CSS.supports("display", "contents")) {
  testText("<div style='display:contents'>abc", "abc", "display:contents container");
  testText("<div><div style='display:contents'>abc", "abc", "display:contents container");
  testText("<div>123<span style='display:contents'>abc", "123abc", "display:contents rendered");
  testText("<div style='display:contents'>   ", "", "display:contents not processed via textContent");
  testText("<div><div style='display:contents'>   ", "", "display:contents not processed via textContent");
}

/**** visibility:hidden ****/

testText("<div style='visibility:hidden'>abc", "", "visibility:hidden container");
testText("<div>123<span style='visibility:hidden'>abc", "123", "visibility:hidden child not rendered");
testText("<div style='visibility:hidden'>123<span style='visibility:visible'>abc", "abc", "visibility:visible child rendered");

/**** visibility:collapse ****/

testText("<table><tbody style='visibility:collapse'><tr><td>abc", "", "visibility:collapse row-group");
testText("<table><tr style='visibility:collapse'><td>abc", "", "visibility:collapse row");
testText("<table><tr><td style='visibility:collapse'>abc", "", "visibility:collapse cell");
testText("<table><tbody style='visibility:collapse'><tr><td style='visibility:visible'>abc", "abc",
         "visibility:collapse row-group with visible cell");
testText("<table><tr style='visibility:collapse'><td style='visibility:visible'>abc", "abc",
         "visibility:collapse row with visible cell");
testText("<div style='display:flex'><span style='visibility:collapse'>1</span><span>2</span></div>",
         "2", "visibility:collapse honored on flex item");
testText("<div style='display:grid'><span style='visibility:collapse'>1</span><span>2</span></div>",
         "2", "visibility:collapse honored on grid item");

/**** opacity:0 ****/

testText("<div style='opacity:0'>abc", "abc", "opacity:0 container");
testText("<div style='opacity:0'>abc  def", "abc def", "Whitespace compression in opacity:0 container");
testText("<div style='opacity:0'> abc def ", "abc def", "Remove leading/trailing whitespace in opacity:0 container");
testText("<div>123<span style='opacity:0'>abc", "123abc", "opacity:0 child rendered");

/**** generated content ****/

testText("<div class='before'>", "", "Generated content not included");
testText("<div><div class='before'>", "", "Generated content on child not included");

/**** innerText on replaced elements ****/

testText("<button>abc", "abc", "<button> contents preserved");
testText("<fieldset>abc", "abc", "<fieldset> contents preserved");
testText("<fieldset><legend>abc", "abc", "<fieldset> <legend> contents preserved");
testText("<input type='text' value='abc'>", "", "<input> contents ignored");
testText("<textarea>abc", "", "<textarea> contents ignored");
testText("<iframe>abc", "", "<iframe> contents ignored");
testText("<iframe><div id='target'>abc", "", "<iframe> contents ignored");
testText("<iframe src='data:text/html,abc'>", "","<iframe> subdocument ignored");
testText("<audio style='display:block'>abc", "", "<audio> contents ignored");
testText("<audio style='display:block'><source id='target' class='poke' style='display:block'>", "", "<audio> contents ignored");
testText("<audio style='display:block'><source id='target' class='poke' style='display:none'>", "abc", "<audio> contents ok if display:none");
testText("<video>abc", "", "<video> contents ignored");
testText("<video style='display:block'><source id='target' class='poke' style='display:block'>", "", "<video> contents ignored");
testText("<video style='display:block'><source id='target' class='poke' style='display:none'>", "abc", "<video> contents ok if display:none");
testText("<canvas>abc", "", "<canvas> contents ignored");
testText("<canvas><div id='target'>abc", "", "<canvas><div id='target'> contents ignored");
testText("<img alt='abc'>", "", "<img> alt text ignored");
testText("<img src='about:blank' class='poke'>", "", "<img> contents ignored");

/**** <select>, <optgroup> & <option> ****/

testText("<select size='1'><option>abc</option><option>def", "abc\ndef", "<select size='1'> contents of options preserved");
testText("<select size='2'><option>abc</option><option>def", "abc\ndef", "<select size='2'> contents of options preserved");
testText("<select size='1'><option id='target'>abc</option><option>def", "abc", "<select size='1'> contents of target option preserved");
testText("<select size='2'><option id='target'>abc</option><option>def", "abc", "<select size='2'> contents of target option preserved");
testText("<div>a<select></select>bc", "abc", "empty <select>");
testText("<div>a<select><optgroup></select>bc", "a\nbc", "empty <optgroup> in <select>");
testText("<div>a<select><option></select>bc", "a\nbc", "empty <option> in <select>");
testText("<select class='poke'></select>", "", "<select> containing text node child");
testText("<select><optgroup class='poke-optgroup'></select>", "", "<optgroup> containing <optgroup>");
testText("<select><optgroup><option>abc</select>", "abc", "<optgroup> containing <option>");
testText("<select><option class='poke-div'>123</select>", "123\nabc", "<div> in <option>");
testText("<div>a<optgroup></optgroup>bc", "a\nbc", "empty <optgroup> in <div>");
testText("<div>a<optgroup>123</optgroup>bc", "a\nbc", "<optgroup> in <div>");
testText("<div>a<option></option>bc", "a\nbc", "empty <option> in <div>");
testText("<div>a<option>123</option>bc", "a\n123\nbc", "<option> in <div>");

/**** innerText on replaced element children ****/

testText("<div><button>abc", "abc", "<button> contents preserved");
testText("<div><fieldset>abc", "abc", "<fieldset> contents preserved");
testText("<div><fieldset><legend>abc", "abc", "<fieldset> <legend> contents preserved");
testText("<div><input type='text' value='abc'>", "", "<input> contents ignored");
testText("<div><textarea>abc", "", "<textarea> contents ignored");
testText("<div><select size='1'><option>abc</option><option>def", "abc\ndef", "<select size='1'> contents of options preserved");
testText("<div><select size='2'><option>abc</option><option>def", "abc\ndef", "<select size='2'> contents of options preserved");
testText("<div><iframe>abc", "", "<iframe> contents ignored");
testText("<div><iframe src='data:text/html,abc'>", ""," <iframe> subdocument ignored");
testText("<div><audio>abc", "", "<audio> contents ignored");
testText("<div><video>abc", "", "<video> contents ignored");
testText("<div><canvas>abc", "", "<canvas> contents ignored");
testText("<div><img alt='abc'>", "", "<img> alt text ignored");

/**** Lines around blocks ****/

testText("<div>123<div>abc</div>def", "123\nabc\ndef", "Newline at block boundary");
testText("<div>123<span style='display:block'>abc</span>def", "123\nabc\ndef", "Newline at display:block boundary");
testText("<div>abc<div></div>def", "abc\ndef", "Empty block induces single line break");
testText("<div>abc<div></div><div></div>def", "abc\ndef", "Consecutive empty blocks ignored");
testText("<div><p>abc", "abc", "No blank lines around <p> alone");
testText("<div><p>abc</p> ", "abc", "No blank lines around <p> followed by only collapsible whitespace");
testText("<div> <p>abc</p>", "abc", "No blank lines around <p> preceded by only collapsible whitespace");
testText("<div><p>abc<p>def", "abc\n\ndef", "Blank line between consecutive <p>s");
testText("<div><p>abc</p> <p>def", "abc\n\ndef", "Blank line between consecutive <p>s separated only by collapsible whitespace");
testText("<div><p>abc</p><div></div><p>def", "abc\n\ndef", "Blank line between consecutive <p>s separated only by empty block");
testText("<div><p>abc</p><div>123</div><p>def", "abc\n\n123\n\ndef", "Blank lines between <p>s separated by non-empty block");
testText("<div>abc<div><p>123</p></div>def", "abc\n\n123\n\ndef", "Blank lines around a <p> in its own block");
testText("<div>abc<p>def", "abc\n\ndef", "Blank line before <p>");
testText("<div><p>abc</p>def", "abc\n\ndef", "Blank line after <p>");
testText("<div><p>abc<p></p><p></p><p>def", "abc\n\ndef", "One blank line between <p>s, ignoring empty <p>s");
testText("<div style='visibility:hidden'><p><span style='visibility:visible'>abc</span></p>\n<div style='visibility:visible'>def</div>",
     "abc\ndef", "Invisible <p> doesn't induce extra line breaks");
testText("<div>abc<div style='margin:2em'>def", "abc\ndef", "No blank lines around <div> with margin");
testText("<div>123<span style='display:inline-block'>abc</span>def", "123abcdef", "No newlines at display:inline-block boundary");
testText("<div>123<span style='display:inline-block'> abc </span>def", "123abcdef", "Leading/trailing space removal at display:inline-block boundary");

/**** Spans ****/

testText("<div>123<span>abc</span>def", "123abcdef", "<span> boundaries are irrelevant");
testText("<div>123 <span>abc</span> def", "123 abc def", "<span> boundaries are irrelevant");
testText("<div style='width:0'>123 <span>abc</span> def", "123 abc def", "<span> boundaries are irrelevant");
testText("<div>123<em>abc</em>def", "123abcdef", "<em> gets no special treatment");
testText("<div>123<b>abc</b>def", "123abcdef", "<b> gets no special treatment");
testText("<div>123<i>abc</i>def", "123abcdef", "<i> gets no special treatment");
testText("<div>123<strong>abc</strong>def", "123abcdef", "<strong> gets no special treatment");
testText("<div>123<tt>abc</tt>def", "123abcdef", "<tt> gets no special treatment");
testText("<div>123<code>abc</code>def", "123abcdef", "<code> gets no special treatment");

/**** Soft hyphen ****/

testText("<div>abc&shy;def", "abc\xADdef", "soft hyphen preserved");
testText("<div style='width:0'>abc&shy;def", "abc\xADdef", "soft hyphen preserved");

/**** Tables ****/

testText("<div><table style='white-space:pre'>  <td>abc</td>  </table>", "abc", "Ignoring non-rendered table whitespace");
testText("<div><table><tr><td>abc<td>def</table>", "abc\tdef", "Tab-separated table cells");
testText("<div><table><tr><td>abc<td><td>def</table>", "abc\t\tdef", "Tab-separated table cells including empty cells");
testText("<div><table><tr><td>abc<td><td></table>", "abc\t\t", "Tab-separated table cells including trailing empty cells");
testText("<div><table><tr><td>abc<tr><td>def</table>", "abc\ndef", "Newline-separated table rows");
testText("<div>abc<table><td>def</table>ghi", "abc\ndef\nghi", "Newlines around table");
testText("<div><table style='border-collapse:collapse'><tr><td>abc<td>def</table>", "abc\tdef",
         "Tab-separated table cells in a border-collapse table");
testText("<div><table><tfoot>x</tfoot><tbody>y</tbody></table>", "xy", "tfoot not reordered");
testText("<table><tfoot><tr><td>footer</tfoot><thead><tr><td style='visibility:collapse'>thead</thead><tbody><tr><td>tbody</tbody></table>",
         "footer\n\ntbody", "");

/**** Table captions ****/

testText("<div><table><tr><td>abc<caption>def</caption></table>", "abc\ndef", "Newline between cells and caption");

/**** display:table ****/

testText("<div><div class='table'><span class='cell'>abc</span>\n<span class='cell'>def</span></div>",
         "abc\tdef", "Tab-separated table cells");
testText("<div><div class='table'><span class='row'><span class='cell'>abc</span></span>\n<span class='row'><span class='cell'>def</span></span></div>",
         "abc\ndef", "Newline-separated table rows");
testText("<div>abc<div class='table'><span class='cell'>def</span></div>ghi", "abc\ndef\nghi", "Newlines around table");

/**** display:inline-table ****/

testText("<div><div class='itable'><span class='cell'>abc</span>\n<span class='cell'>def</span></div>", "abc\tdef", "Tab-separated table cells");
testText("<div><div class='itable'><span class='row'><span class='cell'>abc</span></span>\n<span class='row'><span class='cell'>def</span></span></div>",
         "abc\ndef", "Newline-separated table rows");
testText("<div>abc<div class='itable'><span class='cell'>def</span></div>ghi", "abcdefghi", "No newlines around inline-table");
testText("<div>abc<div class='itable'><span class='row'><span class='cell'>def</span></span>\n<span class='row'><span class='cell'>123</span></span></div>ghi",
         "abcdef\n123ghi", "Single newline in two-row inline-table");

/**** Lists ****/

testText("<div><ol><li>abc", "abc", "<ol> list items get no special treatment");
testText("<div><ul><li>abc", "abc", "<ul> list items get no special treatment");

/**** Misc elements ****/

testText("<div><script style='display:block'>abc", "abc", "display:block <script> is rendered");
testText("<div><style style='display:block'>abc", "abc", "display:block <style> is rendered");
testText("<div><noscript style='display:block'>abc", "", "display:block <noscript> is not rendered (it's not parsed!)");
testText("<div><template style='display:block'>abc", "",
         "display:block <template> contents are not rendered (the contents are in a different document)");
testText("<div>abc<br>def", "abc\ndef", "<br> induces line break");
testText("<div>abc<br>", "abc\n", "<br> induces line break even at end of block");
testText("<div><br class='poke'>", "\n", "<br> content ignored");
testText("<div>abc<hr>def", "abc\ndef", "<hr> induces line break");
testText("<div>abc<hr><hr>def", "abc\ndef", "<hr><hr> induces just one line break");
testText("<div>abc<hr><hr><hr>def", "abc\ndef", "<hr><hr><hr> induces just one line break");
testText("<div><hr class='poke'>", "abc", "<hr> content rendered");
testText("<div>abc<!--comment-->def", "abcdef", "comment ignored");

/**** text-transform ****/

testText("<div><div style='text-transform:uppercase'>abc", "ABC", "text-transform is applied");
testText("<div><div style='text-transform:uppercase'>Ma\xDF", "MASS", "text-transform handles es-zet");
testText("<div><div lang='tr' style='text-transform:uppercase'>i \u0131", "\u0130 I", "text-transform handles Turkish casing");

/**** block-in-inline ****/

testText("<div>abc<span>123<div>456</div>789</span>def", "abc123\n456\n789def", "block-in-inline doesn't add unnecessary newlines");

/**** floats ****/

testText("<div>abc<div style='float:left'>123</div>def", "abc\n123\ndef", "floats induce a block boundary");
testText("<div>abc<span style='float:left'>123</span>def", "abc\n123\ndef", "floats induce a block boundary");

/**** position ****/

testText("<div>abc<div style='position:absolute'>123</div>def", "abc\n123\ndef", "position:absolute induces a block boundary");
testText("<div>abc<span style='position:absolute'>123</span>def", "abc\n123\ndef", "position:absolute induces a block boundary");
testText("<div>abc<div style='position:relative'>123</div>def", "abc\n123\ndef", "position:relative has no effect");
testText("<div>abc<span style='position:relative'>123</span>def", "abc123def", "position:relative has no effect");

/**** text-overflow:ellipsis ****/

testText("<div style='overflow:hidden'>abc", "abc", "overflow:hidden ignored");
// XXX Chrome skips content with width:0 or height:0 and overflow:hidden;
// should we spec that?
testText("<div style='width:0; overflow:hidden'>abc", "abc", "overflow:hidden ignored even with zero width");
testText("<div style='height:0; overflow:hidden'>abc", "abc", "overflow:hidden ignored even with zero height");
testText("<div style='width:0; overflow:hidden; text-overflow:ellipsis'>abc", "abc", "text-overflow:ellipsis ignored");

/**** Support on non-HTML elements ****/

testText("<svg>abc", undefined, "innerText not supported on SVG elements");
testText("<math>abc", undefined, "innerText not supported on MathML elements");

/**** Ruby ****/

testText("<div><ruby>abc<rt>def</rt></ruby>", "abcdef", "<rt> and no <rp>");
testText("<div><ruby>abc<rp>(</rp><rt>def</rt><rp>)</rp></ruby>", "abcdef", "<rp>");
testText("<div><rp>abc</rp>", "", "Lone <rp>");
testText("<div><rp style='visibility:hidden'>abc</rp>", "", "visibility:hidden <rp>");
testText("<div><rp style='display:block'>abc</rp>def", "abc\ndef", "display:block <rp>");
testText("<div><rp style='display:block'> abc </rp>def", "abc\ndef", "display:block <rp> with whitespace");
testText("<div><select class='poke-rp'></select>", "", "<rp> in a <select>");

/**** Shadow DOM ****/

if ("createShadowRoot" in document.body) {
  testText("<div class='shadow'>", "", "Shadow DOM contents ignored");
  testText("<div><div class='shadow'>", "", "Shadow DOM contents ignored");
}

/**** Flexbox ****/

if (CSS.supports('display', 'flex')) {
  testText("<div style='display:flex'><div style='order:1'>1</div><div>2</div></div>",
           "1\n2", "CSS 'order' property ignored");
  testText("<div style='display:flex'><span>1</span><span>2</span></div>",
           "1\n2", "Flex items blockified");
}

/**** Grid ****/

if (CSS.supports('display', 'grid')) {
  testText("<div style='display:grid'><div style='order:1'>1</div><div>2</div></div>",
           "1\n2", "CSS 'order' property ignored");
  testText("<div style='display:grid'><span>1</span><span>2</span></div>",
           "1\n2", "Grid items blockified");
}
