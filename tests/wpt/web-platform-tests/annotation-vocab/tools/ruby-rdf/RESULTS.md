<!DOCTYPE html>
<html lang='en'>
<head>
  <title>RSpec results</title>
  <meta http-equiv="Content-Type" content="text/html; charset=utf-8" />
  <meta http-equiv="Expires" content="-1" />
  <meta http-equiv="Pragma" content="no-cache" />
  <style type="text/css">
  body {
    margin: 0;
    padding: 0;
    background: #fff;
    font-size: 80%;
  }
  </style>
  <script type="text/javascript">
    // <![CDATA[

function addClass(element_id, classname) {
  document.getElementById(element_id).className += (" " + classname);
}

function removeClass(element_id, classname) {
  var elem = document.getElementById(element_id);
  var classlist = elem.className.replace(classname,'');
  elem.className = classlist;
}

function moveProgressBar(percentDone) {
  document.getElementById("rspec-header").style.width = percentDone +"%";
}

function makeRed(element_id) {
  removeClass(element_id, 'passed');
  removeClass(element_id, 'not_implemented');
  addClass(element_id,'failed');
}

function makeYellow(element_id) {
  var elem = document.getElementById(element_id);
  if (elem.className.indexOf("failed") == -1) {  // class doesn't includes failed
    if (elem.className.indexOf("not_implemented") == -1) { // class doesn't include not_implemented
      removeClass(element_id, 'passed');
      addClass(element_id,'not_implemented');
    }
  }
}

function apply_filters() {
  var passed_filter = document.getElementById('passed_checkbox').checked;
  var failed_filter = document.getElementById('failed_checkbox').checked;
  var pending_filter = document.getElementById('pending_checkbox').checked;

  assign_display_style("example passed", passed_filter);
  assign_display_style("example failed", failed_filter);
  assign_display_style("example not_implemented", pending_filter);

  assign_display_style_for_group("example_group passed", passed_filter);
  assign_display_style_for_group("example_group not_implemented", pending_filter, pending_filter || passed_filter);
  assign_display_style_for_group("example_group failed", failed_filter, failed_filter || pending_filter || passed_filter);
}

function get_display_style(display_flag) {
  var style_mode = 'none';
  if (display_flag == true) {
    style_mode = 'block';
  }
  return style_mode;
}

function assign_display_style(classname, display_flag) {
  var style_mode = get_display_style(display_flag);
  var elems = document.getElementsByClassName(classname)
  for (var i=0; i<elems.length;i++) {
    elems[i].style.display = style_mode;
  }
}

function assign_display_style_for_group(classname, display_flag, subgroup_flag) {
  var display_style_mode = get_display_style(display_flag);
  var subgroup_style_mode = get_display_style(subgroup_flag);
  var elems = document.getElementsByClassName(classname)
  for (var i=0; i<elems.length;i++) {
    var style_mode = display_style_mode;
    if ((display_flag != subgroup_flag) && (elems[i].getElementsByTagName('dt')[0].innerHTML.indexOf(", ") != -1)) {
      elems[i].style.display = subgroup_style_mode;
    } else {
      elems[i].style.display = display_style_mode;
    }
  }
}

    // ]]>
  </script>
  <style type="text/css">
#rspec-header {
  background: #65C400; color: #fff; height: 4em;
}

.rspec-report h1 {
  margin: 0px 10px 0px 10px;
  padding: 10px;
  font-family: "Lucida Grande", Helvetica, sans-serif;
  font-size: 1.8em;
  position: absolute;
}

#label {
  float:left;
}

#display-filters {
  float:left;
  padding: 28px 0 0 40%;
  font-family: "Lucida Grande", Helvetica, sans-serif;
}

#summary {
  float:right;
  padding: 5px 10px;
  font-family: "Lucida Grande", Helvetica, sans-serif;
  text-align: right;
}

#summary p {
  margin: 0 0 0 2px;
}

#summary #totals {
  font-size: 1.2em;
}

.example_group {
  margin: 0 10px 5px;
  background: #fff;
}

dl {
  margin: 0; padding: 0 0 5px;
  font: normal 11px "Lucida Grande", Helvetica, sans-serif;
}

dt {
  padding: 3px;
  background: #65C400;
  color: #fff;
  font-weight: bold;
}

dd {
  margin: 5px 0 5px 5px;
  padding: 3px 3px 3px 18px;
}

dd .duration {
  padding-left: 5px;
  text-align: right;
  right: 0px;
  float:right;
}

dd.example.passed {
  border-left: 5px solid #65C400;
  border-bottom: 1px solid #65C400;
  background: #DBFFB4; color: #3D7700;
}

dd.example.not_implemented {
  border-left: 5px solid #FAF834;
  border-bottom: 1px solid #FAF834;
  background: #FCFB98; color: #131313;
}

dd.example.pending_fixed {
  border-left: 5px solid #0000C2;
  border-bottom: 1px solid #0000C2;
  color: #0000C2; background: #D3FBFF;
}

dd.example.failed {
  border-left: 5px solid #C20000;
  border-bottom: 1px solid #C20000;
  color: #C20000; background: #FFFBD3;
}


dt.not_implemented {
  color: #000000; background: #FAF834;
}

dt.pending_fixed {
  color: #FFFFFF; background: #C40D0D;
}

dt.failed {
  color: #FFFFFF; background: #C40D0D;
}


#rspec-header.not_implemented {
  color: #000000; background: #FAF834;
}

#rspec-header.pending_fixed {
  color: #FFFFFF; background: #C40D0D;
}

#rspec-header.failed {
  color: #FFFFFF; background: #C40D0D;
}


.backtrace {
  color: #000;
  font-size: 12px;
}

a {
  color: #BE5C00;
}

/* Ruby code, style similar to vibrant ink */
.ruby {
  font-size: 12px;
  font-family: monospace;
  color: white;
  background-color: black;
  padding: 0.1em 0 0.2em 0;
}

.ruby .keyword { color: #FF6600; }
.ruby .constant { color: #339999; }
.ruby .attribute { color: white; }
.ruby .global { color: white; }
.ruby .module { color: white; }
.ruby .class { color: white; }
.ruby .string { color: #66FF00; }
.ruby .ident { color: white; }
.ruby .method { color: #FFCC00; }
.ruby .number { color: white; }
.ruby .char { color: white; }
.ruby .comment { color: #9933CC; }
.ruby .symbol { color: white; }
.ruby .regex { color: #44B4CC; }
.ruby .punct { color: white; }
.ruby .escape { color: white; }
.ruby .interp { color: white; }
.ruby .expr { color: white; }

.ruby .offending { background-color: gray; }
.ruby .linenum {
  width: 75px;
  padding: 0.1em 1em 0.2em 0;
  color: #000000;
  background-color: #FFFBD3;
}

  </style>
</head>
<body>
<div class="rspec-report">

<div id="rspec-header">
  <div id="label">
    <h1>RSpec Code Examples</h1>
  </div>

  <div id="display-filters">
    <input id="passed_checkbox"  name="passed_checkbox"  type="checkbox" checked="checked" onchange="apply_filters()" value="1" /> <label for="passed_checkbox">Passed</label>
    <input id="failed_checkbox"  name="failed_checkbox"  type="checkbox" checked="checked" onchange="apply_filters()" value="2" /> <label for="failed_checkbox">Failed</label>
    <input id="pending_checkbox" name="pending_checkbox" type="checkbox" checked="checked" onchange="apply_filters()" value="3" /> <label for="pending_checkbox">Pending</label>
  </div>

  <div id="summary">
    <p id="totals">&#160;</p>
    <p id="duration">&#160;</p>
  </div>
</div>


<div class="results">
<div id="div_group_1" class="example_group passed">
  <dl style="margin-left: 0px;">
  <dt id="example_group_1" class="passed">Web Annotation Vocab</dt>
    <script type="text/javascript">moveProgressBar('0.3');</script>
    <dd class="example passed"><span class="passed_spec_name">The JSON-LD context document can be parsed without errors by JSON-LD validators</span><span class='duration'>0.30309s</span></dd>
  </dl>
</div>
<div id="div_group_2" class="example_group passed">
  <dl style="margin-left: 15px;">
  <dt id="example_group_2" class="passed">The JSON-LD context document can be used to convert JSON-LD serialized Annotations into RDF triples</dt>
    <script type="text/javascript">moveProgressBar('0.6');</script>
    <dd class="example passed"><span class="passed_spec_name">anno1.json</span><span class='duration'>0.56069s</span></dd>
    <script type="text/javascript">moveProgressBar('0.9');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno1.json</span><span class='duration'>0.42548s</span></dd>
    <script type="text/javascript">moveProgressBar('1.3');</script>
    <dd class="example passed"><span class="passed_spec_name">anno10.json</span><span class='duration'>0.39919s</span></dd>
    <script type="text/javascript">moveProgressBar('1.6');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno10.json</span><span class='duration'>0.48348s</span></dd>
    <script type="text/javascript">moveProgressBar('1.9');</script>
    <dd class="example passed"><span class="passed_spec_name">anno11.json</span><span class='duration'>0.41240s</span></dd>
    <script type="text/javascript">moveProgressBar('2.3');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno11.json</span><span class='duration'>0.53605s</span></dd>
    <script type="text/javascript">moveProgressBar('2.6');</script>
    <dd class="example passed"><span class="passed_spec_name">anno12.json</span><span class='duration'>0.40508s</span></dd>
    <script type="text/javascript">moveProgressBar('2.9');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno12.json</span><span class='duration'>0.49430s</span></dd>
    <script type="text/javascript">moveProgressBar('3.3');</script>
    <dd class="example passed"><span class="passed_spec_name">anno13.json</span><span class='duration'>0.40935s</span></dd>
    <script type="text/javascript">moveProgressBar('3.6');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno13.json</span><span class='duration'>0.48180s</span></dd>
    <script type="text/javascript">moveProgressBar('3.9');</script>
    <dd class="example passed"><span class="passed_spec_name">anno14.json</span><span class='duration'>0.41280s</span></dd>
    <script type="text/javascript">moveProgressBar('4.3');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno14.json</span><span class='duration'>0.47087s</span></dd>
    <script type="text/javascript">moveProgressBar('4.6');</script>
    <dd class="example passed"><span class="passed_spec_name">anno15.json</span><span class='duration'>0.40029s</span></dd>
    <script type="text/javascript">moveProgressBar('4.9');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno15.json</span><span class='duration'>0.51705s</span></dd>
    <script type="text/javascript">moveProgressBar('5.2');</script>
    <dd class="example passed"><span class="passed_spec_name">anno16.json</span><span class='duration'>0.39632s</span></dd>
    <script type="text/javascript">moveProgressBar('5.6');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno16.json</span><span class='duration'>0.46520s</span></dd>
    <script type="text/javascript">moveProgressBar('5.9');</script>
    <dd class="example passed"><span class="passed_spec_name">anno17.json</span><span class='duration'>0.39630s</span></dd>
    <script type="text/javascript">moveProgressBar('6.2');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno17.json</span><span class='duration'>0.43479s</span></dd>
    <script type="text/javascript">moveProgressBar('6.6');</script>
    <dd class="example passed"><span class="passed_spec_name">anno18.json</span><span class='duration'>0.40481s</span></dd>
    <script type="text/javascript">moveProgressBar('6.9');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno18.json</span><span class='duration'>0.46648s</span></dd>
    <script type="text/javascript">moveProgressBar('7.2');</script>
    <dd class="example passed"><span class="passed_spec_name">anno19.json</span><span class='duration'>0.39744s</span></dd>
    <script type="text/javascript">moveProgressBar('7.6');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno19.json</span><span class='duration'>0.43402s</span></dd>
    <script type="text/javascript">moveProgressBar('7.9');</script>
    <dd class="example passed"><span class="passed_spec_name">anno2.json</span><span class='duration'>0.40744s</span></dd>
    <script type="text/javascript">moveProgressBar('8.2');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno2.json</span><span class='duration'>0.44628s</span></dd>
    <script type="text/javascript">moveProgressBar('8.6');</script>
    <dd class="example passed"><span class="passed_spec_name">anno20.json</span><span class='duration'>0.41189s</span></dd>
    <script type="text/javascript">moveProgressBar('8.9');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno20.json</span><span class='duration'>0.43640s</span></dd>
    <script type="text/javascript">moveProgressBar('9.2');</script>
    <dd class="example passed"><span class="passed_spec_name">anno21.json</span><span class='duration'>0.39995s</span></dd>
    <script type="text/javascript">moveProgressBar('9.6');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno21.json</span><span class='duration'>0.46148s</span></dd>
    <script type="text/javascript">moveProgressBar('9.9');</script>
    <dd class="example passed"><span class="passed_spec_name">anno22.json</span><span class='duration'>0.40286s</span></dd>
    <script type="text/javascript">moveProgressBar('10.2');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno22.json</span><span class='duration'>0.44256s</span></dd>
    <script type="text/javascript">moveProgressBar('10.5');</script>
    <dd class="example passed"><span class="passed_spec_name">anno23.json</span><span class='duration'>0.41701s</span></dd>
    <script type="text/javascript">moveProgressBar('10.9');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno23.json</span><span class='duration'>0.47746s</span></dd>
    <script type="text/javascript">moveProgressBar('11.2');</script>
    <dd class="example passed"><span class="passed_spec_name">anno24.json</span><span class='duration'>0.41397s</span></dd>
    <script type="text/javascript">moveProgressBar('11.5');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno24.json</span><span class='duration'>0.44446s</span></dd>
    <script type="text/javascript">moveProgressBar('11.9');</script>
    <dd class="example passed"><span class="passed_spec_name">anno25.json</span><span class='duration'>0.39897s</span></dd>
    <script type="text/javascript">moveProgressBar('12.2');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno25.json</span><span class='duration'>0.44369s</span></dd>
    <script type="text/javascript">moveProgressBar('12.5');</script>
    <dd class="example passed"><span class="passed_spec_name">anno26.json</span><span class='duration'>0.40163s</span></dd>
    <script type="text/javascript">moveProgressBar('12.9');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno26.json</span><span class='duration'>0.44521s</span></dd>
    <script type="text/javascript">moveProgressBar('13.2');</script>
    <dd class="example passed"><span class="passed_spec_name">anno27.json</span><span class='duration'>0.40337s</span></dd>
    <script type="text/javascript">moveProgressBar('13.5');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno27.json</span><span class='duration'>0.46172s</span></dd>
    <script type="text/javascript">moveProgressBar('13.9');</script>
    <dd class="example passed"><span class="passed_spec_name">anno28.json</span><span class='duration'>0.40573s</span></dd>
    <script type="text/javascript">moveProgressBar('14.2');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno28.json</span><span class='duration'>0.44746s</span></dd>
    <script type="text/javascript">moveProgressBar('14.5');</script>
    <dd class="example passed"><span class="passed_spec_name">anno29.json</span><span class='duration'>0.40069s</span></dd>
    <script type="text/javascript">moveProgressBar('14.9');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno29.json</span><span class='duration'>0.44089s</span></dd>
    <script type="text/javascript">moveProgressBar('15.2');</script>
    <dd class="example passed"><span class="passed_spec_name">anno3.json</span><span class='duration'>0.39832s</span></dd>
    <script type="text/javascript">moveProgressBar('15.5');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno3.json</span><span class='duration'>0.44832s</span></dd>
    <script type="text/javascript">moveProgressBar('15.8');</script>
    <dd class="example passed"><span class="passed_spec_name">anno30.json</span><span class='duration'>0.40439s</span></dd>
    <script type="text/javascript">moveProgressBar('16.2');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno30.json</span><span class='duration'>0.47012s</span></dd>
    <script type="text/javascript">moveProgressBar('16.5');</script>
    <dd class="example passed"><span class="passed_spec_name">anno31.json</span><span class='duration'>0.40352s</span></dd>
    <script type="text/javascript">moveProgressBar('16.8');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno31.json</span><span class='duration'>0.48881s</span></dd>
    <script type="text/javascript">moveProgressBar('17.2');</script>
    <dd class="example passed"><span class="passed_spec_name">anno32.json</span><span class='duration'>0.40521s</span></dd>
    <script type="text/javascript">moveProgressBar('17.5');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno32.json</span><span class='duration'>0.47565s</span></dd>
    <script type="text/javascript">moveProgressBar('17.8');</script>
    <dd class="example passed"><span class="passed_spec_name">anno33.json</span><span class='duration'>0.39758s</span></dd>
    <script type="text/javascript">moveProgressBar('18.2');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno33.json</span><span class='duration'>0.42825s</span></dd>
    <script type="text/javascript">moveProgressBar('18.5');</script>
    <dd class="example passed"><span class="passed_spec_name">anno34.json</span><span class='duration'>0.40342s</span></dd>
    <script type="text/javascript">moveProgressBar('18.8');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno34.json</span><span class='duration'>0.46386s</span></dd>
    <script type="text/javascript">moveProgressBar('19.2');</script>
    <dd class="example passed"><span class="passed_spec_name">anno35.json</span><span class='duration'>0.39727s</span></dd>
    <script type="text/javascript">moveProgressBar('19.5');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno35.json</span><span class='duration'>0.44666s</span></dd>
    <script type="text/javascript">moveProgressBar('19.8');</script>
    <dd class="example passed"><span class="passed_spec_name">anno36.json</span><span class='duration'>0.40368s</span></dd>
    <script type="text/javascript">moveProgressBar('20.1');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno36.json</span><span class='duration'>0.50162s</span></dd>
    <script type="text/javascript">moveProgressBar('20.5');</script>
    <dd class="example passed"><span class="passed_spec_name">anno37.json</span><span class='duration'>0.40413s</span></dd>
    <script type="text/javascript">moveProgressBar('20.8');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno37.json</span><span class='duration'>0.44505s</span></dd>
    <script type="text/javascript">moveProgressBar('21.1');</script>
    <dd class="example passed"><span class="passed_spec_name">anno38.json</span><span class='duration'>0.40365s</span></dd>
    <script type="text/javascript">moveProgressBar('21.5');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno38.json</span><span class='duration'>0.44806s</span></dd>
    <script type="text/javascript">moveProgressBar('21.8');</script>
    <dd class="example passed"><span class="passed_spec_name">anno39.json</span><span class='duration'>0.39650s</span></dd>
    <script type="text/javascript">moveProgressBar('22.1');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno39.json</span><span class='duration'>0.45765s</span></dd>
    <script type="text/javascript">moveProgressBar('22.5');</script>
    <dd class="example passed"><span class="passed_spec_name">anno4.json</span><span class='duration'>0.40110s</span></dd>
    <script type="text/javascript">moveProgressBar('22.8');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno4.json</span><span class='duration'>0.43765s</span></dd>
    <script type="text/javascript">moveProgressBar('23.1');</script>
    <dd class="example passed"><span class="passed_spec_name">anno40.json</span><span class='duration'>0.39694s</span></dd>
    <script type="text/javascript">moveProgressBar('23.5');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno40.json</span><span class='duration'>0.42795s</span></dd>
    <script type="text/javascript">moveProgressBar('23.8');</script>
    <dd class="example passed"><span class="passed_spec_name">anno41-example44.json</span><span class='duration'>0.42444s</span></dd>
    <script type="text/javascript">moveProgressBar('24.1');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno41-example44.json</span><span class='duration'>0.82530s</span></dd>
    <script type="text/javascript">moveProgressBar('24.5');</script>
    <dd class="example passed"><span class="passed_spec_name">anno5.json</span><span class='duration'>0.40352s</span></dd>
    <script type="text/javascript">moveProgressBar('24.8');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno5.json</span><span class='duration'>0.43810s</span></dd>
    <script type="text/javascript">moveProgressBar('25.1');</script>
    <dd class="example passed"><span class="passed_spec_name">anno6.json</span><span class='duration'>0.40099s</span></dd>
    <script type="text/javascript">moveProgressBar('25.4');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno6.json</span><span class='duration'>0.41499s</span></dd>
    <script type="text/javascript">moveProgressBar('25.8');</script>
    <dd class="example passed"><span class="passed_spec_name">anno7.json</span><span class='duration'>0.39763s</span></dd>
    <script type="text/javascript">moveProgressBar('26.1');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno7.json</span><span class='duration'>0.45328s</span></dd>
    <script type="text/javascript">moveProgressBar('26.4');</script>
    <dd class="example passed"><span class="passed_spec_name">anno8.json</span><span class='duration'>0.40016s</span></dd>
    <script type="text/javascript">moveProgressBar('26.8');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno8.json</span><span class='duration'>0.42356s</span></dd>
    <script type="text/javascript">moveProgressBar('27.1');</script>
    <dd class="example passed"><span class="passed_spec_name">anno9.json</span><span class='duration'>0.40070s</span></dd>
    <script type="text/javascript">moveProgressBar('27.4');</script>
    <dd class="example passed"><span class="passed_spec_name">lint anno9.json</span><span class='duration'>0.43877s</span></dd>
    <script type="text/javascript">moveProgressBar('27.8');</script>
    <dd class="example passed"><span class="passed_spec_name">collection1.json</span><span class='duration'>0.75470s</span></dd>
    <script type="text/javascript">moveProgressBar('28.1');</script>
    <dd class="example passed"><span class="passed_spec_name">lint collection1.json</span><span class='duration'>3.15045s</span></dd>
    <script type="text/javascript">moveProgressBar('28.4');</script>
    <dd class="example passed"><span class="passed_spec_name">example41.json</span><span class='duration'>0.39996s</span></dd>
    <script type="text/javascript">moveProgressBar('28.8');</script>
    <dd class="example passed"><span class="passed_spec_name">lint example41.json</span><span class='duration'>0.43951s</span></dd>
    <script type="text/javascript">moveProgressBar('29.1');</script>
    <dd class="example passed"><span class="passed_spec_name">example42.json</span><span class='duration'>0.40507s</span></dd>
    <script type="text/javascript">moveProgressBar('29.4');</script>
    <dd class="example passed"><span class="passed_spec_name">lint example42.json</span><span class='duration'>0.47543s</span></dd>
    <script type="text/javascript">moveProgressBar('29.8');</script>
    <dd class="example passed"><span class="passed_spec_name">example43.json</span><span class='duration'>0.40551s</span></dd>
    <script type="text/javascript">moveProgressBar('30.1');</script>
    <dd class="example passed"><span class="passed_spec_name">lint example43.json</span><span class='duration'>0.50432s</span></dd>
  </dl>
</div>
<div id="div_group_3" class="example_group passed">
  <dl style="margin-left: 15px;">
  <dt id="example_group_3" class="passed">detects errors in incorrect examples</dt>
    <script type="text/javascript">moveProgressBar('30.4');</script>
    <dd class="example passed"><span class="passed_spec_name">anno1.json</span><span class='duration'>0.00142s</span></dd>
    <script type="text/javascript">moveProgressBar('30.7');</script>
    <dd class="example passed"><span class="passed_spec_name">anno10.json</span><span class='duration'>0.00076s</span></dd>
    <script type="text/javascript">moveProgressBar('31.1');</script>
    <dd class="example passed"><span class="passed_spec_name">anno11.json</span><span class='duration'>0.19856s</span></dd>
    <script type="text/javascript">moveProgressBar('31.4');</script>
    <dd class="example passed"><span class="passed_spec_name">anno12.json</span><span class='duration'>0.00087s</span></dd>
    <script type="text/javascript">moveProgressBar('31.7');</script>
    <dd class="example passed"><span class="passed_spec_name">anno13.json</span><span class='duration'>0.00075s</span></dd>
    <script type="text/javascript">moveProgressBar('32.1');</script>
    <dd class="example passed"><span class="passed_spec_name">anno14.json</span><span class='duration'>0.00077s</span></dd>
    <script type="text/javascript">moveProgressBar('32.4');</script>
    <dd class="example passed"><span class="passed_spec_name">anno15.json</span><span class='duration'>0.00116s</span></dd>
    <script type="text/javascript">moveProgressBar('32.7');</script>
    <dd class="example passed"><span class="passed_spec_name">anno16.json</span><span class='duration'>0.00074s</span></dd>
    <script type="text/javascript">moveProgressBar('33.1');</script>
    <dd class="example passed"><span class="passed_spec_name">anno17.json</span><span class='duration'>0.00075s</span></dd>
    <script type="text/javascript">moveProgressBar('33.4');</script>
    <dd class="example passed"><span class="passed_spec_name">anno18.json</span><span class='duration'>0.00077s</span></dd>
    <script type="text/javascript">moveProgressBar('33.7');</script>
    <dd class="example passed"><span class="passed_spec_name">anno19.json</span><span class='duration'>0.00071s</span></dd>
    <script type="text/javascript">makeYellow('rspec-header');</script>
    <script type="text/javascript">makeYellow('div_group_3');</script>
    <script type="text/javascript">makeYellow('example_group_3');</script>
    <script type="text/javascript">moveProgressBar('34.1');</script>
    <dd class="example not_implemented"><span class="not_implemented_spec_name">anno2.json (PENDING: Empty Documents are invalid)</span></dd>
    <script type="text/javascript">moveProgressBar('34.4');</script>
    <dd class="example passed"><span class="passed_spec_name">anno20.json</span><span class='duration'>0.00112s</span></dd>
    <script type="text/javascript">moveProgressBar('34.7');</script>
    <dd class="example passed"><span class="passed_spec_name">anno21.json</span><span class='duration'>0.00119s</span></dd>
    <script type="text/javascript">moveProgressBar('35.0');</script>
    <dd class="example passed"><span class="passed_spec_name">anno22.json</span><span class='duration'>0.00106s</span></dd>
    <script type="text/javascript">moveProgressBar('35.4');</script>
    <dd class="example passed"><span class="passed_spec_name">anno23.json</span><span class='duration'>0.00075s</span></dd>
    <script type="text/javascript">moveProgressBar('35.7');</script>
    <dd class="example passed"><span class="passed_spec_name">anno24.json</span><span class='duration'>0.00076s</span></dd>
    <script type="text/javascript">moveProgressBar('36.0');</script>
    <dd class="example passed"><span class="passed_spec_name">anno25.json</span><span class='duration'>0.00066s</span></dd>
    <script type="text/javascript">moveProgressBar('36.4');</script>
    <dd class="example passed"><span class="passed_spec_name">anno26.json</span><span class='duration'>0.19880s</span></dd>
    <script type="text/javascript">moveProgressBar('36.7');</script>
    <dd class="example passed"><span class="passed_spec_name">anno27.json</span><span class='duration'>0.20570s</span></dd>
    <script type="text/javascript">moveProgressBar('37.0');</script>
    <dd class="example passed"><span class="passed_spec_name">anno28.json</span><span class='duration'>0.19991s</span></dd>
    <script type="text/javascript">moveProgressBar('37.4');</script>
    <dd class="example passed"><span class="passed_spec_name">anno29.json</span><span class='duration'>0.19764s</span></dd>
    <script type="text/javascript">makeYellow('rspec-header');</script>
    <script type="text/javascript">makeYellow('div_group_3');</script>
    <script type="text/javascript">makeYellow('example_group_3');</script>
    <script type="text/javascript">moveProgressBar('37.7');</script>
    <dd class="example not_implemented"><span class="not_implemented_spec_name">anno3.json (PENDING: Empty Documents are invalid)</span></dd>
    <script type="text/javascript">moveProgressBar('38.0');</script>
    <dd class="example passed"><span class="passed_spec_name">anno30.json</span><span class='duration'>0.19944s</span></dd>
    <script type="text/javascript">moveProgressBar('38.4');</script>
    <dd class="example passed"><span class="passed_spec_name">anno31.json</span><span class='duration'>0.20036s</span></dd>
    <script type="text/javascript">moveProgressBar('38.7');</script>
    <dd class="example passed"><span class="passed_spec_name">anno32.json</span><span class='duration'>0.19468s</span></dd>
    <script type="text/javascript">moveProgressBar('39.0');</script>
    <dd class="example passed"><span class="passed_spec_name">anno33.json</span><span class='duration'>0.20145s</span></dd>
    <script type="text/javascript">moveProgressBar('39.4');</script>
    <dd class="example passed"><span class="passed_spec_name">anno34.json</span><span class='duration'>0.19786s</span></dd>
    <script type="text/javascript">moveProgressBar('39.7');</script>
    <dd class="example passed"><span class="passed_spec_name">anno35.json</span><span class='duration'>0.19442s</span></dd>
    <script type="text/javascript">moveProgressBar('40.0');</script>
    <dd class="example passed"><span class="passed_spec_name">anno36.json</span><span class='duration'>0.20170s</span></dd>
    <script type="text/javascript">moveProgressBar('40.3');</script>
    <dd class="example passed"><span class="passed_spec_name">anno37.json</span><span class='duration'>0.00073s</span></dd>
    <script type="text/javascript">moveProgressBar('40.7');</script>
    <dd class="example passed"><span class="passed_spec_name">anno38.json</span><span class='duration'>0.19785s</span></dd>
    <script type="text/javascript">moveProgressBar('41.0');</script>
    <dd class="example passed"><span class="passed_spec_name">anno39.json</span><span class='duration'>0.20030s</span></dd>
    <script type="text/javascript">moveProgressBar('41.3');</script>
    <dd class="example passed"><span class="passed_spec_name">anno4.json</span><span class='duration'>0.00247s</span></dd>
    <script type="text/javascript">moveProgressBar('41.7');</script>
    <dd class="example passed"><span class="passed_spec_name">anno5.json</span><span class='duration'>0.11420s</span></dd>
    <script type="text/javascript">moveProgressBar('42.0');</script>
    <dd class="example passed"><span class="passed_spec_name">anno6.json</span><span class='duration'>0.19756s</span></dd>
    <script type="text/javascript">moveProgressBar('42.3');</script>
    <dd class="example passed"><span class="passed_spec_name">anno7.json</span><span class='duration'>0.19826s</span></dd>
    <script type="text/javascript">moveProgressBar('42.7');</script>
    <dd class="example passed"><span class="passed_spec_name">anno8.json</span><span class='duration'>0.19797s</span></dd>
    <script type="text/javascript">moveProgressBar('43.0');</script>
    <dd class="example passed"><span class="passed_spec_name">anno9.json</span><span class='duration'>0.19496s</span></dd>
  </dl>
</div>
<div id="div_group_4" class="example_group passed">
  <dl style="margin-left: 15px;">
  <dt id="example_group_4" class="passed">The ontology documents can be parsed without errors by RDF Schema validators</dt>
    <script type="text/javascript">moveProgressBar('43.3');</script>
    <dd class="example passed"><span class="passed_spec_name">JSON-LD version is isomorphic to jsonld</span><span class='duration'>0.33990s</span></dd>
    <script type="text/javascript">moveProgressBar('43.7');</script>
    <dd class="example passed"><span class="passed_spec_name">JSON-LD version is isomorphic to rdfa</span><span class='duration'>0.24762s</span></dd>
    <script type="text/javascript">moveProgressBar('44.0');</script>
    <dd class="example passed"><span class="passed_spec_name">JSON-LD version is isomorphic to rdfxml</span><span class='duration'>0.38899s</span></dd>
    <script type="text/javascript">moveProgressBar('44.3');</script>
    <dd class="example passed"><span class="passed_spec_name">JSON-LD version is isomorphic to ttl</span><span class='duration'>0.47473s</span></dd>
  </dl>
</div>
<div id="div_group_5" class="example_group passed">
  <dl style="margin-left: 15px;">
  <dt id="example_group_5" class="passed">The ontology documents are isomorphic to each other</dt>
    <script type="text/javascript">makeRed('rspec-header');</script>
    <script type="text/javascript">makeRed('div_group_5');</script>
    <script type="text/javascript">makeRed('example_group_5');</script>
    <script type="text/javascript">moveProgressBar('44.7');</script>
    <dd class="example failed">
      <span class="failed_spec_name">rdfa</span>
      <span class="duration">0.44157s</span>
      <div class="failure" id="failure_1">
        <div class="message"><pre>Failure/Error: expect(fg).to be_equivalent_graph(vocab_graph)

  Graph entry counts differ:
  expected: 385
  actual:   24

  #&lt;struct RDF::Spec::Matchers::Info id=nil, logger=nil, action=nil, result=nil&gt;
  Expected:
  @prefix dc: &lt;http://purl.org/dc/terms/&gt; .
  @prefix dc11: &lt;http://purl.org/dc/elements/1.1/&gt; .
  @prefix oa: &lt;http://www.w3.org/ns/oa#&gt; .
  @prefix owl: &lt;http://www.w3.org/2002/07/owl#&gt; .
  @prefix rdf: &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#&gt; .
  @prefix rdfs: &lt;http://www.w3.org/2000/01/rdf-schema#&gt; .
  @prefix skos: &lt;http://www.w3.org/2004/02/skos/core#&gt; .
  @prefix xsd: &lt;http://www.w3.org/2001/XMLSchema#&gt; .

  oa:Annotation a rdfs:Class;
     rdfs:label &quot;Annotation&quot;;
     rdfs:comment &quot;The class for Web Annotations.&quot;;
     rdfs:isDefinedBy oa: .

  oa:Choice a rdfs:Class;
     rdfs:label &quot;Choice&quot;;
     rdfs:comment &quot;A subClass of as:OrderedCollection that conveys to a consuming application that it should select one of the resources in the as:items list to use, rather than all of them. This is typically used to provide a choice of resources to render to the user, based on further supplied properties. If the consuming application cannot determine the user&#39;s preference, then it should use the first in the list.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:as:OrderedCollection .

  oa:Composite a rdfs:Class;
     rdfs:label &quot;Composite&quot;;
     rdfs:comment &quot;A subClass of as:OrderedCollection that conveys to a consuming application that it should use all of the resources in the as:items list, but that order is not important. This class is at-risk.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:as:OrderedCollection .

  oa:CssSelector a rdfs:Class;
     rdfs:label &quot;CssSelector&quot;;
     rdfs:comment &quot;A CssSelector describes a Segment of interest in a representation that conforms to the Document Object Model through the use of the CSS selector specification.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:CssStyle a rdfs:Class;
     rdfs:label &quot;CssStyle&quot;;
     rdfs:comment &quot;A resource which describes styles for resources participating in the Annotation using CSS.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Style .

  oa:DataPositionSelector a rdfs:Class;
     rdfs:label &quot;DataPositionSelector&quot;;
     rdfs:comment &quot;DataPositionSelector describes a range of data by recording the start and end positions of the selection in the stream. Position 0 would be immediately before the first byte, position 1 would be immediately before the second byte, and so on. The start byte is thus included in the list, but the end byte is not.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:Direction a rdfs:Class;
     rdfs:label &quot;Direction&quot;;
     rdfs:comment &quot;A class to encapsulate the different text directions that a textual resource might take. It is not used directly in the Annotation Model, only its three instances.&quot;;
     rdfs:isDefinedBy oa: .

  oa:FragmentSelector a rdfs:Class;
     rdfs:label &quot;FragmentSelector&quot;;
     rdfs:comment &quot;The FragmentSelector class is used to record the segment of a representation using the IRI fragment specification defined by the representation&#39;s media type.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:HttpRequestState a rdfs:Class;
     rdfs:label &quot;HttpRequestState&quot;;
     rdfs:comment &quot;The HttpRequestState class is used to record the HTTP request headers that a client SHOULD use to request the correct representation from the resource.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:State .

  oa:Independents a rdfs:Class;
     rdfs:label &quot;Independents&quot;;
     rdfs:comment &quot;A subClass of as:OrderedCollection that conveys to a consuming application that each of the resources in the as:items list are independently associated with all of the other bodies or targets. This class is at-risk.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:as:OrderedCollection .

  oa:List a rdfs:Class;
     rdfs:label &quot;List&quot;;
     rdfs:comment &quot;A subClass of as:OrderedCollection that conveys to a consuming application that it should use each of the resources in the as:items list, and that their order is important. This class is at-risk.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:as:OrderedCollection .

  oa:Motivation a rdfs:Class;
     rdfs:label &quot;Motivation&quot;;
     rdfs:comment &quot;The Motivation class is used to record the user&#39;s intent or motivation for the creation of the Annotation, or the inclusion of the body or target, that it is associated with.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf skos:Concept .

  oa:RangeSelector a rdfs:Class;
     rdfs:label &quot;RangeSelector&quot;;
     rdfs:comment &quot;A Range Selector can be used to identify the beginning and the end of the selection by using other Selectors. The selection consists of everything from the beginning of the starting selector through to the beginning of the ending selector, but not including it.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:ResourceSelection a rdfs:Class;
     rdfs:label &quot;ResourceSelection&quot;;
     rdfs:comment &quot;Instances of the ResourceSelection class identify part (described by an oa:Selector) of another resource (referenced with oa:hasSource), possibly from a particular representation of a resource (described by an oa:State). Please note that ResourceSelection is not used directly in the Web Annotation model, but is provided as a separate class for further application profiles to use, separate from oa:SpecificResource which has many Annotation specific features.&quot;;
     rdfs:isDefinedBy oa: .

  oa:Selector a rdfs:Class;
     rdfs:label &quot;Selector&quot;;
     rdfs:comment &quot;A resource which describes the segment of interest in a representation of a Source resource, indicated with oa:hasSelector from the Specific Resource. This class is not used directly in the Annotation model, only its subclasses.&quot;;
     rdfs:isDefinedBy oa: .

  oa:SpecificResource a rdfs:Class;
     rdfs:label &quot;SpecificResource&quot;;
     rdfs:comment &quot;Instances of the SpecificResource class identify part of another resource (referenced with oa:hasSource), a particular representation of a resource, a resource with styling hints for renders, or any combination of these, as used within an Annotation.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:ResourceSelection .

  oa:State a rdfs:Class;
     rdfs:label &quot;State&quot;;
     rdfs:comment &quot;A State describes the intended state of a resource as applied to the particular Annotation, and thus provides the information needed to retrieve the correct representation of that resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:Style a rdfs:Class;
     rdfs:label &quot;Style&quot;;
     rdfs:comment &quot;A Style describes the intended styling of a resource as applied to the particular Annotation, and thus provides the information to ensure that rendering is consistent across implementations.&quot;;
     rdfs:isDefinedBy oa: .

  oa:SvgSelector a rdfs:Class;
     rdfs:label &quot;SvgSelector&quot;;
     rdfs:comment &quot;An SvgSelector defines an area through the use of the Scalable Vector Graphics [SVG] standard. This allows the user to select a non-rectangular area of the content, such as a circle or polygon by describing the region using SVG. The SVG may be either embedded within the Annotation or referenced as an External Resource.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:TextPositionSelector a rdfs:Class;
     rdfs:label &quot;TextPositionSelector&quot;;
     rdfs:comment &quot;The TextPositionSelector describes a range of text by recording the start and end positions of the selection in the stream. Position 0 would be immediately before the first character, position 1 would be immediately before the second character, and so on.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:TextQuoteSelector a rdfs:Class;
     rdfs:label &quot;TextQuoteSelector&quot;;
     rdfs:comment &quot;The TextQuoteSelector describes a range of text by copying it, and including some of the text immediately before (a prefix) and after (a suffix) it to distinguish between multiple copies of the same sequence of characters.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:TextualBody a rdfs:Class;
     rdfs:label &quot;TextualBody&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:TimeState a rdfs:Class;
     rdfs:label &quot;TimeState&quot;;
     rdfs:comment &quot;A TimeState records the time at which the resource&#39;s state is appropriate for the Annotation, typically the time that the Annotation was created and/or a link to a persistent copy of the current version.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:State .

  oa:XPathSelector a rdfs:Class;
     rdfs:label &quot;XPathSelector&quot;;
     rdfs:comment &quot;An XPathSelector is used to select elements and content within a resource that supports the Document Object Model via a specified XPath value.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:as:Application a rdfs:Class;
     rdfs:label &quot;as:Application&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:as:OrderedCollection a rdfs:Class;
     rdfs:label &quot;as:OrderedCollection&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:as:OrderedCollectionPage a rdfs:Class;
     rdfs:label &quot;as:OrderedCollectionPage&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:Dataset a rdfs:Class;
     rdfs:label &quot;dctypes:Dataset&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:MovingImage a rdfs:Class;
     rdfs:label &quot;dctypes:MovingImage&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:Sound a rdfs:Class;
     rdfs:label &quot;dctypes:Sound&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:StillImage a rdfs:Class;
     rdfs:label &quot;dctypes:StillImage&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:Text a rdfs:Class;
     rdfs:label &quot;dctypes:Text&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:foaf:Organization a rdfs:Class;
     rdfs:label &quot;foaf:Organization&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:foaf:Person a rdfs:Class;
     rdfs:label &quot;foaf:Person&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:schema:Audience a rdfs:Class;
     rdfs:label &quot;schema:Audience&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:PreferContainedDescriptions a rdfs:Resource;
     rdfs:label &quot;PreferContainedDescriptions&quot;;
     rdfs:comment &quot;An IRI to signal the client prefers to receive full descriptions of the Annotations from a container, not just their IRIs.&quot;;
     rdfs:isDefinedBy oa: .

  oa:PreferContainedIRIs a rdfs:Resource;
     rdfs:label &quot;PreferContainedIRIs&quot;;
     rdfs:comment &quot;An IRI to signal that the client prefers to receive only the IRIs of the Annotations from a container, not their full descriptions.&quot;;
     rdfs:isDefinedBy oa: .

  oa:annotationService a rdf:Property;
     rdfs:label &quot;annotationService&quot;;
     rdfs:comment &quot;The object of the relationship is the end point of a service that conforms to the annotation-protocol, and it may be associated with any resource. The expectation of asserting the relationship is that the object is the preferred service for maintaining annotations about the subject resource, according to the publisher of the relationship. This relationship is intended to be used both within Linked Data descriptions and as the rel type of a Link, via HTTP Link Headers rfc5988 for binary resources and in HTML &lt;link&gt; elements. For more information about these, please see the Annotation Protocol specification annotation-protocol.&quot;;
     rdfs:isDefinedBy oa: .

  oa:as:first a rdf:Property;
     rdfs:label &quot;as:first&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain oa:as:OrderedCollection;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt; .

  oa:as:generator a rdf:Property;
     rdfs:label &quot;as:generator&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:as:items a rdf:Property;
     rdfs:label &quot;as:items&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range rdf:List .

  oa:as:last a rdf:Property;
     rdfs:label &quot;as:last&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain oa:as:OrderedCollection;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt; .

  oa:as:next a rdf:Property;
     rdfs:label &quot;as:next&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt; .

  oa:as:partOf a rdf:Property;
     rdfs:label &quot;as:partOf&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollection&gt; .

  oa:as:prev a rdf:Property;
     rdfs:label &quot;as:prev&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt; .

  oa:as:startIndex a rdf:Property;
     rdfs:label &quot;as:startIndex&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:nonNegativeInteger .

  oa:as:totalItems a rdf:Property;
     rdfs:label &quot;as:totalItems&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollection&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:nonNegativeInteger .

  oa:assessing a oa:Motivation;
     rdfs:label &quot;assessing&quot;;
     rdfs:comment &quot;The motivation for when the user intends to provide an assessment about the Target resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:bodyValue a rdf:Property;
     rdfs:label &quot;bodyValue&quot;;
     rdfs:comment &quot;The object of the predicate is a plain text string to be used as the content of the body of the Annotation. The value MUST be an xsd:string and that data type MUST NOT be expressed in the serialization. Note that language MUST NOT be associated with the value either as a language tag, as that is only available for rdf:langString .&quot;;
     rdfs:domain oa:Annotation;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:string .

  oa:bookmarking a oa:Motivation;
     rdfs:label &quot;bookmarking&quot;;
     rdfs:comment &quot;The motivation for when the user intends to create a bookmark to the Target or part thereof.&quot;;
     rdfs:isDefinedBy oa: .

  oa:cachedSource a rdf:Property;
     rdfs:label &quot;cachedSource&quot;;
     rdfs:comment &quot;A object of the relationship is a copy of the Source resource&#39;s representation, appropriate for the Annotation.&quot;;
     rdfs:domain oa:TimeState;
     rdfs:isDefinedBy oa: .

  oa:canonical a rdf:Property;
     rdfs:label &quot;canonical&quot;;
     rdfs:comment &quot;A object of the relationship is the canonical IRI that can always be used to deduplicate the Annotation, regardless of the current IRI used to access the representation.&quot;;
     rdfs:isDefinedBy oa: .

  oa:classifying a oa:Motivation;
     rdfs:label &quot;classifying&quot;;
     rdfs:comment &quot;The motivation for when the user intends to that classify the Target as something.&quot;;
     rdfs:isDefinedBy oa: .

  oa:commenting a oa:Motivation;
     rdfs:label &quot;commenting&quot;;
     rdfs:comment &quot;The motivation for when the user intends to comment about the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:describing a oa:Motivation;
     rdfs:label &quot;describing&quot;;
     rdfs:comment &quot;The motivation for when the user intends to describe the Target, as opposed to a comment about them.&quot;;
     rdfs:isDefinedBy oa: .

  oa:editing a oa:Motivation;
     rdfs:label &quot;editing&quot;;
     rdfs:comment &quot;The motivation for when the user intends to request a change or edit to the Target resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:end a rdf:Property;
     rdfs:label &quot;end&quot;;
     rdfs:comment &quot;The end property is used to convey the 0-based index of the end position of a range of content.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:nonNegativeInteger .

  oa:exact a rdf:Property;
     rdfs:label &quot;exact&quot;;
     rdfs:comment &quot;The object of the predicate is a copy of the text which is being selected, after normalization.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:string .

  oa:hasBody a rdf:Property;
     rdfs:label &quot;hasBody&quot;;
     rdfs:comment &quot;The object of the relationship is a resource that is a body of the Annotation.&quot;;
     rdfs:domain oa:Annotation;
     rdfs:isDefinedBy oa: .

  oa:hasEndSelector a rdf:Property;
     rdfs:label &quot;hasEndSelector&quot;;
     rdfs:comment &quot;The relationship between a RangeSelector and the Selector that describes the end position of the range.&quot;;
     rdfs:domain oa:RangeSelector;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Selector .

  oa:hasPurpose a rdf:Property;
     rdfs:label &quot;hasPurpose&quot;;
     rdfs:comment &quot;The purpose served by the resource in the Annotation.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Motivation .

  oa:hasScope a rdf:Property;
     rdfs:label &quot;hasScope&quot;;
     rdfs:comment &quot;The scope or context in which the resource is used within the Annotation.&quot;;
     rdfs:domain oa:SpecificResource;
     rdfs:isDefinedBy oa: .

  oa:hasSelector a rdf:Property;
     rdfs:label &quot;hasSelector&quot;;
     rdfs:comment &quot;The object of the relationship is a Selector that describes the segment or region of interest within the source resource. Please note that the domain ( oa:ResourceSelection ) is not used directly in the Web Annotation model.&quot;;
     rdfs:domain oa:ResourceSelection;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Selector .

  oa:hasSource a rdf:Property;
     rdfs:label &quot;hasSource&quot;;
     rdfs:comment &quot;The resource that the ResourceSelection, or its subclass SpecificResource, is refined from, or more specific than. Please note that the domain ( oa:ResourceSelection ) is not used directly in the Web Annotation model.&quot;;
     rdfs:domain oa:ResourceSelection;
     rdfs:isDefinedBy oa: .

  oa:hasStartSelector a rdf:Property;
     rdfs:label &quot;hasStartSelector&quot;;
     rdfs:comment &quot;The relationship between a RangeSelector and the Selector that describes the start position of the range.&quot;;
     rdfs:domain oa:RangeSelector;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Selector .

  oa:hasState a rdf:Property;
     rdfs:label &quot;hasState&quot;;
     rdfs:comment &quot;The relationship between the ResourceSelection, or its subclass SpecificResource, and a State resource. Please note that the domain ( oa:ResourceSelection ) is not used directly in the Web Annotation model.&quot;;
     rdfs:domain oa:ResourceSelection;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:State .

  oa:hasTarget a rdf:Property;
     rdfs:label &quot;hasTarget&quot;;
     rdfs:comment &quot;The relationship between an Annotation and its Target.&quot;;
     rdfs:domain oa:Annotation;
     rdfs:isDefinedBy oa: .

  oa:highlighting a oa:Motivation;
     rdfs:label &quot;highlighting&quot;;
     rdfs:comment &quot;The motivation for when the user intends to highlight the Target resource or segment of it.&quot;;
     rdfs:isDefinedBy oa: .

  oa:identifying a oa:Motivation;
     rdfs:label &quot;identifying&quot;;
     rdfs:comment &quot;The motivation for when the user intends to assign an identity to the Target or identify what is being depicted or described in the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:linking a oa:Motivation;
     rdfs:label &quot;linking&quot;;
     rdfs:comment &quot;The motivation for when the user intends to link to a resource related to the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:ltrDirection a oa:Direction;
     rdfs:label &quot;ltrDirection&quot;;
     rdfs:comment &quot;The direction of text that is read from left to right.&quot;;
     rdfs:isDefinedBy oa: .

  oa:moderating a oa:Motivation;
     rdfs:label &quot;moderating&quot;;
     rdfs:comment &quot;The motivation for when the user intends to assign some value or quality to the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:motivatedBy a rdf:Property;
     rdfs:label &quot;motivatedBy&quot;;
     rdfs:comment &quot;The relationship between an Annotation and a Motivation that describes the reason for the Annotation&#39;s creation.&quot;;
     rdfs:domain oa:Annotation;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Motivation .

  oa:prefix a rdf:Property;
     rdfs:label &quot;prefix&quot;;
     rdfs:comment &quot;The object of the property is a snippet of content that occurs immediately before the content which is being selected by the Selector.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:string .

  oa:processingLanguage a rdf:Property;
     rdfs:label &quot;processingLanguage&quot;;
     rdfs:comment &quot;The object of the property is the language that should be used for textual processing algorithms when dealing with the content of the resource, including hyphenation, line breaking, which font to use for rendering and so forth. The value must follow the recommendations of BCP47.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:string .

  oa:questioning a oa:Motivation;
     rdfs:label &quot;questioning&quot;;
     rdfs:comment &quot;The motivation for when the user intends to ask a question about the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:refinedBy a rdf:Property;
     rdfs:label &quot;refinedBy&quot;;
     rdfs:comment &quot;The relationship between a Selector and another Selector or a State and a Selector or State that should be applied to the results of the first to refine the processing of the source resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:replying a oa:Motivation;
     rdfs:label &quot;replying&quot;;
     rdfs:comment &quot;The motivation for when the user intends to reply to a previous statement, either an Annotation or another resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:rtlDirection a oa:Direction;
     rdfs:label &quot;rtlDirection&quot;;
     rdfs:comment &quot;The direction of text that is read from right to left.&quot;;
     rdfs:isDefinedBy oa: .

  oa:tagging a oa:Motivation;
     rdfs:label &quot;tagging&quot;;
     rdfs:comment &quot;The motivation for when the user intends to associate a tag with the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa: a owl:Ontology;
     dc11:title &quot;Web Annotation Ontology&quot;;
     dc:creator &quot;Robert Sanderson&quot;,
       &quot;Paolo Ciccarese&quot;,
       &quot;Benjamin Young&quot;;
     dc:modified &quot;2016-09-30T16:51:18Z&quot;;
     rdfs:comment &quot;The Web Annotation ontology defines the terms of the Web Annotation vocabulary. Any changes to this document MUST be from a Working Group in the W3C that has established expertise in the area.&quot;;
     rdfs:seeAlso &lt;http://www.w3.org/TR/annotation-vocab/&gt;;
     owl:previousVersionURI &lt;http://www.openannotation.org/spec/core/20130208/oa.owl&gt;;
     owl:versionInfo &quot;2016-09-30T16:51:18Z&quot; .
  Results:
  @prefix dc: &lt;http://purl.org/dc/terms/&gt; .
  @prefix foaf: &lt;http://xmlns.com/foaf/0.1/&gt; .
  @prefix oa: &lt;http://www.w3.org/ns/oa#&gt; .
  @prefix rdf: &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#&gt; .
  @prefix xhv: &lt;http://www.w3.org/1999/xhtml/vocab#&gt; .
  @prefix xsd: &lt;http://www.w3.org/2001/XMLSchema#&gt; .

  &lt;http://www.w3.org/ns/oa&gt; dc:title &quot;Web Annotation Ontology&quot;;
     &lt;bibo:editor&gt; [];
     xhv:license &lt;http://www.w3.org/Consortium/Legal/copyright-documents&gt; .

  oa:respecDocument xhv:role xhv:document .

  oa:respecHeader xhv:role xhv:contentinfo .

  ([
     a foaf:Person;
     foaf:homepage &lt;http://www.paolociccarese.info&gt;;
     foaf:mbox &lt;mailto:paolo.ciccarese@gmail.com&gt;;
     foaf:name &quot;Paolo Ciccarese&quot;
   ] [
     a foaf:Person;
     foaf:homepage &lt;http://bigbluehat.com/&gt;;
     foaf:mbox &lt;mailto:byoung@bigbluehat.com&gt;;
     foaf:name &quot;Benjamin Young&quot;
   ]) .

  _:g70220666792480 a foaf:Person;
     foaf:homepage &lt;http://www.stanford.edu/~azaroth/&gt;;
     foaf:mbox &lt;mailto:azaroth42@gmail.com&gt;;
     foaf:name &quot;Robert Sanderson&quot;;
     foaf:workplaceHomepage &lt;http://www.stanford.edu/&gt; .

  Debug:</pre></div>
        <div class="backtrace"><pre>./annotation-vocab_spec.rb:83:in `block (4 levels) in &lt;top (required)&gt;&#39;</pre></div>
    <pre class="ruby"><code><span class="linenum">81</span>        # XXX Normalize whitespace in literals to ease comparision
<span class="linenum">82</span>        fg.each_object {|o| o.squish! if o.literal?}
<span class="offending"><span class="linenum">83</span>        expect(fg).to be_equivalent_graph(vocab_graph)</span>
<span class="linenum">84</span>      end
<span class="linenum">85</span>    end
<span class="linenum">86</span><span class="comment"># Install the coderay gem to get syntax highlighting</span></code></pre>
      </div>
    </dd>
    <script type="text/javascript">moveProgressBar('45.0');</script>
    <dd class="example failed">
      <span class="failed_spec_name">rdfxml</span>
      <span class="duration">0.80222s</span>
      <div class="failure" id="failure_2">
        <div class="message"><pre>Failure/Error: expect(fg).to be_equivalent_graph(vocab_graph)

  Graphs differ

  #&lt;struct RDF::Spec::Matchers::Info id=nil, logger=nil, action=nil, result=nil&gt;
  Expected:
  @prefix dc: &lt;http://purl.org/dc/terms/&gt; .
  @prefix dc11: &lt;http://purl.org/dc/elements/1.1/&gt; .
  @prefix oa: &lt;http://www.w3.org/ns/oa#&gt; .
  @prefix owl: &lt;http://www.w3.org/2002/07/owl#&gt; .
  @prefix rdf: &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#&gt; .
  @prefix rdfs: &lt;http://www.w3.org/2000/01/rdf-schema#&gt; .
  @prefix skos: &lt;http://www.w3.org/2004/02/skos/core#&gt; .
  @prefix xsd: &lt;http://www.w3.org/2001/XMLSchema#&gt; .

  oa:Annotation a rdfs:Class;
     rdfs:label &quot;Annotation&quot;;
     rdfs:comment &quot;The class for Web Annotations.&quot;;
     rdfs:isDefinedBy oa: .

  oa:Choice a rdfs:Class;
     rdfs:label &quot;Choice&quot;;
     rdfs:comment &quot;A subClass of as:OrderedCollection that conveys to a consuming application that it should select one of the resources in the as:items list to use, rather than all of them. This is typically used to provide a choice of resources to render to the user, based on further supplied properties. If the consuming application cannot determine the user&#39;s preference, then it should use the first in the list.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:as:OrderedCollection .

  oa:Composite a rdfs:Class;
     rdfs:label &quot;Composite&quot;;
     rdfs:comment &quot;A subClass of as:OrderedCollection that conveys to a consuming application that it should use all of the resources in the as:items list, but that order is not important. This class is at-risk.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:as:OrderedCollection .

  oa:CssSelector a rdfs:Class;
     rdfs:label &quot;CssSelector&quot;;
     rdfs:comment &quot;A CssSelector describes a Segment of interest in a representation that conforms to the Document Object Model through the use of the CSS selector specification.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:CssStyle a rdfs:Class;
     rdfs:label &quot;CssStyle&quot;;
     rdfs:comment &quot;A resource which describes styles for resources participating in the Annotation using CSS.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Style .

  oa:DataPositionSelector a rdfs:Class;
     rdfs:label &quot;DataPositionSelector&quot;;
     rdfs:comment &quot;DataPositionSelector describes a range of data by recording the start and end positions of the selection in the stream. Position 0 would be immediately before the first byte, position 1 would be immediately before the second byte, and so on. The start byte is thus included in the list, but the end byte is not.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:Direction a rdfs:Class;
     rdfs:label &quot;Direction&quot;;
     rdfs:comment &quot;A class to encapsulate the different text directions that a textual resource might take. It is not used directly in the Annotation Model, only its three instances.&quot;;
     rdfs:isDefinedBy oa: .

  oa:FragmentSelector a rdfs:Class;
     rdfs:label &quot;FragmentSelector&quot;;
     rdfs:comment &quot;The FragmentSelector class is used to record the segment of a representation using the IRI fragment specification defined by the representation&#39;s media type.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:HttpRequestState a rdfs:Class;
     rdfs:label &quot;HttpRequestState&quot;;
     rdfs:comment &quot;The HttpRequestState class is used to record the HTTP request headers that a client SHOULD use to request the correct representation from the resource.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:State .

  oa:Independents a rdfs:Class;
     rdfs:label &quot;Independents&quot;;
     rdfs:comment &quot;A subClass of as:OrderedCollection that conveys to a consuming application that each of the resources in the as:items list are independently associated with all of the other bodies or targets. This class is at-risk.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:as:OrderedCollection .

  oa:List a rdfs:Class;
     rdfs:label &quot;List&quot;;
     rdfs:comment &quot;A subClass of as:OrderedCollection that conveys to a consuming application that it should use each of the resources in the as:items list, and that their order is important. This class is at-risk.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:as:OrderedCollection .

  oa:Motivation a rdfs:Class;
     rdfs:label &quot;Motivation&quot;;
     rdfs:comment &quot;The Motivation class is used to record the user&#39;s intent or motivation for the creation of the Annotation, or the inclusion of the body or target, that it is associated with.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf skos:Concept .

  oa:RangeSelector a rdfs:Class;
     rdfs:label &quot;RangeSelector&quot;;
     rdfs:comment &quot;A Range Selector can be used to identify the beginning and the end of the selection by using other Selectors. The selection consists of everything from the beginning of the starting selector through to the beginning of the ending selector, but not including it.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:ResourceSelection a rdfs:Class;
     rdfs:label &quot;ResourceSelection&quot;;
     rdfs:comment &quot;Instances of the ResourceSelection class identify part (described by an oa:Selector) of another resource (referenced with oa:hasSource), possibly from a particular representation of a resource (described by an oa:State). Please note that ResourceSelection is not used directly in the Web Annotation model, but is provided as a separate class for further application profiles to use, separate from oa:SpecificResource which has many Annotation specific features.&quot;;
     rdfs:isDefinedBy oa: .

  oa:Selector a rdfs:Class;
     rdfs:label &quot;Selector&quot;;
     rdfs:comment &quot;A resource which describes the segment of interest in a representation of a Source resource, indicated with oa:hasSelector from the Specific Resource. This class is not used directly in the Annotation model, only its subclasses.&quot;;
     rdfs:isDefinedBy oa: .

  oa:SpecificResource a rdfs:Class;
     rdfs:label &quot;SpecificResource&quot;;
     rdfs:comment &quot;Instances of the SpecificResource class identify part of another resource (referenced with oa:hasSource), a particular representation of a resource, a resource with styling hints for renders, or any combination of these, as used within an Annotation.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:ResourceSelection .

  oa:State a rdfs:Class;
     rdfs:label &quot;State&quot;;
     rdfs:comment &quot;A State describes the intended state of a resource as applied to the particular Annotation, and thus provides the information needed to retrieve the correct representation of that resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:Style a rdfs:Class;
     rdfs:label &quot;Style&quot;;
     rdfs:comment &quot;A Style describes the intended styling of a resource as applied to the particular Annotation, and thus provides the information to ensure that rendering is consistent across implementations.&quot;;
     rdfs:isDefinedBy oa: .

  oa:SvgSelector a rdfs:Class;
     rdfs:label &quot;SvgSelector&quot;;
     rdfs:comment &quot;An SvgSelector defines an area through the use of the Scalable Vector Graphics [SVG] standard. This allows the user to select a non-rectangular area of the content, such as a circle or polygon by describing the region using SVG. The SVG may be either embedded within the Annotation or referenced as an External Resource.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:TextPositionSelector a rdfs:Class;
     rdfs:label &quot;TextPositionSelector&quot;;
     rdfs:comment &quot;The TextPositionSelector describes a range of text by recording the start and end positions of the selection in the stream. Position 0 would be immediately before the first character, position 1 would be immediately before the second character, and so on.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:TextQuoteSelector a rdfs:Class;
     rdfs:label &quot;TextQuoteSelector&quot;;
     rdfs:comment &quot;The TextQuoteSelector describes a range of text by copying it, and including some of the text immediately before (a prefix) and after (a suffix) it to distinguish between multiple copies of the same sequence of characters.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:TextualBody a rdfs:Class;
     rdfs:label &quot;TextualBody&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:TimeState a rdfs:Class;
     rdfs:label &quot;TimeState&quot;;
     rdfs:comment &quot;A TimeState records the time at which the resource&#39;s state is appropriate for the Annotation, typically the time that the Annotation was created and/or a link to a persistent copy of the current version.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:State .

  oa:XPathSelector a rdfs:Class;
     rdfs:label &quot;XPathSelector&quot;;
     rdfs:comment &quot;An XPathSelector is used to select elements and content within a resource that supports the Document Object Model via a specified XPath value.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:as:Application a rdfs:Class;
     rdfs:label &quot;as:Application&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:as:OrderedCollection a rdfs:Class;
     rdfs:label &quot;as:OrderedCollection&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:as:OrderedCollectionPage a rdfs:Class;
     rdfs:label &quot;as:OrderedCollectionPage&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:Dataset a rdfs:Class;
     rdfs:label &quot;dctypes:Dataset&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:MovingImage a rdfs:Class;
     rdfs:label &quot;dctypes:MovingImage&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:Sound a rdfs:Class;
     rdfs:label &quot;dctypes:Sound&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:StillImage a rdfs:Class;
     rdfs:label &quot;dctypes:StillImage&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:Text a rdfs:Class;
     rdfs:label &quot;dctypes:Text&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:foaf:Organization a rdfs:Class;
     rdfs:label &quot;foaf:Organization&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:foaf:Person a rdfs:Class;
     rdfs:label &quot;foaf:Person&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:schema:Audience a rdfs:Class;
     rdfs:label &quot;schema:Audience&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:PreferContainedDescriptions a rdfs:Resource;
     rdfs:label &quot;PreferContainedDescriptions&quot;;
     rdfs:comment &quot;An IRI to signal the client prefers to receive full descriptions of the Annotations from a container, not just their IRIs.&quot;;
     rdfs:isDefinedBy oa: .

  oa:PreferContainedIRIs a rdfs:Resource;
     rdfs:label &quot;PreferContainedIRIs&quot;;
     rdfs:comment &quot;An IRI to signal that the client prefers to receive only the IRIs of the Annotations from a container, not their full descriptions.&quot;;
     rdfs:isDefinedBy oa: .

  oa:annotationService a rdf:Property;
     rdfs:label &quot;annotationService&quot;;
     rdfs:comment &quot;The object of the relationship is the end point of a service that conforms to the annotation-protocol, and it may be associated with any resource. The expectation of asserting the relationship is that the object is the preferred service for maintaining annotations about the subject resource, according to the publisher of the relationship. This relationship is intended to be used both within Linked Data descriptions and as the rel type of a Link, via HTTP Link Headers rfc5988 for binary resources and in HTML &lt;link&gt; elements. For more information about these, please see the Annotation Protocol specification annotation-protocol.&quot;;
     rdfs:isDefinedBy oa: .

  oa:as:first a rdf:Property;
     rdfs:label &quot;as:first&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain oa:as:OrderedCollection;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt; .

  oa:as:generator a rdf:Property;
     rdfs:label &quot;as:generator&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:as:items a rdf:Property;
     rdfs:label &quot;as:items&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range rdf:List .

  oa:as:last a rdf:Property;
     rdfs:label &quot;as:last&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain oa:as:OrderedCollection;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt; .

  oa:as:next a rdf:Property;
     rdfs:label &quot;as:next&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt; .

  oa:as:partOf a rdf:Property;
     rdfs:label &quot;as:partOf&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollection&gt; .

  oa:as:prev a rdf:Property;
     rdfs:label &quot;as:prev&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt; .

  oa:as:startIndex a rdf:Property;
     rdfs:label &quot;as:startIndex&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:nonNegativeInteger .

  oa:as:totalItems a rdf:Property;
     rdfs:label &quot;as:totalItems&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollection&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:nonNegativeInteger .

  oa:assessing a oa:Motivation;
     rdfs:label &quot;assessing&quot;;
     rdfs:comment &quot;The motivation for when the user intends to provide an assessment about the Target resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:bodyValue a rdf:Property;
     rdfs:label &quot;bodyValue&quot;;
     rdfs:comment &quot;The object of the predicate is a plain text string to be used as the content of the body of the Annotation. The value MUST be an xsd:string and that data type MUST NOT be expressed in the serialization. Note that language MUST NOT be associated with the value either as a language tag, as that is only available for rdf:langString .&quot;;
     rdfs:domain oa:Annotation;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:string .

  oa:bookmarking a oa:Motivation;
     rdfs:label &quot;bookmarking&quot;;
     rdfs:comment &quot;The motivation for when the user intends to create a bookmark to the Target or part thereof.&quot;;
     rdfs:isDefinedBy oa: .

  oa:cachedSource a rdf:Property;
     rdfs:label &quot;cachedSource&quot;;
     rdfs:comment &quot;A object of the relationship is a copy of the Source resource&#39;s representation, appropriate for the Annotation.&quot;;
     rdfs:domain oa:TimeState;
     rdfs:isDefinedBy oa: .

  oa:canonical a rdf:Property;
     rdfs:label &quot;canonical&quot;;
     rdfs:comment &quot;A object of the relationship is the canonical IRI that can always be used to deduplicate the Annotation, regardless of the current IRI used to access the representation.&quot;;
     rdfs:isDefinedBy oa: .

  oa:classifying a oa:Motivation;
     rdfs:label &quot;classifying&quot;;
     rdfs:comment &quot;The motivation for when the user intends to that classify the Target as something.&quot;;
     rdfs:isDefinedBy oa: .

  oa:commenting a oa:Motivation;
     rdfs:label &quot;commenting&quot;;
     rdfs:comment &quot;The motivation for when the user intends to comment about the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:describing a oa:Motivation;
     rdfs:label &quot;describing&quot;;
     rdfs:comment &quot;The motivation for when the user intends to describe the Target, as opposed to a comment about them.&quot;;
     rdfs:isDefinedBy oa: .

  oa:editing a oa:Motivation;
     rdfs:label &quot;editing&quot;;
     rdfs:comment &quot;The motivation for when the user intends to request a change or edit to the Target resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:end a rdf:Property;
     rdfs:label &quot;end&quot;;
     rdfs:comment &quot;The end property is used to convey the 0-based index of the end position of a range of content.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:nonNegativeInteger .

  oa:exact a rdf:Property;
     rdfs:label &quot;exact&quot;;
     rdfs:comment &quot;The object of the predicate is a copy of the text which is being selected, after normalization.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:string .

  oa:hasBody a rdf:Property;
     rdfs:label &quot;hasBody&quot;;
     rdfs:comment &quot;The object of the relationship is a resource that is a body of the Annotation.&quot;;
     rdfs:domain oa:Annotation;
     rdfs:isDefinedBy oa: .

  oa:hasEndSelector a rdf:Property;
     rdfs:label &quot;hasEndSelector&quot;;
     rdfs:comment &quot;The relationship between a RangeSelector and the Selector that describes the end position of the range.&quot;;
     rdfs:domain oa:RangeSelector;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Selector .

  oa:hasPurpose a rdf:Property;
     rdfs:label &quot;hasPurpose&quot;;
     rdfs:comment &quot;The purpose served by the resource in the Annotation.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Motivation .

  oa:hasScope a rdf:Property;
     rdfs:label &quot;hasScope&quot;;
     rdfs:comment &quot;The scope or context in which the resource is used within the Annotation.&quot;;
     rdfs:domain oa:SpecificResource;
     rdfs:isDefinedBy oa: .

  oa:hasSelector a rdf:Property;
     rdfs:label &quot;hasSelector&quot;;
     rdfs:comment &quot;The object of the relationship is a Selector that describes the segment or region of interest within the source resource. Please note that the domain ( oa:ResourceSelection ) is not used directly in the Web Annotation model.&quot;;
     rdfs:domain oa:ResourceSelection;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Selector .

  oa:hasSource a rdf:Property;
     rdfs:label &quot;hasSource&quot;;
     rdfs:comment &quot;The resource that the ResourceSelection, or its subclass SpecificResource, is refined from, or more specific than. Please note that the domain ( oa:ResourceSelection ) is not used directly in the Web Annotation model.&quot;;
     rdfs:domain oa:ResourceSelection;
     rdfs:isDefinedBy oa: .

  oa:hasStartSelector a rdf:Property;
     rdfs:label &quot;hasStartSelector&quot;;
     rdfs:comment &quot;The relationship between a RangeSelector and the Selector that describes the start position of the range.&quot;;
     rdfs:domain oa:RangeSelector;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Selector .

  oa:hasState a rdf:Property;
     rdfs:label &quot;hasState&quot;;
     rdfs:comment &quot;The relationship between the ResourceSelection, or its subclass SpecificResource, and a State resource. Please note that the domain ( oa:ResourceSelection ) is not used directly in the Web Annotation model.&quot;;
     rdfs:domain oa:ResourceSelection;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:State .

  oa:hasTarget a rdf:Property;
     rdfs:label &quot;hasTarget&quot;;
     rdfs:comment &quot;The relationship between an Annotation and its Target.&quot;;
     rdfs:domain oa:Annotation;
     rdfs:isDefinedBy oa: .

  oa:highlighting a oa:Motivation;
     rdfs:label &quot;highlighting&quot;;
     rdfs:comment &quot;The motivation for when the user intends to highlight the Target resource or segment of it.&quot;;
     rdfs:isDefinedBy oa: .

  oa:identifying a oa:Motivation;
     rdfs:label &quot;identifying&quot;;
     rdfs:comment &quot;The motivation for when the user intends to assign an identity to the Target or identify what is being depicted or described in the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:linking a oa:Motivation;
     rdfs:label &quot;linking&quot;;
     rdfs:comment &quot;The motivation for when the user intends to link to a resource related to the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:ltrDirection a oa:Direction;
     rdfs:label &quot;ltrDirection&quot;;
     rdfs:comment &quot;The direction of text that is read from left to right.&quot;;
     rdfs:isDefinedBy oa: .

  oa:moderating a oa:Motivation;
     rdfs:label &quot;moderating&quot;;
     rdfs:comment &quot;The motivation for when the user intends to assign some value or quality to the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:motivatedBy a rdf:Property;
     rdfs:label &quot;motivatedBy&quot;;
     rdfs:comment &quot;The relationship between an Annotation and a Motivation that describes the reason for the Annotation&#39;s creation.&quot;;
     rdfs:domain oa:Annotation;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Motivation .

  oa:prefix a rdf:Property;
     rdfs:label &quot;prefix&quot;;
     rdfs:comment &quot;The object of the property is a snippet of content that occurs immediately before the content which is being selected by the Selector.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:string .

  oa:processingLanguage a rdf:Property;
     rdfs:label &quot;processingLanguage&quot;;
     rdfs:comment &quot;The object of the property is the language that should be used for textual processing algorithms when dealing with the content of the resource, including hyphenation, line breaking, which font to use for rendering and so forth. The value must follow the recommendations of BCP47.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:string .

  oa:questioning a oa:Motivation;
     rdfs:label &quot;questioning&quot;;
     rdfs:comment &quot;The motivation for when the user intends to ask a question about the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:refinedBy a rdf:Property;
     rdfs:label &quot;refinedBy&quot;;
     rdfs:comment &quot;The relationship between a Selector and another Selector or a State and a Selector or State that should be applied to the results of the first to refine the processing of the source resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:replying a oa:Motivation;
     rdfs:label &quot;replying&quot;;
     rdfs:comment &quot;The motivation for when the user intends to reply to a previous statement, either an Annotation or another resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:rtlDirection a oa:Direction;
     rdfs:label &quot;rtlDirection&quot;;
     rdfs:comment &quot;The direction of text that is read from right to left.&quot;;
     rdfs:isDefinedBy oa: .

  oa:tagging a oa:Motivation;
     rdfs:label &quot;tagging&quot;;
     rdfs:comment &quot;The motivation for when the user intends to associate a tag with the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa: a owl:Ontology;
     dc11:title &quot;Web Annotation Ontology&quot;;
     dc:creator &quot;Robert Sanderson&quot;,
       &quot;Paolo Ciccarese&quot;,
       &quot;Benjamin Young&quot;;
     dc:modified &quot;2016-09-30T16:51:18Z&quot;;
     rdfs:comment &quot;The Web Annotation ontology defines the terms of the Web Annotation vocabulary. Any changes to this document MUST be from a Working Group in the W3C that has established expertise in the area.&quot;;
     rdfs:seeAlso &lt;http://www.w3.org/TR/annotation-vocab/&gt;;
     owl:previousVersionURI &lt;http://www.openannotation.org/spec/core/20130208/oa.owl&gt;;
     owl:versionInfo &quot;2016-09-30T16:51:18Z&quot; .
  Results:
  @prefix dc: &lt;http://purl.org/dc/terms/&gt; .
  @prefix dc11: &lt;http://purl.org/dc/elements/1.1/&gt; .
  @prefix oa: &lt;http://www.w3.org/ns/oa#&gt; .
  @prefix owl: &lt;http://www.w3.org/2002/07/owl#&gt; .
  @prefix rdf: &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#&gt; .
  @prefix rdfs: &lt;http://www.w3.org/2000/01/rdf-schema#&gt; .
  @prefix skos: &lt;http://www.w3.org/2004/02/skos/core#&gt; .
  @prefix xsd: &lt;http://www.w3.org/2001/XMLSchema#&gt; .

  oa:Annotation a rdfs:Class;
     rdfs:label &quot;Annotation&quot;;
     rdfs:comment &quot;The class for Web Annotations.&quot;;
     rdfs:isDefinedBy oa: .

  oa:Choice a rdfs:Class;
     rdfs:label &quot;Choice&quot;;
     rdfs:comment &quot;A subClass of as:OrderedCollection that conveys to a consuming application that it should select one of the resources in the as:items list to use, rather than all of them. This is typically used to provide a choice of resources to render to the user, based on further supplied properties. If the consuming application cannot determine the user&#39;s preference, then it should use the first in the list.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:as:OrderedCollection .

  oa:Composite a rdfs:Class;
     rdfs:label &quot;Composite&quot;;
     rdfs:comment &quot;A subClass of as:OrderedCollection that conveys to a consuming application that it should use all of the resources in the as:items list, but that order is not important. This class is at-risk.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:as:OrderedCollection .

  oa:CssSelector a rdfs:Class;
     rdfs:label &quot;CssSelector&quot;;
     rdfs:comment &quot;A CssSelector describes a Segment of interest in a representation that conforms to the Document Object Model through the use of the CSS selector specification.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:CssStyle a rdfs:Class;
     rdfs:label &quot;CssStyle&quot;;
     rdfs:comment &quot;A resource which describes styles for resources participating in the Annotation using CSS.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Style .

  oa:DataPositionSelector a rdfs:Class;
     rdfs:label &quot;DataPositionSelector&quot;;
     rdfs:comment &quot;DataPositionSelector describes a range of data by recording the start and end positions of the selection in the stream. Position 0 would be immediately before the first byte, position 1 would be immediately before the second byte, and so on. The start byte is thus included in the list, but the end byte is not.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:Direction a rdfs:Class;
     rdfs:label &quot;Direction&quot;;
     rdfs:comment &quot;A class to encapsulate the different text directions that a textual resource might take. It is not used directly in the Annotation Model, only its three instances.&quot;;
     rdfs:isDefinedBy oa: .

  oa:FragmentSelector a rdfs:Class;
     rdfs:label &quot;FragmentSelector&quot;;
     rdfs:comment &quot;The FragmentSelector class is used to record the segment of a representation using the IRI fragment specification defined by the representation&#39;s media type.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:HttpRequestState a rdfs:Class;
     rdfs:label &quot;HttpRequestState&quot;;
     rdfs:comment &quot;The HttpRequestState class is used to record the HTTP request headers that a client SHOULD use to request the correct representation from the resource.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:State .

  oa:Independents a rdfs:Class;
     rdfs:label &quot;Independents&quot;;
     rdfs:comment &quot;A subClass of as:OrderedCollection that conveys to a consuming application that each of the resources in the as:items list are independently associated with all of the other bodies or targets. This class is at-risk.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:as:OrderedCollection .

  oa:List a rdfs:Class;
     rdfs:label &quot;List&quot;;
     rdfs:comment &quot;A subClass of as:OrderedCollection that conveys to a consuming application that it should use each of the resources in the as:items list, and that their order is important. This class is at-risk.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:as:OrderedCollection .

  oa:Motivation a rdfs:Class;
     rdfs:label &quot;Motivation&quot;;
     rdfs:comment &quot;The Motivation class is used to record the user&#39;s intent or motivation for the creation of the Annotation, or the inclusion of the body or target, that it is associated with.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf skos:Concept .

  oa:RangeSelector a rdfs:Class;
     rdfs:label &quot;RangeSelector&quot;;
     rdfs:comment &quot;A Range Selector can be used to identify the beginning and the end of the selection by using other Selectors. The selection consists of everything from the beginning of the starting selector through to the beginning of the ending selector, but not including it.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:ResourceSelection a rdfs:Class;
     rdfs:label &quot;ResourceSelection&quot;;
     rdfs:comment &quot;Instances of the ResourceSelection class identify part (described by an oa:Selector) of another resource (referenced with oa:hasSource), possibly from a particular representation of a resource (described by an oa:State). Please note that ResourceSelection is not used directly in the Web Annotation model, but is provided as a separate class for further application profiles to use, separate from oa:SpecificResource which has many Annotation specific features.&quot;;
     rdfs:isDefinedBy oa: .

  oa:Selector a rdfs:Class;
     rdfs:label &quot;Selector&quot;;
     rdfs:comment &quot;A resource which describes the segment of interest in a representation of a Source resource, indicated with oa:hasSelector from the Specific Resource. This class is not used directly in the Annotation model, only its subclasses.&quot;;
     rdfs:isDefinedBy oa: .

  oa:SpecificResource a rdfs:Class;
     rdfs:label &quot;SpecificResource&quot;;
     rdfs:comment &quot;Instances of the SpecificResource class identify part of another resource (referenced with oa:hasSource), a particular representation of a resource, a resource with styling hints for renders, or any combination of these, as used within an Annotation.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:ResourceSelection .

  oa:State a rdfs:Class;
     rdfs:label &quot;State&quot;;
     rdfs:comment &quot;A State describes the intended state of a resource as applied to the particular Annotation, and thus provides the information needed to retrieve the correct representation of that resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:Style a rdfs:Class;
     rdfs:label &quot;Style&quot;;
     rdfs:comment &quot;A Style describes the intended styling of a resource as applied to the particular Annotation, and thus provides the information to ensure that rendering is consistent across implementations.&quot;;
     rdfs:isDefinedBy oa: .

  oa:SvgSelector a rdfs:Class;
     rdfs:label &quot;SvgSelector&quot;;
     rdfs:comment &quot;An SvgSelector defines an area through the use of the Scalable Vector Graphics [SVG] standard. This allows the user to select a non-rectangular area of the content, such as a circle or polygon by describing the region using SVG. The SVG may be either embedded within the Annotation or referenced as an External Resource.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:TextPositionSelector a rdfs:Class;
     rdfs:label &quot;TextPositionSelector&quot;;
     rdfs:comment &quot;The TextPositionSelector describes a range of text by recording the start and end positions of the selection in the stream. Position 0 would be immediately before the first character, position 1 would be immediately before the second character, and so on.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:TextQuoteSelector a rdfs:Class;
     rdfs:label &quot;TextQuoteSelector&quot;;
     rdfs:comment &quot;The TextQuoteSelector describes a range of text by copying it, and including some of the text immediately before (a prefix) and after (a suffix) it to distinguish between multiple copies of the same sequence of characters.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:TextualBody a rdfs:Class;
     rdfs:label &quot;TextualBody&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:TimeState a rdfs:Class;
     rdfs:label &quot;TimeState&quot;;
     rdfs:comment &quot;A TimeState records the time at which the resource&#39;s state is appropriate for the Annotation, typically the time that the Annotation was created and/or a link to a persistent copy of the current version.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:State .

  oa:XPathSelector a rdfs:Class;
     rdfs:label &quot;XPathSelector&quot;;
     rdfs:comment &quot;An XPathSelector is used to select elements and content within a resource that supports the Document Object Model via a specified XPath value.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:subClassOf oa:Selector .

  oa:as:Application a rdfs:Class;
     rdfs:label &quot;as:Application&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:as:OrderedCollection a rdfs:Class;
     rdfs:label &quot;as:OrderedCollection&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:as:OrderedCollectionPage a rdfs:Class;
     rdfs:label &quot;as:OrderedCollectionPage&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:Dataset a rdfs:Class;
     rdfs:label &quot;dctypes:Dataset&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:MovingImage a rdfs:Class;
     rdfs:label &quot;dctypes:MovingImage&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:Sound a rdfs:Class;
     rdfs:label &quot;dctypes:Sound&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:StillImage a rdfs:Class;
     rdfs:label &quot;dctypes:StillImage&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:dctypes:Text a rdfs:Class;
     rdfs:label &quot;dctypes:Text&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:foaf:Organization a rdfs:Class;
     rdfs:label &quot;foaf:Organization&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:foaf:Person a rdfs:Class;
     rdfs:label &quot;foaf:Person&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:schema:Audience a rdfs:Class;
     rdfs:label &quot;schema:Audience&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:PreferContainedDescriptions a rdfs:Resource;
     rdfs:label &quot;PreferContainedDescriptions&quot;;
     rdfs:comment &quot;An IRI to signal the client prefers to receive full descriptions of the Annotations from a container, not just their IRIs.&quot;;
     rdfs:isDefinedBy oa: .

  oa:PreferContainedIRIs a rdfs:Resource;
     rdfs:label &quot;PreferContainedIRIs&quot;;
     rdfs:comment &quot;An IRI to signal that the client prefers to receive only the IRIs of the Annotations from a container, not their full descriptions.&quot;;
     rdfs:isDefinedBy oa: .

  oa:annotationService a rdf:Property;
     rdfs:label &quot;annotationService&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:as:first a rdf:Property;
     rdfs:label &quot;as:first&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain oa:as:OrderedCollection;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt; .

  oa:as:generator a rdf:Property;
     rdfs:label &quot;as:generator&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:isDefinedBy oa: .

  oa:as:items a rdf:Property;
     rdfs:label &quot;as:items&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range rdf:List .

  oa:as:last a rdf:Property;
     rdfs:label &quot;as:last&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain oa:as:OrderedCollection;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt; .

  oa:as:next a rdf:Property;
     rdfs:label &quot;as:next&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt; .

  oa:as:partOf a rdf:Property;
     rdfs:label &quot;as:partOf&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollection&gt; .

  oa:as:prev a rdf:Property;
     rdfs:label &quot;as:prev&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt; .

  oa:as:startIndex a rdf:Property;
     rdfs:label &quot;as:startIndex&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollectionPage&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:nonNegativeInteger .

  oa:as:totalItems a rdf:Property;
     rdfs:label &quot;as:totalItems&quot;;
     rdfs:comment &quot;&quot;;
     rdfs:domain &lt;http://www.w3.org/ns/activitystreams#OrderedCollection&gt;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:nonNegativeInteger .

  oa:assessing a oa:Motivation;
     rdfs:label &quot;assessing&quot;;
     rdfs:comment &quot;The motivation for when the user intends to provide an assessment about the Target resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:bodyValue a rdf:Property;
     rdfs:label &quot;bodyValue&quot;;
     rdfs:comment &quot;The object of the predicate is a plain text string to be used as the content of the body of the Annotation. The value MUST be an xsd:string and that data type MUST NOT be expressed in the serialization. Note that language MUST NOT be associated with the value either as a language tag, as that is only available for rdf:langString .&quot;;
     rdfs:domain oa:Annotation;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:string .

  oa:bookmarking a oa:Motivation;
     rdfs:label &quot;bookmarking&quot;;
     rdfs:comment &quot;The motivation for when the user intends to create a bookmark to the Target or part thereof.&quot;;
     rdfs:isDefinedBy oa: .

  oa:cachedSource a rdf:Property;
     rdfs:label &quot;cachedSource&quot;;
     rdfs:comment &quot;A object of the relationship is a copy of the Source resource&#39;s representation, appropriate for the Annotation.&quot;;
     rdfs:domain oa:TimeState;
     rdfs:isDefinedBy oa: .

  oa:canonical a rdf:Property;
     rdfs:label &quot;canonical&quot;;
     rdfs:comment &quot;A object of the relationship is the canonical IRI that can always be used to deduplicate the Annotation, regardless of the current IRI used to access the representation.&quot;;
     rdfs:isDefinedBy oa: .

  oa:classifying a oa:Motivation;
     rdfs:label &quot;classifying&quot;;
     rdfs:comment &quot;The motivation for when the user intends to that classify the Target as something.&quot;;
     rdfs:isDefinedBy oa: .

  oa:commenting a oa:Motivation;
     rdfs:label &quot;commenting&quot;;
     rdfs:comment &quot;The motivation for when the user intends to comment about the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:describing a oa:Motivation;
     rdfs:label &quot;describing&quot;;
     rdfs:comment &quot;The motivation for when the user intends to describe the Target, as opposed to a comment about them.&quot;;
     rdfs:isDefinedBy oa: .

  oa:editing a oa:Motivation;
     rdfs:label &quot;editing&quot;;
     rdfs:comment &quot;The motivation for when the user intends to request a change or edit to the Target resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:end a rdf:Property;
     rdfs:label &quot;end&quot;;
     rdfs:comment &quot;The end property is used to convey the 0-based index of the end position of a range of content.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:nonNegativeInteger .

  oa:exact a rdf:Property;
     rdfs:label &quot;exact&quot;;
     rdfs:comment &quot;The object of the predicate is a copy of the text which is being selected, after normalization.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:string .

  oa:hasBody a rdf:Property;
     rdfs:label &quot;hasBody&quot;;
     rdfs:comment &quot;The object of the relationship is a resource that is a body of the Annotation.&quot;;
     rdfs:domain oa:Annotation;
     rdfs:isDefinedBy oa: .

  oa:hasEndSelector a rdf:Property;
     rdfs:label &quot;hasEndSelector&quot;;
     rdfs:comment &quot;The relationship between a RangeSelector and the Selector that describes the end position of the range.&quot;;
     rdfs:domain oa:RangeSelector;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Selector .

  oa:hasPurpose a rdf:Property;
     rdfs:label &quot;hasPurpose&quot;;
     rdfs:comment &quot;The purpose served by the resource in the Annotation.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Motivation .

  oa:hasScope a rdf:Property;
     rdfs:label &quot;hasScope&quot;;
     rdfs:comment &quot;The scope or context in which the resource is used within the Annotation.&quot;;
     rdfs:domain oa:SpecificResource;
     rdfs:isDefinedBy oa: .

  oa:hasSelector a rdf:Property;
     rdfs:label &quot;hasSelector&quot;;
     rdfs:comment &quot;The object of the relationship is a Selector that describes the segment or region of interest within the source resource. Please note that the domain ( oa:ResourceSelection ) is not used directly in the Web Annotation model.&quot;;
     rdfs:domain oa:ResourceSelection;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Selector .

  oa:hasSource a rdf:Property;
     rdfs:label &quot;hasSource&quot;;
     rdfs:comment &quot;The resource that the ResourceSelection, or its subclass SpecificResource, is refined from, or more specific than. Please note that the domain ( oa:ResourceSelection ) is not used directly in the Web Annotation model.&quot;;
     rdfs:domain oa:ResourceSelection;
     rdfs:isDefinedBy oa: .

  oa:hasStartSelector a rdf:Property;
     rdfs:label &quot;hasStartSelector&quot;;
     rdfs:comment &quot;The relationship between a RangeSelector and the Selector that describes the start position of the range.&quot;;
     rdfs:domain oa:RangeSelector;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Selector .

  oa:hasState a rdf:Property;
     rdfs:label &quot;hasState&quot;;
     rdfs:comment &quot;The relationship between the ResourceSelection, or its subclass SpecificResource, and a State resource. Please note that the domain ( oa:ResourceSelection ) is not used directly in the Web Annotation model.&quot;;
     rdfs:domain oa:ResourceSelection;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:State .

  oa:hasTarget a rdf:Property;
     rdfs:label &quot;hasTarget&quot;;
     rdfs:comment &quot;The relationship between an Annotation and its Target.&quot;;
     rdfs:domain oa:Annotation;
     rdfs:isDefinedBy oa: .

  oa:highlighting a oa:Motivation;
     rdfs:label &quot;highlighting&quot;;
     rdfs:comment &quot;The motivation for when the user intends to highlight the Target resource or segment of it.&quot;;
     rdfs:isDefinedBy oa: .

  oa:identifying a oa:Motivation;
     rdfs:label &quot;identifying&quot;;
     rdfs:comment &quot;The motivation for when the user intends to assign an identity to the Target or identify what is being depicted or described in the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:linking a oa:Motivation;
     rdfs:label &quot;linking&quot;;
     rdfs:comment &quot;The motivation for when the user intends to link to a resource related to the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:ltrDirection a oa:Direction;
     rdfs:label &quot;ltrDirection&quot;;
     rdfs:comment &quot;The direction of text that is read from left to right.&quot;;
     rdfs:isDefinedBy oa: .

  oa:moderating a oa:Motivation;
     rdfs:label &quot;moderating&quot;;
     rdfs:comment &quot;The motivation for when the user intends to assign some value or quality to the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:motivatedBy a rdf:Property;
     rdfs:label &quot;motivatedBy&quot;;
     rdfs:comment &quot;The relationship between an Annotation and a Motivation that describes the reason for the Annotation&#39;s creation.&quot;;
     rdfs:domain oa:Annotation;
     rdfs:isDefinedBy oa:;
     rdfs:range oa:Motivation .

  oa:prefix a rdf:Property;
     rdfs:label &quot;prefix&quot;;
     rdfs:comment &quot;The object of the property is a snippet of content that occurs immediately before the content which is being selected by the Selector.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:string .

  oa:processingLanguage a rdf:Property;
     rdfs:label &quot;processingLanguage&quot;;
     rdfs:comment &quot;The object of the property is the language that should be used for textual processing algorithms when dealing with the content of the resource, including hyphenation, line breaking, which font to use for rendering and so forth. The value must follow the recommendations of BCP47.&quot;;
     rdfs:isDefinedBy oa:;
     rdfs:range xsd:string .

  oa:questioning a oa:Motivation;
     rdfs:label &quot;questioning&quot;;
     rdfs:comment &quot;The motivation for when the user intends to ask a question about the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa:refinedBy a rdf:Property;
     rdfs:label &quot;refinedBy&quot;;
     rdfs:comment &quot;The relationship between a Selector and another Selector or a State and a Selector or State that should be applied to the results of the first to refine the processing of the source resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:replying a oa:Motivation;
     rdfs:label &quot;replying&quot;;
     rdfs:comment &quot;The motivation for when the user intends to reply to a previous statement, either an Annotation or another resource.&quot;;
     rdfs:isDefinedBy oa: .

  oa:rtlDirection a oa:Direction;
     rdfs:label &quot;rtlDirection&quot;;
     rdfs:comment &quot;The direction of text that is read from right to left.&quot;;
     rdfs:isDefinedBy oa: .

  oa:tagging a oa:Motivation;
     rdfs:label &quot;tagging&quot;;
     rdfs:comment &quot;The motivation for when the user intends to associate a tag with the Target.&quot;;
     rdfs:isDefinedBy oa: .

  oa: a owl:Ontology;
     dc11:title &quot;Web Annotation Ontology&quot;;
     dc:creator &quot;Robert Sanderson&quot;,
       &quot;Paolo Ciccarese&quot;,
       &quot;Benjamin Young&quot;;
     dc:modified &quot;2016-09-30T16:51:18Z&quot;;
     rdfs:comment &quot;The Web Annotation ontology defines the terms of the Web Annotation vocabulary. Any changes to this document MUST be from a Working Group in the W3C that has established expertise in the area.&quot;;
     rdfs:seeAlso &lt;http://www.w3.org/TR/annotation-vocab/&gt;;
     owl:previousVersionURI &lt;http://www.openannotation.org/spec/core/20130208/oa.owl&gt;;
     owl:versionInfo &quot;2016-09-30T16:51:18Z&quot; .

  Debug:</pre></div>
        <div class="backtrace"><pre>./annotation-vocab_spec.rb:83:in `block (4 levels) in &lt;top (required)&gt;&#39;</pre></div>
    <pre class="ruby"><code><span class="linenum">81</span>        # XXX Normalize whitespace in literals to ease comparision
<span class="linenum">82</span>        fg.each_object {|o| o.squish! if o.literal?}
<span class="offending"><span class="linenum">83</span>        expect(fg).to be_equivalent_graph(vocab_graph)</span>
<span class="linenum">84</span>      end
<span class="linenum">85</span>    end
<span class="linenum">86</span><span class="comment"># Install the coderay gem to get syntax highlighting</span></code></pre>
      </div>
    </dd>
    <script type="text/javascript">moveProgressBar('45.3');</script>
    <dd class="example passed"><span class="passed_spec_name">ttl</span><span class='duration'>0.48723s</span></dd>
  </dl>
</div>
<div id="div_group_6" class="example_group passed">
  <dl style="margin-left: 15px;">
  <dt id="example_group_6" class="passed">The ontology is internally consistent with respect to domains, ranges, inverses, and any other ontology features specified.</dt>
    <script type="text/javascript">makeRed('div_group_6');</script>
    <script type="text/javascript">makeRed('example_group_6');</script>
    <script type="text/javascript">moveProgressBar('45.6');</script>
    <dd class="example failed">
      <span class="failed_spec_name">lints cleanly</span>
      <span class="duration">2.09350s</span>
      <div class="failure" id="failure_3">
        <div class="message"><pre>Failure/Error: expect(entailed_graph.lint).to be_empty
  expected `{:property=&gt;{&quot;dc:creator&quot;=&gt;[&quot;Object \&quot;Robert Sanderson\&quot; not compatible with range (dc:Agent)&quot;, &quot;Obje...mpatible with range (dc:Agent)&quot;, &quot;Object \&quot;Benjamin Young\&quot; not compatible with range (dc:Agent)&quot;]}}.empty?` to return true, got false</pre></div>
        <div class="backtrace"><pre>./annotation-vocab_spec.rb:91:in `block (3 levels) in &lt;top (required)&gt;&#39;</pre></div>
    <pre class="ruby"><code><span class="linenum">89</span>    it "lints cleanly" do
<span class="linenum">90</span>      entailed_graph = vocab_graph.dup.entail!
<span class="offending"><span class="linenum">91</span>      expect(entailed_graph.lint).to be_empty</span>
<span class="linenum">92</span>    end
<span class="linenum">93</span>
<span class="linenum">94</span><span class="comment"># Install the coderay gem to get syntax highlighting</span></code></pre>
      </div>
    </dd>
  </dl>
</div>
<div id="div_group_7" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_7" class="passed">oa:Annotation</dt>
    <script type="text/javascript">moveProgressBar('46.0');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00013s</span></dd>
    <script type="text/javascript">moveProgressBar('46.3');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00057s</span></dd>
  </dl>
</div>
<div id="div_group_8" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_8" class="passed">oa:Choice</dt>
    <script type="text/javascript">moveProgressBar('46.6');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.01390s</span></dd>
    <script type="text/javascript">moveProgressBar('47.0');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00054s</span></dd>
  </dl>
</div>
<div id="div_group_9" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_9" class="passed">oa:Composite</dt>
    <script type="text/javascript">moveProgressBar('47.3');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00201s</span></dd>
    <script type="text/javascript">moveProgressBar('47.6');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00046s</span></dd>
  </dl>
</div>
<div id="div_group_10" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_10" class="passed">oa:CssSelector</dt>
    <script type="text/javascript">moveProgressBar('48.0');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00087s</span></dd>
    <script type="text/javascript">moveProgressBar('48.3');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00049s</span></dd>
  </dl>
</div>
<div id="div_group_11" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_11" class="passed">oa:CssStyle</dt>
    <script type="text/javascript">moveProgressBar('48.6');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('49.0');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00049s</span></dd>
  </dl>
</div>
<div id="div_group_12" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_12" class="passed">oa:DataPositionSelector</dt>
    <script type="text/javascript">moveProgressBar('49.3');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('49.6');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00059s</span></dd>
  </dl>
</div>
<div id="div_group_13" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_13" class="passed">oa:Direction</dt>
    <script type="text/javascript">moveProgressBar('50.0');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('50.3');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00049s</span></dd>
  </dl>
</div>
<div id="div_group_14" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_14" class="passed">oa:FragmentSelector</dt>
    <script type="text/javascript">moveProgressBar('50.6');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00089s</span></dd>
    <script type="text/javascript">moveProgressBar('50.9');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00047s</span></dd>
  </dl>
</div>
<div id="div_group_15" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_15" class="passed">oa:HttpRequestState</dt>
    <script type="text/javascript">moveProgressBar('51.3');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00090s</span></dd>
    <script type="text/javascript">moveProgressBar('51.6');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00047s</span></dd>
  </dl>
</div>
<div id="div_group_16" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_16" class="passed">oa:Independents</dt>
    <script type="text/javascript">moveProgressBar('51.9');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00204s</span></dd>
    <script type="text/javascript">moveProgressBar('52.3');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00046s</span></dd>
  </dl>
</div>
<div id="div_group_17" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_17" class="passed">oa:List</dt>
    <script type="text/javascript">moveProgressBar('52.6');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00220s</span></dd>
    <script type="text/javascript">moveProgressBar('52.9');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00052s</span></dd>
  </dl>
</div>
<div id="div_group_18" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_18" class="passed">oa:Motivation</dt>
    <script type="text/javascript">moveProgressBar('53.3');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00120s</span></dd>
    <script type="text/javascript">moveProgressBar('53.6');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00053s</span></dd>
  </dl>
</div>
<div id="div_group_19" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_19" class="passed">oa:RangeSelector</dt>
    <script type="text/javascript">moveProgressBar('53.9');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00093s</span></dd>
    <script type="text/javascript">moveProgressBar('54.3');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00048s</span></dd>
  </dl>
</div>
<div id="div_group_20" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_20" class="passed">oa:ResourceSelection</dt>
    <script type="text/javascript">moveProgressBar('54.6');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('54.9');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00049s</span></dd>
  </dl>
</div>
<div id="div_group_21" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_21" class="passed">oa:Selector</dt>
    <script type="text/javascript">moveProgressBar('55.2');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('55.6');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00049s</span></dd>
  </dl>
</div>
<div id="div_group_22" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_22" class="passed">oa:SpecificResource</dt>
    <script type="text/javascript">moveProgressBar('55.9');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00089s</span></dd>
    <script type="text/javascript">moveProgressBar('56.2');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00050s</span></dd>
  </dl>
</div>
<div id="div_group_23" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_23" class="passed">oa:State</dt>
    <script type="text/javascript">moveProgressBar('56.6');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('56.9');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00048s</span></dd>
  </dl>
</div>
<div id="div_group_24" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_24" class="passed">oa:Style</dt>
    <script type="text/javascript">moveProgressBar('57.2');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('57.6');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00048s</span></dd>
  </dl>
</div>
<div id="div_group_25" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_25" class="passed">oa:SvgSelector</dt>
    <script type="text/javascript">moveProgressBar('57.9');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00086s</span></dd>
    <script type="text/javascript">moveProgressBar('58.2');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00050s</span></dd>
  </dl>
</div>
<div id="div_group_26" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_26" class="passed">oa:TextPositionSelector</dt>
    <script type="text/javascript">moveProgressBar('58.6');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00091s</span></dd>
    <script type="text/javascript">moveProgressBar('58.9');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00049s</span></dd>
  </dl>
</div>
<div id="div_group_27" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_27" class="passed">oa:TextQuoteSelector</dt>
    <script type="text/javascript">moveProgressBar('59.2');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00089s</span></dd>
    <script type="text/javascript">moveProgressBar('59.6');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00048s</span></dd>
  </dl>
</div>
<div id="div_group_28" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_28" class="passed">oa:TextualBody</dt>
    <script type="text/javascript">moveProgressBar('59.9');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('60.2');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00049s</span></dd>
  </dl>
</div>
<div id="div_group_29" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_29" class="passed">oa:TimeState</dt>
    <script type="text/javascript">moveProgressBar('60.5');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00089s</span></dd>
    <script type="text/javascript">moveProgressBar('60.9');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00049s</span></dd>
  </dl>
</div>
<div id="div_group_30" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_30" class="passed">oa:XPathSelector</dt>
    <script type="text/javascript">moveProgressBar('61.2');</script>
    <dd class="example passed"><span class="passed_spec_name">subClassOf</span><span class='duration'>0.00089s</span></dd>
    <script type="text/javascript">moveProgressBar('61.5');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentClass</span><span class='duration'>0.00048s</span></dd>
  </dl>
</div>
<div id="div_group_31" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_31" class="passed">oa:annotationService</dt>
    <script type="text/javascript">moveProgressBar('61.9');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00011s</span></dd>
    <script type="text/javascript">moveProgressBar('62.2');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('62.5');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('62.9');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00013s</span></dd>
  </dl>
</div>
<div id="div_group_32" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_32" class="passed">oa:bodyValue</dt>
    <script type="text/javascript">moveProgressBar('63.2');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('63.5');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('63.9');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00083s</span></dd>
    <script type="text/javascript">moveProgressBar('64.2');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00015s</span></dd>
  </dl>
</div>
<div id="div_group_33" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_33" class="passed">oa:cachedSource</dt>
    <script type="text/javascript">moveProgressBar('64.5');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('64.9');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00089s</span></dd>
    <script type="text/javascript">moveProgressBar('65.2');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('65.5');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00012s</span></dd>
  </dl>
</div>
<div id="div_group_34" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_34" class="passed">oa:canonical</dt>
    <script type="text/javascript">moveProgressBar('65.8');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('66.2');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00011s</span></dd>
    <script type="text/javascript">moveProgressBar('66.5');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('66.8');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00012s</span></dd>
  </dl>
</div>
<div id="div_group_35" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_35" class="passed">oa:end</dt>
    <script type="text/javascript">moveProgressBar('67.2');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('67.5');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('67.8');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00085s</span></dd>
    <script type="text/javascript">moveProgressBar('68.2');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00012s</span></dd>
  </dl>
</div>
<div id="div_group_36" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_36" class="passed">oa:exact</dt>
    <script type="text/javascript">moveProgressBar('68.5');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('68.8');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('69.2');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00092s</span></dd>
    <script type="text/javascript">moveProgressBar('69.5');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00012s</span></dd>
  </dl>
</div>
<div id="div_group_37" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_37" class="passed">oa:hasBody</dt>
    <script type="text/javascript">moveProgressBar('69.8');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('70.1');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00090s</span></dd>
    <script type="text/javascript">moveProgressBar('70.5');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('70.8');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00013s</span></dd>
  </dl>
</div>
<div id="div_group_38" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_38" class="passed">oa:hasEndSelector</dt>
    <script type="text/javascript">moveProgressBar('71.1');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00011s</span></dd>
    <script type="text/javascript">moveProgressBar('71.5');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00089s</span></dd>
    <script type="text/javascript">moveProgressBar('71.8');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00089s</span></dd>
    <script type="text/javascript">moveProgressBar('72.1');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00013s</span></dd>
  </dl>
</div>
<div id="div_group_39" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_39" class="passed">oa:hasPurpose</dt>
    <script type="text/javascript">moveProgressBar('72.5');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('72.8');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('73.1');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('73.5');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00013s</span></dd>
  </dl>
</div>
<div id="div_group_40" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_40" class="passed">oa:hasScope</dt>
    <script type="text/javascript">moveProgressBar('73.8');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('74.1');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00087s</span></dd>
    <script type="text/javascript">moveProgressBar('74.5');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('74.8');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00011s</span></dd>
  </dl>
</div>
<div id="div_group_41" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_41" class="passed">oa:hasSelector</dt>
    <script type="text/javascript">moveProgressBar('75.1');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('75.4');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('75.8');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('76.1');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00013s</span></dd>
  </dl>
</div>
<div id="div_group_42" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_42" class="passed">oa:hasSource</dt>
    <script type="text/javascript">moveProgressBar('76.4');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('76.8');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('77.1');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('77.4');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00012s</span></dd>
  </dl>
</div>
<div id="div_group_43" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_43" class="passed">oa:hasStartSelector</dt>
    <script type="text/javascript">moveProgressBar('77.8');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('78.1');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('78.4');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('78.8');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00013s</span></dd>
  </dl>
</div>
<div id="div_group_44" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_44" class="passed">oa:hasState</dt>
    <script type="text/javascript">moveProgressBar('79.1');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('79.4');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('79.8');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00087s</span></dd>
    <script type="text/javascript">moveProgressBar('80.1');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00012s</span></dd>
  </dl>
</div>
<div id="div_group_45" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_45" class="passed">oa:hasTarget</dt>
    <script type="text/javascript">moveProgressBar('80.4');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('80.7');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('81.1');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('81.4');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00013s</span></dd>
  </dl>
</div>
<div id="div_group_46" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_46" class="passed">oa:motivatedBy</dt>
    <script type="text/javascript">moveProgressBar('81.7');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00011s</span></dd>
    <script type="text/javascript">moveProgressBar('82.1');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00089s</span></dd>
    <script type="text/javascript">moveProgressBar('82.4');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00087s</span></dd>
    <script type="text/javascript">moveProgressBar('82.7');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00013s</span></dd>
  </dl>
</div>
<div id="div_group_47" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_47" class="passed">oa:prefix</dt>
    <script type="text/javascript">moveProgressBar('83.1');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('83.4');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('83.7');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00082s</span></dd>
    <script type="text/javascript">moveProgressBar('84.1');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00014s</span></dd>
  </dl>
</div>
<div id="div_group_48" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_48" class="passed">oa:processingLanguage</dt>
    <script type="text/javascript">moveProgressBar('84.4');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('84.7');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00012s</span></dd>
    <script type="text/javascript">moveProgressBar('85.0');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00085s</span></dd>
    <script type="text/javascript">moveProgressBar('85.4');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00012s</span></dd>
  </dl>
</div>
<div id="div_group_49" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_49" class="passed">oa:refinedBy</dt>
    <script type="text/javascript">moveProgressBar('85.7');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('86.0');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('86.4');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('86.7');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00011s</span></dd>
  </dl>
</div>
<div id="div_group_50" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_50" class="passed">oa:renderedVia</dt>
    <script type="text/javascript">moveProgressBar('87.0');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00011s</span></dd>
    <script type="text/javascript">moveProgressBar('87.4');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('87.7');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00086s</span></dd>
    <script type="text/javascript">moveProgressBar('88.0');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00013s</span></dd>
  </dl>
</div>
<div id="div_group_51" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_51" class="passed">oa:sourceDate</dt>
    <script type="text/javascript">moveProgressBar('88.4');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('88.7');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00087s</span></dd>
    <script type="text/javascript">moveProgressBar('89.0');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00081s</span></dd>
    <script type="text/javascript">moveProgressBar('89.4');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00011s</span></dd>
  </dl>
</div>
<div id="div_group_52" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_52" class="passed">oa:sourceDateEnd</dt>
    <script type="text/javascript">moveProgressBar('89.7');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('90.0');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00087s</span></dd>
    <script type="text/javascript">moveProgressBar('90.3');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00082s</span></dd>
    <script type="text/javascript">moveProgressBar('90.7');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00011s</span></dd>
  </dl>
</div>
<div id="div_group_53" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_53" class="passed">oa:sourceDateStart</dt>
    <script type="text/javascript">moveProgressBar('91.0');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('91.3');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00089s</span></dd>
    <script type="text/javascript">moveProgressBar('91.7');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00082s</span></dd>
    <script type="text/javascript">moveProgressBar('92.0');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00012s</span></dd>
  </dl>
</div>
<div id="div_group_54" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_54" class="passed">oa:start</dt>
    <script type="text/javascript">moveProgressBar('92.3');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('92.7');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('93.0');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00083s</span></dd>
    <script type="text/javascript">moveProgressBar('93.3');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00013s</span></dd>
  </dl>
</div>
<div id="div_group_55" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_55" class="passed">oa:styleClass</dt>
    <script type="text/javascript">moveProgressBar('93.7');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('94.0');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('94.3');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00082s</span></dd>
    <script type="text/javascript">moveProgressBar('94.7');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00012s</span></dd>
  </dl>
</div>
<div id="div_group_56" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_56" class="passed">oa:styledBy</dt>
    <script type="text/javascript">moveProgressBar('95.0');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('95.3');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00092s</span></dd>
    <script type="text/javascript">moveProgressBar('95.6');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('96.0');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00013s</span></dd>
  </dl>
</div>
<div id="div_group_57" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_57" class="passed">oa:suffix</dt>
    <script type="text/javascript">moveProgressBar('96.3');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('96.6');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00010s</span></dd>
    <script type="text/javascript">moveProgressBar('97.0');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00081s</span></dd>
    <script type="text/javascript">moveProgressBar('97.3');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00012s</span></dd>
  </dl>
</div>
<div id="div_group_58" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_58" class="passed">oa:textDirection</dt>
    <script type="text/javascript">moveProgressBar('97.6');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('98.0');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('98.3');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00088s</span></dd>
    <script type="text/javascript">moveProgressBar('98.6');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00012s</span></dd>
  </dl>
</div>
<div id="div_group_59" class="example_group passed">
  <dl style="margin-left: 30px;">
  <dt id="example_group_59" class="passed">oa:via</dt>
    <script type="text/javascript">moveProgressBar('99.0');</script>
    <dd class="example passed"><span class="passed_spec_name">subPropertyOf</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('99.3');</script>
    <dd class="example passed"><span class="passed_spec_name">domain</span><span class='duration'>0.00009s</span></dd>
    <script type="text/javascript">moveProgressBar('99.6');</script>
    <dd class="example passed"><span class="passed_spec_name">range</span><span class='duration'>0.00011s</span></dd>
    <script type="text/javascript">moveProgressBar('100.0');</script>
    <dd class="example passed"><span class="passed_spec_name">equivalentProperty</span><span class='duration'>0.00011s</span></dd>
  </dl>
</div>
<script type="text/javascript">document.getElementById('duration').innerHTML = "Finished in <strong>52.01192 seconds</strong>";</script>
<script type="text/javascript">document.getElementById('totals').innerHTML = "302 examples, 3 failures, 2 pending";</script>
</div>
</div>
</body>
</html>
