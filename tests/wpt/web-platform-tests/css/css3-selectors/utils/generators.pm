package utils::generators;
use strict;
use utils::helpers;
1;

sub extensions {
    my($type) = @_;
    return 'html' if $type eq 'tng';
    return $type =~ m/^(?:|.*[^x])html/o ? 'html' : 'xml';
}

##############################################################################
# Index Generators                                                           #
##############################################################################

sub generateTopIndex { # points to test type indexes
    my($testDatabase) = @_;
    open(FILE, '>dist/index.html') or die "failed to open output file: $!";
    local $" = ', ';
    # XXX Hardcoded to say Selectors
    print FILE '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN">
<html>
 <head>
  <title>CSS3 Selectors Test Suite Index</title>
 </head>
 <body>
  <h1><a href="http://www.w3.org/"><img src="http://www.w3.org/Icons/WWW/w3c_home" alt="W3C" width="72" height="48"></a> CSS3 Selectors Test Suite Index</h1>
  <p>The tests are available in several variants.</p>
  <ul>';
    foreach my $type (split ' ', $utils::helpers::types{'DESTINATION_TYPES'}) {
        print FILE "\n   <li><a href=\"".&utils::helpers::escape("$type/index.html").'">'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$type}).'</a></li>';
    }
    # XXX Most of the following should be stored in a config file or something
    print FILE '
  </ul>
  <p>A list of recent changes may be found in the <a
  href="CHANGES">CHANGES</a> file.</p>
  <h2>The Role Of This Test Suite</h2>

  <p>The role of this test is primarily to help implementors develop
  more comprehensive tests and to help authors gauge the level of
  support for the basics of the Selectors specification.</p>

  <p>It is also a key part of the Selectors specification exit
  criteria.  For this specification to exit the CR stage, the following
  conditions shall be met:</p>
  <ol>
  <li><p> There must be at least two interoperable implementations for
     every feature in the Selectors Module.</p>
     <p>For the purposes of this criterion, we define the following terms:</p>
      <dl><dt>feature</dt><dd><p>a section or subsection in the Selectors Module.</p></dd>
      <dt>interoperable</dt><dd><p>passing the respective test case(s) in the
      Selectors Module test suite, or, if the implementation is not a
      web browser, an equivalent test. Every relevant test in the test
      suite should have an equivalent test created if such a UA is to
      be used to claim interoperability. In addition if such a UA is
      to be used to claim interoperability, then there must one or
      more additional UAs which can also pass those equivalent tests
      in the same way for the purpose of interoperability. The
      equivalent tests must be made publically available for the
      purposes of peer review.</p></dd>
      <dt>implementation</dt><dd><p>a user agent which:</p>
     <ol>
        <li>implements the feature.</li>
        <li>is available (i.e. publicly downloadable or available
           through some other public point of sale mechanism). This is
           the "show me" requirement.</li>
        <li>is shipping (i.e. development, private or unofficial
           versions are insufficient).</li>
        <li>is not experimental (i.e. is intended for a wide audience
           and could be used on a daily basis.)</li></ol></dd></dl>
  <li><p>A minimum of six months of the CR period must have elapsed.
     This is to ensure that enough time is given for any remaining
     major errors to be caught.</p>
   </li></ol>

  <h2>Contributors</h2>
  <p>The authors of the test suite are ';
    my %authors;
    foreach my $test (values(%$testDatabase)) {
        foreach my $author (@{$test->{'author'}}) {
            $authors{$author}++;
        }
    }
    my @authors = sort(keys(%authors));
    foreach my $index (0..$#authors) {
        if ($index > 0) {
            if ($index eq $#authors) {
                print FILE ' and ';
            } else {
                print FILE ', ';
            }
        }
        print FILE $authors[$index];
    }
    print FILE '.</p>
  <p class=copyright><a href="http://www.w3.org/Consortium/Legal/ipr-notice-20000612#Copyright">Copyright</a> &copy;2001 <a href="http://www.w3.org/"><abbr title="World Wide Web Consortium">W3C</abbr></a><sup>&reg;</sup> (<a href="http://www.lcs.mit.edu/"><abbr title="Massachusetts Institute of Technology">MIT</abbr></a>, <a href="http://www.inria.fr/"><abbr lang=fr title="Institut National de Recherche en Informatique et Automatique">INRIA</abbr></a>, <a href="http://www.keio.ac.jp/">Keio</a>), All Rights Reserved. W3C <a href="http://www.w3.org/Consortium/Legal/ipr-notice-20000612#Legal_Disclaimer">liability</a>, <a href="http://www.w3.org/Consortium/Legal/ipr-notice-20000612#W3C_Trademarks">trademark</a>, <a href="http://www.w3.org/Consortium/Legal/copyright-documents-19990405">document use</a> and <a href="http://www.w3.org/Consortium/Legal/copyright-software-19980720">software licensing</a> rules apply.</p>
 </body>
</html>';
    close(FILE);
}

sub generateSubIndex { # points to mini test index and all indexes for this test type
    my($destinationType, $testList, $testDatabase) = @_;
    open(FILE, ">dist/$destinationType/index.html") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN">
<html>
 <head>
  <title>'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' Test Index</title>
  <link rel="top" href="../index.html">
 </head>
 <body>
  <h1>'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' Test Index</h1>
  <p>The '.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' tests are available in several variants.</p>
  <h2>Tests With Navigation Aids</h2>
  <p>Each category of test is available using several different harnesses. The name of the harness describes how the test markup is contained within it, for example the Xlink embed case uses an XLink with the show axis set to embed.</p>
  <ul>';
    foreach my $category (split ' ', $utils::helpers::types{'TEST_TYPES'}) {
        print FILE "\n   <li><a href=\"".&utils::helpers::escape($category).'/index.html" title="'.&utils::helpers::escape($utils::helpers::TestTypeDescriptions{$category}).'">'.&utils::helpers::escape($utils::helpers::TestTypeShortTitles{$category}).'</a>: ';
        print FILE '<a href="'.&utils::helpers::escape("$category/flat/index.html").'">Self Contained</a>';
        foreach my $type (split ' ', $utils::helpers::types{'SHELL_TYPES'}) {
            print FILE ', <a href="'.&utils::helpers::escape("$category/$type/index.html").'" title="'.&utils::helpers::escape($utils::helpers::ShellTypeDescriptions{$type}).'">'.&utils::helpers::escape($utils::helpers::ShellTypeTitles{$type}).'</a>';
        }
        print FILE '</li>';
    }
    print FILE '
  </ul>
  <h2>Unadorned Tests</h2>
  <ul>';
    foreach my $test (@$testList) {
        print FILE "\n   <li><a href=\"".&utils::helpers::escape("tests/$test.".&extensions($destinationType)).'">'.&utils::helpers::escape($testDatabase->{$test}->{'def'})."</a> (#".&utils::helpers::escape($testDatabase->{$test}->{'number'}).")</li>";
    }
    print FILE '
  </ul>
  <p>See also: <a href="../index.html">Index</a>';
    foreach my $type (split ' ', $utils::helpers::types{'DESTINATION_TYPES'}) {
        if ($type ne $destinationType) {
            print FILE ', <a href="'.&utils::helpers::escape("../$type/index.html").'">'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$type}).'</a>';
        }
    }
    print FILE '</p>
 </body>
</html>';
    close(FILE);
}

sub generateMiniTestIndex { # points to all mini tests
    my($destinationType, $testList, $testDatabase) = @_;
    open(FILE, ">dist/$destinationType/tests/index.html") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN">
<html>
 <head>
  <title>'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' Unadorned Test Index</title>
  <link rel="up" href="../index.html">
  <link rel="top" href="../../index.html">
 </head>
 <body>
  <h1>'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' Unadorned Test Index</h1>
  <ul>';
    foreach my $test (@$testList) {
        print FILE "\n   <li><a href=\"".&utils::helpers::escape("$test.".&extensions($destinationType)).'">'.&utils::helpers::escape($testDatabase->{$test}->{'def'})."</a> (#".&utils::helpers::escape($testDatabase->{$test}->{'number'}).")</li>";
    }
    print FILE '
  </ul>
  <p>See also: <a href="../../index.html">Index</a>, <a href="../index.html">'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' Index</a>';
    foreach my $type (split ' ', $utils::helpers::types{'DESTINATION_TYPES'}) {
        if ($type ne $destinationType) {
            print FILE ', <a href="'.&utils::helpers::escape("../../$type/index.html").'">'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$type}).'</a>';
        }
    }
    print FILE '</p>
 </body>
</html>';
    close(FILE);
}

sub generateTestTypeIndex { # points to flat test index and each shell index
    my($destinationType, $testType, $testList, $testDatabase) = @_;
    open(FILE, ">dist/$destinationType/$testType/index.html") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN">
<html>
 <head>
  <title>'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' '.&utils::helpers::escape($utils::helpers::TestTypeShortTitles{$testType}).' Index</title>
  <link rel="up" href="../index.html">
  <link rel="top" href="../../index.html">
 </head>
 <body>
  <h1>'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' '.&utils::helpers::escape($utils::helpers::TestTypeShortTitles{$testType}).' Index</h1>
  <p>'.&utils::helpers::escape($utils::helpers::TestTypeDescriptions{$testType}).'</p>
  <p>Please select the type of test harness you wish to use to embed the tests inside the navigation aids:</p>
  <dl>
   <dt><a href="'.&utils::helpers::escape("flat/index.html").'">Self Contained</a></dt>
   <dd>Tests consist of an '.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' page describing the test and containing, inline, the test content.</dd>';
    foreach my $type (split ' ', $utils::helpers::types{'SHELL_TYPES'}) {
        print FILE "\n   <dt><a href=\"".&utils::helpers::escape("$type/index.html").'">'.&utils::helpers::escape($utils::helpers::ShellTypeTitles{$type}).'</a></dt>
   <dd>'.&utils::helpers::escape($utils::helpers::ShellTypeDescriptions{$type}).'</dd>';
    }
    print FILE '
  </dl>
  <p>See also: <a href="../../index.html">Index</a>, <a href="../index.html">'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' Index</a>';
    foreach my $type (split ' ', $utils::helpers::types{'DESTINATION_TYPES'}) {
        if ($type ne $destinationType) {
            print FILE ', <a href="'.&utils::helpers::escape("../../../$type/index.html").'">'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$type}).'</a>';
        }
    }
    print FILE '</p>
 </body>
</html>';
    close(FILE);
}

sub generateFlatTestIndex { # points to flat tests
    my($destinationType, $testType, $testList, $testDatabase) = @_;
    open(FILE, ">dist/$destinationType/$testType/flat/index.html") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN">
<html>
 <head>
  <title>'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' Self Contained '.&utils::helpers::escape($utils::helpers::TestTypeShortTitles{$testType}).' Index</title>
  <link rel="up" href="../index.html">
  <link rel="top" href="../../../index.html">
 </head>
 <body>
  <h1>'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' Self Contained '.&utils::helpers::escape($utils::helpers::TestTypeShortTitles{$testType}).' Index</h1>
  <ul>';
    foreach my $test (@$testList) {
        print FILE "\n   <li><a href=\"".&utils::helpers::escape("$test.".&extensions($destinationType)).'">'.&utils::helpers::escape($testDatabase->{$test}->{'def'})."</a> (#".&utils::helpers::escape($testDatabase->{$test}->{'number'}).")</li>";
    }
    print FILE '
  </ul>
  <p>See also: <a href="../../../index.html">Index</a>, <a href="../../index.html">'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' Index</a>';
    foreach my $type (split ' ', $utils::helpers::types{'DESTINATION_TYPES'}) {
        if ($type ne $destinationType) {
            print FILE ', <a href="'.&utils::helpers::escape("../../../$type/index.html").'">'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$type}).'</a>';
        }
    }
    print FILE '</p>
 </body>
</html>';
    close(FILE);
}

sub generateShellTestIndex { # points to shell tests
    my($destinationType, $testType, $shellType, $testList, $testDatabase) = @_;
    open(FILE, ">dist/$destinationType/$testType/$shellType/index.html") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN">
<html>
 <head>
  <title>'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' '.&utils::helpers::escape($utils::helpers::ShellTypeTitles{$shellType}).' '.&utils::helpers::escape($utils::helpers::TestTypeShortTitles{$testType}).' Index</title>
  <link rel="up" href="../index.html">
  <link rel="top" href="../../../index.html">
 </head>
 <body>
  <h1>'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' '.&utils::helpers::escape($utils::helpers::ShellTypeTitles{$shellType}).' '.&utils::helpers::escape($utils::helpers::TestTypeShortTitles{$testType}).' Index</h1>
  <p>'.&utils::helpers::escape($utils::helpers::ShellTypeDescriptions{$shellType}).'</p>
  <ul>';
    foreach my $test (@$testList) {
        print FILE "\n   <li><a href=\"".&utils::helpers::escape("$test.".&extensions($shellType)).'">'.&utils::helpers::escape($testDatabase->{$test}->{'def'})."</a> (#".&utils::helpers::escape($testDatabase->{$test}->{'number'}).")</li>";
    }
    print FILE '
  </ul>
  <p>See also: <a href="../../../index.html">Index</a>, <a href="../../index.html">'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$destinationType}).' Index</a>';
    foreach my $type (split ' ', $utils::helpers::types{'DESTINATION_TYPES'}) {
        if ($type ne $destinationType) {
            print FILE ', <a href="'.&utils::helpers::escape("../../../$type/index.html").'">'.&utils::helpers::escape($utils::helpers::DestinationTypeTitles{$type}).'</a>';
        }
    }
    print FILE '</p>
 </body>
</html>';
    close(FILE);
}


##############################################################################
# Test Meta Generators                                                       #
##############################################################################

sub generateMiniTest {
    my($destinationType, $tests, $testDatabase, $testIndex) = @_;
    my $func = UNIVERSAL::can(__PACKAGE__, "print_mini_${destinationType}");
    if (defined($func)) {
        &$func($tests, $testDatabase, $testIndex);
    } else {
        die("No generator defined for mini $destinationType tests. Aborted while processing test $tests->[$testIndex]");
    }
}

sub generateFlatTest {
    my($destinationType, $testType, $tests, $testDatabase, $testIndex) = @_;
    my $func = UNIVERSAL::can(__PACKAGE__, "print_flat_${destinationType}");
    if (defined($func)) {
        &$func($testType, $tests, $testDatabase, $testIndex);
    } else {
        die("No generator defined for full $destinationType tests. Aborted while processing test $tests->[$testIndex]");
    }
}

sub generateShell {
    my($destinationType, $testType, $shellType, $tests, $testDatabase, $testIndex) = @_;
    my $func = UNIVERSAL::can(__PACKAGE__, "print_shell_${shellType}");
    if (defined($func)) {
        &$func($destinationType, $testType, $tests, $testDatabase, $testIndex);
    } else {
        die("No generator defined for $shellType shells of $destinationType tests. Aborted while processing test $tests->[$testIndex]");
    }
}


##############################################################################
# Test Generators                                                            #
##############################################################################

sub print_mini_xhtml {
    my($tests, $testDatabase, $testIndex) = @_;
    my $name = $tests->[$testIndex];
    my $data = $testDatabase->{$tests->[$testIndex]};
    open(FILE, ">dist/xhtml/tests/$name.xml") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Strict//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd\">\n" unless defined($data->{'namespaced'});
    print FILE '<html xmlns="http://www.w3.org/1999/xhtml">
 <head>
  <title>'.&utils::helpers::escape($data->{'def'}).'</title>
  <style type="text/css"><![CDATA['."$data->{cssrules}]]></style>";
    if ($testIndex > 0) {
        print FILE "\n  <link rel=\"first\" href=\"".&utils::helpers::escape($tests->[0]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[0]}->{'def'}).'"/>';
        print FILE "\n  <link rel=\"prev\" href=\"".&utils::helpers::escape($tests->[$testIndex-1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'}).'"/>';
    }
    if ($testIndex < $#$tests) {
        print FILE "\n  <link rel=\"next\" href=\"".&utils::helpers::escape($tests->[$testIndex+1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'}).'"/>';
        print FILE "\n  <link rel=\"last\" href=\"".&utils::helpers::escape($tests->[$#$tests]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$#$tests]}->{'def'}).'"/>';
    }
    # XXX shoud list alternates (i.e. flat and each shell)
    print FILE "
  <link rel=\"up\" href=\"./index.html\"/>
  <link rel=\"top\" href=\"../../index.html\"/>
 </head>
 <body$data->{'namespaces'}>$data->{'code-xhtml'}</body>
</html>";
    close(FILE);
}

sub print_mini_html {
    my($tests, $testDatabase, $testIndex) = @_;
    my $name = $tests->[$testIndex];
    my $data = $testDatabase->{$tests->[$testIndex]};
    open(FILE, ">dist/html/tests/$name.html") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN">
<html>
 <head>
  <title>'.&utils::helpers::escape($data->{'def'}).'</title>
  <style type="text/css">'."$data->{cssrules}</style>";
    if ($testIndex > 0) {
        print FILE "\n  <link rel=\"first\" href=\"".&utils::helpers::escape($tests->[0]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[0]}->{'def'}).'">';
        print FILE "\n  <link rel=\"prev\" href=\"".&utils::helpers::escape($tests->[$testIndex-1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'}).'">';
    }
    if ($testIndex < $#$tests) {
        print FILE "\n  <link rel=\"next\" href=\"".&utils::helpers::escape($tests->[$testIndex+1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'}).'">';
        print FILE "\n  <link rel=\"last\" href=\"".&utils::helpers::escape($tests->[$#$tests]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$#$tests]}->{'def'}).'">';
    }
    # XXX shoud list alternates (i.e. flat and each shell)
    print FILE "
  <link rel=\"up\" href=\"./index.html\">
  <link rel=\"top\" href=\"../../index.html\">
 </head>
 <body>$data->{'code-html'}</body>
</html>";
    close(FILE);
}

sub print_mini_xml {
    my($tests, $testDatabase, $testIndex) = @_;
    my $name = $tests->[$testIndex];
    my $data = $testDatabase->{$tests->[$testIndex]};
    open(FILE, ">dist/xml/tests/$name.xml") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<?xml-stylesheet href="'.&utils::helpers::escape($name).".css\" type=\"text/css\"?>
<test$data->{'namespaces'}>$data->{'code-xml'}</test>";
    close(FILE);
    open(FILE, ">dist/xml/tests/$name.css") or die "failed to open output file: $!";
    print FILE $data->{cssrules};
    close(FILE);
}


sub print_flat_xhtml {
    my($testType, $tests, $testDatabase, $testIndex) = @_;
    my $name = $tests->[$testIndex];
    my $data = $testDatabase->{$tests->[$testIndex]};
    open(FILE, ">dist/xhtml/$testType/flat/$name.xml") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE "<?xml-stylesheet href=\"../../../style/xhtml-full.css\" type=\"text/css\"?>\n";
    print FILE "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Strict//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd\">\n" unless defined($data->{'namespaced'});
    print FILE '<html xmlns="http://www.w3.org/1999/xhtml">
 <head>
  <title>'.&utils::helpers::escape($data->{'def'}).'</title>
  <meta name="author" content="'.&utils::helpers::escape("@{$data->{'author'}}").'"/>
  <link rel="stylesheet" type="text/css" href="../../../style/xhtml-full.css"/> <!-- yes this means compliant UAs get to import this twice -->
  <style type="text/css"><![CDATA['."$data->{cssrules}]]></style>";
    if ($testIndex > 0) {
        print FILE "\n  <link rel=\"first\" href=\"".&utils::helpers::escape($tests->[0]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[0]}->{'def'}).'"/>';
        print FILE "\n  <link rel=\"prev\" href=\"".&utils::helpers::escape($tests->[$testIndex-1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'}).'"/>';
    }
    if ($testIndex < $#$tests) {
        print FILE "\n  <link rel=\"next\" href=\"".&utils::helpers::escape($tests->[$testIndex+1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'}).'"/>';
        print FILE "\n  <link rel=\"last\" href=\"".&utils::helpers::escape($tests->[$#$tests]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$#$tests]}->{'def'}).'"/>';
    }
    # XXX shoud list alternates (i.e. mini and each shell)
    print FILE '
  <link rel="up" href="./index.html"/>
  <link rel="top" href="../../../index.html"/>
 </head>
 <body>
  <table class="testDescription">
   <tr>
    <th class="b">CSS 3 Module</th> <!-- XXX hard coded to say CSS 3 -->
    <th class="c" colspan="2">';
    if ($testIndex > 0) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex-1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'})."\">&lt;==</a>";
    } else {
        print FILE "\n     &lt;==";
    }
    print FILE "\n     Test #";
    if ($testIndex < $#$tests) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex+1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'})."\">==&gt;</a>";
    } else {
        print FILE "\n     ==&gt;";
    }
    print FILE '
    </th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'module'}).'</td>
    <td class="c" colspan="2">'.($testIndex+1).' of '.(scalar(@$tests));
    if ($utils::helpers::TestTypeTitles{$testType} ne '') {
        print FILE ' of the '.&utils::helpers::escape($utils::helpers::TestTypeTitles{$testType});
    }
    print FILE '</td>
   </tr>
   <tr>
    <th class="b">Testing</th>
    <th class="a">Date</th>
    <th class="a">Revision</th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'def'}).' (ID #'.&utils::helpers::escape($data->{'number'}).')</td>
    <td class="a">'.&utils::helpers::escape($data->{'date'}).'</td>
    <td class="a">'.&utils::helpers::escape($data->{'rev'}).'</td>
   </tr>
  </table>';
    if (defined($data->{'interactive'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: User interaction is required for this test.</p>";
    }
    if (defined($data->{'historyneeded'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support the concept of a session history for this test.</p>";
    }
    if (defined($data->{'dynamic'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support ECMA-262 and DOM Level 2 Core for this test.</p>";
    }
    if (defined($data->{'namespaced'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support namespaces for this test.</p>";
    }
    print FILE "
  <div class=\"testSource\">
   <div class=\"testText\"$data->{'namespaces'}>$data->{'code-xhtml'}</div>
   <pre class=\"rules\">$data->{'escapedcode-css'}</pre>
   <pre class=\"rules\">$data->{'escapedcode-xhtml'}</pre>
  </div>
 </body>
</html>";
    close(FILE);
}

sub print_flat_html {
    my($testType, $tests, $testDatabase, $testIndex) = @_;
    my $name = $tests->[$testIndex];
    my $data = $testDatabase->{$tests->[$testIndex]};
    open(FILE, ">dist/html/$testType/flat/$name.html") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN">
<html>
 <head>
  <title>'.&utils::helpers::escape($data->{'def'}).'</title>
  <meta name="author" content="'.&utils::helpers::escape("@{$data->{'author'}}").'">
  <link rel="stylesheet" type="text/css" href="../../../style/html-full.css">
  <style type="text/css">'."$data->{cssrules}</style>";
    if ($testIndex > 0) {
        print FILE "\n  <link rel=\"first\" href=\"".&utils::helpers::escape($tests->[0]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[0]}->{'def'}).'">';
        print FILE "\n  <link rel=\"prev\" href=\"".&utils::helpers::escape($tests->[$testIndex-1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'}).'">';
    }
    if ($testIndex < $#$tests) {
        print FILE "\n  <link rel=\"next\" href=\"".&utils::helpers::escape($tests->[$testIndex+1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'}).'">';
        print FILE "\n  <link rel=\"last\" href=\"".&utils::helpers::escape($tests->[$#$tests]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$#$tests]}->{'def'}).'">';
    }
    # XXX shoud list alternates (i.e. mini and each shell)
    print FILE '
  <link rel="up" href="./index.html">
  <link rel="top" href="../../../index.html">
 </head>
 <body>
  <table class="testDescription">
   <tr>
    <th class="b">CSS 3 Module</th> <!-- XXX hard coded to say CSS 3 -->
    <th class="c" colspan="2">';
    if ($testIndex > 0) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex-1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'})."\">&lt;==</a>";
    } else {
        print FILE "\n     &lt;==";
    }
    print FILE "\n     Test #";
    if ($testIndex < $#$tests) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex+1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'})."\">==&gt;</a>";
    } else {
        print FILE "\n     ==&gt;";
    }
    print FILE '
    </th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'module'}).'</td>
    <td class="c" colspan="2">'.($testIndex+1).' of '.(scalar(@$tests));
    if ($utils::helpers::TestTypeTitles{$testType} ne '') {
        print FILE ' of the '.&utils::helpers::escape($utils::helpers::TestTypeTitles{$testType});
    }
    print FILE '</td>
   </tr>
   <tr>
    <th class="b">Testing</th>
    <th class="a">Date</th>
    <th class="a">Revision</th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'def'}).' (ID #'.&utils::helpers::escape($data->{'number'}).')</td>
    <td class="a">'.&utils::helpers::escape($data->{'date'}).'</td>
    <td class="a">'.&utils::helpers::escape($data->{'rev'}).'</td>
   </tr>
  </table>';
    if (defined($data->{'interactive'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: User interaction is required for this test.</p>";
    }
    if (defined($data->{'historyneeded'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support the concept of a session history for this test.</p>";
    }
    if (defined($data->{'dynamic'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support ECMA-262 and DOM Level 2 Core for this test.</p>";
    }
    if (defined($data->{'only-xml'})) {
        die("Inconsistency error: XML-specific test $name passed to HTML test generator");
    }
    print FILE "
  <div class=\"testSource\">
   <div class=\"testText\">$data->{'code-html'}</div>
   <pre class=\"rules\">$data->{'escapedcode-css'}</pre>
   <pre class=\"rules\">$data->{'escapedcode-html'}</pre>
  </div>
 </body>
</html>";
    close(FILE);
}

sub print_flat_xml {
    my($testType, $tests, $testDatabase, $testIndex) = @_;
    my $name = $tests->[$testIndex];
    my $data = $testDatabase->{$tests->[$testIndex]};
    open(FILE, ">dist/xml/$testType/flat/$name.xml") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<?xml-stylesheet href="../../../style/xml-full.css" type="text/css"?>
<?xml-stylesheet href="'.&utils::helpers::escape($name).'.css" type="text/css"?>
<test xmlns:xlink="http://www.w3.org/1999/xlink">
 <title>'.&utils::helpers::escape($data->{'def'}).'</title>';
    foreach my $author (@{$data->{'author'}}) {
        print FILE "\n <author>".&utils::helpers::escape($author).'</author>';
    }
    print FILE '
 <metadata>
  <item> <name>CSS 3 Module</name> <data>'.&utils::helpers::escape($data->{'module'}).'</data> </item>'; # XXX HARD CODED
    if ($testIndex < $#$tests) {
        print FILE "\n  <item> <name>Next</name> <data xlink:type=\"simple\" xlink:href=\"".&utils::helpers::escape($tests->[$testIndex+1]).'.xml">'.($testDatabase->{$tests->[$testIndex+1]}->{'def'}).'</data> </item>';
    }
    if ($testIndex > 0) {
        print FILE "\n  <item> <name>Previous</name> <data xlink:type=\"simple\" xlink:href=\"".&utils::helpers::escape($tests->[$testIndex-1]).'.xml">'.($testDatabase->{$tests->[$testIndex-1]}->{'def'}).'</data> </item>';
    }
    print FILE '
  <item> <name>Test #</name> <data>'.($testIndex+1).' of '.(scalar(@$tests));
    if ($utils::helpers::TestTypeTitles{$testType} ne '') {
        print FILE ' of the '.&utils::helpers::escape($utils::helpers::TestTypeTitles{$testType});
    }
    print FILE '</data> </item>
  <item> <name>Testing</name> <data>'.&utils::helpers::escape($data->{'def'}).'</data> </item>
  <item> <name>ID</name> <data>'.&utils::helpers::escape($data->{'number'}).'</data> </item>
  <item> <name>Date</name> <data>'.&utils::helpers::escape($data->{'date'}).'</data> </item>
  <item> <name>Revision</name> <data>'.&utils::helpers::escape($data->{'rev'}).'</data> </item>
 </metadata>';
    if (defined($data->{'interactive'})) {
        print FILE "\n   <requirement>User interaction is required for this test.</requirement>";
    }
    if (defined($data->{'historyneeded'})) {
        print FILE "\n   <requirement>The UA must support the concept of a session history for this test.</requirement>";
    }
    if (defined($data->{'dynamic'})) {
        print FILE "\n   <requirement>The UA must support ECMA-262 and DOM Level 2 Core for this test.</requirement>";
    }
    if (defined($data->{'namespaced'})) {
        print FILE "\n   <requirement>The UA must support namespaces for this test.</requirement>";
    }
    print FILE "
  <content$data->{'namespaces'}>$data->{'code-xml'}</content>
  <source>
   <css>$data->{'escapedcode-css'}</css>
   <xml>$data->{'escapedcode-xml'}</xml>
  </source>
</test>";
    close(FILE);
    # XXX we generate one of these for each test file --
    # XXX we could put them up one directory, or just use the stylesheets from the mini tests
    open(FILE, ">dist/xml/$testType/flat/$name.css") or die "failed to open output file: $!";
    print FILE $data->{cssrules};
    close(FILE);
}


sub print_shell_xhtml_iframe {
    my($destinationType, $testType, $tests, $testDatabase, $testIndex) = @_;
    my $name = $tests->[$testIndex];
    my $data = $testDatabase->{$tests->[$testIndex]};
    open(FILE, ">dist/$destinationType/$testType/xhtml_iframe/$name.xml") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<?xml-stylesheet href="../../../style/xhtml-shell.css" type="text/css"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Frameset//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-frameset.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
 <head>
  <title>'.&utils::helpers::escape($data->{'def'}).'</title>
  <meta name="author" content="'.&utils::helpers::escape("@{$data->{'author'}}").'"/>
  <link rel="stylesheet" type="text/css" href="../../../style/xhtml-shell.css"/> <!-- yes this means compliant UAs get to import this twice -->';
    if ($testIndex > 0) {
        print FILE "\n  <link rel=\"first\" href=\"".&utils::helpers::escape($tests->[0]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[0]}->{'def'}).'"/>';
        print FILE "\n  <link rel=\"prev\" href=\"".&utils::helpers::escape($tests->[$testIndex-1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'}).'"/>';
    }
    if ($testIndex < $#$tests) {
        print FILE "\n  <link rel=\"next\" href=\"".&utils::helpers::escape($tests->[$testIndex+1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'}).'"/>';
        print FILE "\n  <link rel=\"last\" href=\"".&utils::helpers::escape($tests->[$#$tests]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$#$tests]}->{'def'}).'"/>';
    }
    # XXX shoud list alternates (i.e. mini, flat and the other shells)
    print FILE '
  <link rel="up" href="./index.html"/>
  <link rel="top" href="../../../index.html"/>
 </head>
 <body>
  <table class="testDescription">
   <tr>
    <th class="b">CSS 3 Module</th> <!-- XXX hard coded to say CSS 3 -->
    <th class="c" colspan="2">';
    if ($testIndex > 0) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex-1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'})."\">&lt;==</a>";
    } else {
        print FILE "\n     &lt;==";
    }
    print FILE "\n     Test #";
    if ($testIndex < $#$tests) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex+1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'})."\">==&gt;</a>";
    } else {
        print FILE "\n     ==&gt;";
    }
    print FILE '
    </th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'module'}).'</td>
    <td class="c" colspan="2">'.($testIndex+1).' of '.(scalar(@$tests));
    if ($utils::helpers::TestTypeTitles{$testType} ne '') {
        print FILE ' of the '.&utils::helpers::escape($utils::helpers::TestTypeTitles{$testType});
    }
    print FILE '</td>
   </tr>
   <tr>
    <th class="b">Testing</th>
    <th class="a">Date</th>
    <th class="a">Revision</th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'def'}).' (ID #'.&utils::helpers::escape($data->{'number'}).')</td>
    <td class="a">'.&utils::helpers::escape($data->{'date'}).'</td>
    <td class="a">'.&utils::helpers::escape($data->{'rev'}).'</td>
   </tr>
  </table>';
    if (defined($data->{'interactive'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: User interaction is required for this test.</p>";
    }
    if (defined($data->{'historyneeded'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support the concept of a session history for this test.</p>";
    }
    if (defined($data->{'dynamic'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support ECMA-262 and DOM Level 2 Core for this test.</p>";
    }
    if (defined($data->{'namespaced'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support namespaces for this test.</p>";
    }
    my $extension = &extensions($destinationType); # having the extension in the filename is so wrong...
    print FILE "
  <iframe src=\"../../tests/$name.$extension\"/>
 </body>
</html>";
    close(FILE);
}

sub print_shell_xhtml_object {
    my($destinationType, $testType, $tests, $testDatabase, $testIndex) = @_;
    my $name = $tests->[$testIndex];
    my $data = $testDatabase->{$tests->[$testIndex]};
    open(FILE, ">dist/$destinationType/$testType/xhtml_object/$name.xml") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<?xml-stylesheet href="../../../style/xhtml-shell.css" type="text/css"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
 <head>
  <title>'.&utils::helpers::escape($data->{'def'}).'</title>
  <meta name="author" content="'.&utils::helpers::escape("@{$data->{'author'}}").'"/>
  <link rel="stylesheet" type="text/css" href="../../../style/xhtml-shell.css"/> <!-- yes this means compliant UAs get to import this twice -->';
    if ($testIndex > 0) {
        print FILE "\n  <link rel=\"first\" href=\"".&utils::helpers::escape($tests->[0]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[0]}->{'def'}).'"/>';
        print FILE "\n  <link rel=\"prev\" href=\"".&utils::helpers::escape($tests->[$testIndex-1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'}).'"/>';
    }
    if ($testIndex < $#$tests) {
        print FILE "\n  <link rel=\"next\" href=\"".&utils::helpers::escape($tests->[$testIndex+1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'}).'"/>';
        print FILE "\n  <link rel=\"last\" href=\"".&utils::helpers::escape($tests->[$#$tests]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$#$tests]}->{'def'}).'"/>';
    }
    # XXX shoud list alternates (i.e. mini, flat and the other shells)
    print FILE '
  <link rel="up" href="./index.html"/>
  <link rel="top" href="../../../index.html"/>
 </head>
 <body>
  <table class="testDescription">
   <tr>
    <th class="b">CSS 3 Module</th> <!-- XXX hard coded to say CSS 3 -->
    <th class="c" colspan="2">';
    if ($testIndex > 0) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex-1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'})."\">&lt;==</a>";
    } else {
        print FILE "\n     &lt;==";
    }
    print FILE "\n     Test #";
    if ($testIndex < $#$tests) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex+1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'})."\">==&gt;</a>";
    } else {
        print FILE "\n     ==&gt;";
    }
    print FILE '
    </th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'module'}).'</td>
    <td class="c" colspan="2">'.($testIndex+1).' of '.(scalar(@$tests));
    if ($utils::helpers::TestTypeTitles{$testType} ne '') {
        print FILE ' of the '.&utils::helpers::escape($utils::helpers::TestTypeTitles{$testType});
    }
    print FILE '</td>
   </tr>
   <tr>
    <th class="b">Testing</th>
    <th class="a">Date</th>
    <th class="a">Revision</th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'def'}).' (ID #'.&utils::helpers::escape($data->{'number'}).')</td>
    <td class="a">'.&utils::helpers::escape($data->{'date'}).'</td>
    <td class="a">'.&utils::helpers::escape($data->{'rev'}).'</td>
   </tr>
  </table>';
    if (defined($data->{'interactive'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: User interaction is required for this test.</p>";
    }
    if (defined($data->{'historyneeded'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support the concept of a session history for this test.</p>";
    }
    if (defined($data->{'dynamic'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support ECMA-262 and DOM Level 2 Core for this test.</p>";
    }
    if (defined($data->{'namespaced'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support namespaces for this test.</p>";
    }
    my $extension = &extensions($destinationType); # having the extension in the filename is so wrong...
    print FILE "
  <object data=\"../../tests/$name.$extension\"/>
 </body>
</html>";
    close(FILE);
}

sub print_shell_xhtml_frames {
    my($destinationType, $testType, $tests, $testDatabase, $testIndex) = @_;
    my $name = $tests->[$testIndex];
    my $data = $testDatabase->{$tests->[$testIndex]};
    my $extension = &extensions($destinationType); # having the extension in the filename is so wrong...
    my $topframe = &utils::helpers::escape("$name-top.xml");
    my $bottomframe = &utils::helpers::escape("../../tests/$name.$extension");
    open(FILE, ">dist/$destinationType/$testType/xhtml_frames/$name-top.xml") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<?xml-stylesheet href="../../../style/xhtml-shell.css" type="text/css"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Frameset//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-frameset.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
 <head>
  <title>'.&utils::helpers::escape($data->{'def'}).'</title>
  <meta name="author" content="'.&utils::helpers::escape("@{$data->{'author'}}").'"/>
  <link rel="stylesheet" type="text/css" href="../../../style/xhtml-shell.css" target="_top"/> <!-- yes this means compliant UAs get to import this twice -->';
    if ($testIndex > 0) {
        print FILE "\n  <link rel=\"first\" href=\"".&utils::helpers::escape($tests->[0]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[0]}->{'def'}).'" target="_top"/>';
        print FILE "\n  <link rel=\"prev\" href=\"".&utils::helpers::escape($tests->[$testIndex-1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'})."\" target=\"_top\"/>";
    }
    if ($testIndex < $#$tests) {
        print FILE "\n  <link rel=\"next\" href=\"".&utils::helpers::escape($tests->[$testIndex+1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'})."\" target=\"_top\"/>";
        print FILE "\n  <link rel=\"last\" href=\"".&utils::helpers::escape($tests->[$#$tests]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$#$tests]}->{'def'}).'" target="_top"/>';
    }
    # XXX shoud list alternates (i.e. mini, flat and the other shells)
    print FILE '
  <link rel="up" href="./index.html" target="_top"/>
  <link rel="top" href="../../../index.html" target="_top"/>
 </head>
 <body>
  <table class="testDescription">
   <tr>
    <th class="b">CSS 3 Module</th> <!-- XXX hard coded to say CSS 3 -->
    <th class="c" colspan="2">';
    if ($testIndex > 0) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex-1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'})."\" target=\"_top\">&lt;==</a>";
    } else {
        print FILE "\n     &lt;==";
    }
    print FILE "\n     Test #";
    if ($testIndex < $#$tests) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex+1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'})."\" target=\"_top\">==&gt;</a>";
    } else {
        print FILE "\n     ==&gt;";
    }
    print FILE '
    </th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'module'}).'</td>
    <td class="c" colspan="2">'.($testIndex+1).' of '.(scalar(@$tests));
    if ($utils::helpers::TestTypeTitles{$testType} ne '') {
        print FILE ' of the '.&utils::helpers::escape($utils::helpers::TestTypeTitles{$testType});
    }
    print FILE '</td>
   </tr>
   <tr>
    <th class="b">Testing</th>
    <th class="a">Date</th>
    <th class="a">Revision</th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'def'}).' (ID #'.&utils::helpers::escape($data->{'number'}).')</td>
    <td class="a">'.&utils::helpers::escape($data->{'date'}).'</td>
    <td class="a">'.&utils::helpers::escape($data->{'rev'}).'</td>
   </tr>
  </table>';
    if (defined($data->{'interactive'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: User interaction is required for this test.</p>";
    }
    if (defined($data->{'historyneeded'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support the concept of a session history for this test.</p>";
    }
    if (defined($data->{'dynamic'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support ECMA-262 and DOM Level 2 Core for this test.</p>";
    }
    if (defined($data->{'namespaced'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support namespaces for this test.</p>";
    }
    print FILE "
 </body>
</html>";
    close(FILE);
    open(FILE, ">dist/$destinationType/$testType/xhtml_frames/$name.xml") or die "failed to open output file: $!";
    print FILE '<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Frameset//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-frameset.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
 <head>
  <title>'.&utils::helpers::escape($data->{'def'}).'</title>
  <meta name="author" content="'.&utils::helpers::escape("@{$data->{'author'}}").'"/>';
    if ($testIndex > 0) {
        print FILE "\n  <link rel=\"first\" href=\"".&utils::helpers::escape($tests->[0]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[0]}->{'def'}).'" target="_top"/>';
        print FILE "\n  <link rel=\"prev\" href=\"".&utils::helpers::escape($tests->[$testIndex-1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'})."\" target=\"_top\"/>";
    }
    if ($testIndex < $#$tests) {
        print FILE "\n  <link rel=\"next\" href=\"".&utils::helpers::escape($tests->[$testIndex+1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'})."\" target=\"_top\"/>";
        print FILE "\n  <link rel=\"last\" href=\"".&utils::helpers::escape($tests->[$#$tests]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$#$tests]}->{'def'}).'" target="_top"/>';
    }
    print FILE "
  <link rel=\"up\" href=\"./index.html\" target=\"_top\"/>
  <link rel=\"top\" href=\"../../../index.html\" target=\"_top\"/>
 </head>
 <frameset rows=\"35%,*\" cols=\"*\">
  <frame src=\"$topframe\"/>
  <frame src=\"$bottomframe\"/>
 </frameset>
</html>";
    close(FILE);
}

sub print_shell_html_iframe {
    my($destinationType, $testType, $tests, $testDatabase, $testIndex) = @_;
    my $name = $tests->[$testIndex];
    my $data = $testDatabase->{$tests->[$testIndex]};
    open(FILE, ">dist/$destinationType/$testType/html_iframe/$name.html") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01 Frameset//EN">
<html>
 <head>
  <title>'.&utils::helpers::escape($data->{'def'}).'</title>
  <meta name="author" content="'.&utils::helpers::escape("@{$data->{'author'}}").'">
  <link rel="stylesheet" type="text/css" href="../../../style/html-shell.css"> <!-- yes this means compliant UAs get to import this twice -->';
    if ($testIndex > 0) {
        print FILE "\n  <link rel=\"first\" href=\"".&utils::helpers::escape($tests->[0]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[0]}->{'def'}).'">';
        print FILE "\n  <link rel=\"prev\" href=\"".&utils::helpers::escape($tests->[$testIndex-1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'}).'">';
    }
    if ($testIndex < $#$tests) {
        print FILE "\n  <link rel=\"next\" href=\"".&utils::helpers::escape($tests->[$testIndex+1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'}).'">';
        print FILE "\n  <link rel=\"last\" href=\"".&utils::helpers::escape($tests->[$#$tests]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$#$tests]}->{'def'}).'">';
    }
    # XXX shoud list alternates (i.e. mini, flat and the other shells)
    print FILE '
  <link rel="up" href="./index.html">
  <link rel="top" href="../../../index.html">
 </head>
 <body>
  <table class="testDescription">
   <tr>
    <th class="b">CSS 3 Module</th> <!-- XXX hard coded to say CSS 3 -->
    <th class="c" colspan="2">';
    if ($testIndex > 0) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex-1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'})."\">&lt;==</a>";
    } else {
        print FILE "\n     &lt;==";
    }
    print FILE "\n     Test #";
    if ($testIndex < $#$tests) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex+1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'})."\">==&gt;</a>";
    } else {
        print FILE "\n     ==&gt;";
    }
    print FILE '
    </th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'module'}).'</td>
    <td class="c" colspan="2">'.($testIndex+1).' of '.(scalar(@$tests));
    if ($utils::helpers::TestTypeTitles{$testType} ne '') {
        print FILE ' of the '.&utils::helpers::escape($utils::helpers::TestTypeTitles{$testType});
    }
    print FILE '</td>
   </tr>
   <tr>
    <th class="b">Testing</th>
    <th class="a">Date</th>
    <th class="a">Revision</th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'def'}).' (ID #'.&utils::helpers::escape($data->{'number'}).')</td>
    <td class="a">'.&utils::helpers::escape($data->{'date'}).'</td>
    <td class="a">'.&utils::helpers::escape($data->{'rev'}).'</td>
   </tr>
  </table>';
    if (defined($data->{'interactive'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: User interaction is required for this test.</p>";
    }
    if (defined($data->{'historyneeded'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support the concept of a session history for this test.</p>";
    }
    if (defined($data->{'dynamic'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support ECMA-262 and DOM Level 2 Core for this test.</p>";
    }
    if (defined($data->{'namespaced'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support namespaces for this test.</p>";
    }
    my $extension = &extensions($destinationType); # having the extension in the filename is so wrong...
    print FILE "
  <iframe src=\"../../tests/$name.$extension\">
 </body>
</html>";
    close(FILE);
}

sub print_shell_html_object {
    my($destinationType, $testType, $tests, $testDatabase, $testIndex) = @_;
    my $name = $tests->[$testIndex];
    my $data = $testDatabase->{$tests->[$testIndex]};
    open(FILE, ">dist/$destinationType/$testType/html_object/$name.html") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN">
<html>
 <head>
  <title>'.&utils::helpers::escape($data->{'def'}).'</title>
  <meta name="author" content="'.&utils::helpers::escape("@{$data->{'author'}}").'">
  <link rel="stylesheet" type="text/css" href="../../../style/html-shell.css">';
    if ($testIndex > 0) {
        print FILE "\n  <link rel=\"first\" href=\"".&utils::helpers::escape($tests->[0]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[0]}->{'def'}).'">';
        print FILE "\n  <link rel=\"prev\" href=\"".&utils::helpers::escape($tests->[$testIndex-1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'}).'">';
    }
    if ($testIndex < $#$tests) {
        print FILE "\n  <link rel=\"next\" href=\"".&utils::helpers::escape($tests->[$testIndex+1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'}).'">';
        print FILE "\n  <link rel=\"last\" href=\"".&utils::helpers::escape($tests->[$#$tests]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$#$tests]}->{'def'}).'">';
    }
    # XXX shoud list alternates (i.e. mini, flat and the other shells)
    print FILE '
  <link rel="up" href="./index.html">
  <link rel="top" href="../../../index.html">
 </head>
 <body>
  <table class="testDescription">
   <tr>
    <th class="b">CSS 3 Module</th> <!-- XXX hard coded to say CSS 3 -->
    <th class="c" colspan="2">';
    if ($testIndex > 0) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex-1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'})."\">&lt;==</a>";
    } else {
        print FILE "\n     &lt;==";
    }
    print FILE "\n     Test #";
    if ($testIndex < $#$tests) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex+1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'})."\">==&gt;</a>";
    } else {
        print FILE "\n     ==&gt;";
    }
    print FILE '
    </th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'module'}).'</td>
    <td class="c" colspan="2">'.($testIndex+1).' of '.(scalar(@$tests));
    if ($utils::helpers::TestTypeTitles{$testType} ne '') {
        print FILE ' of the '.&utils::helpers::escape($utils::helpers::TestTypeTitles{$testType});
    }
    print FILE '</td>
   </tr>
   <tr>
    <th class="b">Testing</th>
    <th class="a">Date</th>
    <th class="a">Revision</th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'def'}).' (ID #'.&utils::helpers::escape($data->{'number'}).')</td>
    <td class="a">'.&utils::helpers::escape($data->{'date'}).'</td>
    <td class="a">'.&utils::helpers::escape($data->{'rev'}).'</td>
   </tr>
  </table>';
    if (defined($data->{'interactive'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: User interaction is required for this test.</p>";
    }
    if (defined($data->{'historyneeded'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support the concept of a session history for this test.</p>";
    }
    if (defined($data->{'dynamic'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support ECMA-262 and DOM Level 2 Core for this test.</p>";
    }
    if (defined($data->{'namespaced'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support namespaces for this test.</p>";
    }
    my $extension = &extensions($destinationType); # having the extension in the filename is so wrong...
    print FILE "
  <object data=\"../../tests/$name.$extension\"></object>
 </body>
</html>";
    close(FILE);
}

sub print_shell_tng {
    my($destinationType, $testType, $tests, $testDatabase, $testIndex) = @_;
    my $name = $tests->[$testIndex];
    my $data = $testDatabase->{$tests->[$testIndex]};
    open(FILE, ">dist/$destinationType/$testType/tng/$name.html") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.0 Transitional//EN" "http://www.w3.org/TR/REC-html40/loose.dtd">
<HTML>
<HEAD>
<TITLE>CSS3 Test Suite: '.&utils::helpers::escape($data->{'def'}).'</TITLE>
<META http-equiv="Content-Type" content="text/html; charset=iso-8859-1">
<META http-equiv="Content-Style-Type" content="text/css">

<LINK rel="stylesheet" type="text/css" media="screen" href="../../../style/tng.css"></HEAD>

<BODY>
<DIV class="navigation">
<H2>CSS3 Test Suite: '.&utils::helpers::escape($data->{'def'}).'</H2>

<HR>';
    if ($testIndex > 0) {
        print FILE "\n[<A HREF=\"".&utils::helpers::escape($tests->[$testIndex-1]).".html\">Previous</A>]";
    } else {
        print FILE "\n[Previous]";
    }
    if ($testIndex < $#$tests) {
        print FILE ' <A HREF="'.&utils::helpers::escape($tests->[$testIndex+1]).'.html">[Next]</A>';
    } else {
        print FILE ' <A HREF="../index.html">[Next]</A>';
    }
    print FILE ' <A HREF="index.html">[Contents]</A>';

    my $extension = &extensions($destinationType); # having the extension in the filename is so wrong...
    print FILE '<BR>';

    if (defined($data->{'interactive'})) {
        print FILE "\n<P CLASS=\"WARNING\">NOTE: User interaction is required for this test.</P>";
    }
    if (defined($data->{'historyneeded'})) {
        print FILE "\n<P CLASS=\"WARNING\">NOTE: The UA must support the concept of a session history for this test.</P>";
    }
    if (defined($data->{'dynamic'})) {
        print FILE "\n<P CLASS=\"WARNING\">NOTE: The UA must support ECMA-262 and DOM Level 2 Core for this test.</P>";
    }
    if (defined($data->{'namespaced'})) {
        print FILE "\n<P CLASS=\"WARNING\">NOTE: The UA must support namespaces for this test.</P>";

    }
    print FILE '
</DIV>
<OBJECT height="100%" width="100%" border="0" type="text/html" data="'."../../tests/$name.$extension".'"><A class="navigation" href="'."../../tests/$name.$extension".'" target="testwindow">Test</A></OBJECT>
</BODY>
</HTML>';
    close(FILE);
}

sub print_shell_html_frames {
    my($destinationType, $testType, $tests, $testDatabase, $testIndex) = @_;
    my $name = $tests->[$testIndex];
    my $data = $testDatabase->{$tests->[$testIndex]};
    my $extension = &extensions($destinationType); # having the extension in the filename is so wrong...
    my $topframe = &utils::helpers::escape("$name-top.html");
    my $bottomframe = &utils::helpers::escape("../../tests/$name.$extension");
    open(FILE, ">dist/$destinationType/$testType/html_frames/$name-top.html") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01 Frameset//EN">
<html>
 <head>
  <title>'.&utils::helpers::escape($data->{'def'}).'</title>
  <meta name="author" content="'.&utils::helpers::escape("@{$data->{'author'}}").'">
  <link rel="stylesheet" type="text/css" href="../../../style/html-shell.css" target="_top"> <!-- yes this means compliant UAs get to import this twice -->';
    if ($testIndex > 0) {
        print FILE "\n  <link rel=\"first\" href=\"".&utils::helpers::escape($tests->[0]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[0]}->{'def'}).'" target="_top">';
        print FILE "\n  <link rel=\"prev\" href=\"".&utils::helpers::escape($tests->[$testIndex-1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'})."\" target=\"_top\">";
    }
    if ($testIndex < $#$tests) {
        print FILE "\n  <link rel=\"next\" href=\"".&utils::helpers::escape($tests->[$testIndex+1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'})."\" target=\"_top\">";
        print FILE "\n  <link rel=\"last\" href=\"".&utils::helpers::escape($tests->[$#$tests]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$#$tests]}->{'def'}).'" target="_top">';
    }
    # XXX shoud list alternates (i.e. mini, flat and the other shells)
    print FILE '
  <link rel="up" href="./index.html" target="_top">
  <link rel="top" href="../../../index.html" target="_top">
 </head>
 <body>
  <table class="testDescription">
   <tr>
    <th class="b">CSS 3 Module</th> <!-- XXX hard coded to say CSS 3 -->
    <th class="c" colspan="2">';
    if ($testIndex > 0) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex-1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'})."\" target=\"_top\">&lt;==</a>";
    } else {
        print FILE "\n     &lt;==";
    }
    print FILE "\n     Test #";
    if ($testIndex < $#$tests) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex+1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'})."\" target=\"_top\">==&gt;</a>";
    } else {
        print FILE "\n     ==&gt;";
    }
    print FILE '
    </th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'module'}).'</td>
    <td class="c" colspan="2">'.($testIndex+1).' of '.(scalar(@$tests));
    if ($utils::helpers::TestTypeTitles{$testType} ne '') {
        print FILE ' of the '.&utils::helpers::escape($utils::helpers::TestTypeTitles{$testType});
    }
    print FILE '</td>
   </tr>
   <tr>
    <th class="b">Testing</th>
    <th class="a">Date</th>
    <th class="a">Revision</th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'def'}).' (ID #'.&utils::helpers::escape($data->{'number'}).')</td>
    <td class="a">'.&utils::helpers::escape($data->{'date'}).'</td>
    <td class="a">'.&utils::helpers::escape($data->{'rev'}).'</td>
   </tr>
  </table>';
    if (defined($data->{'interactive'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: User interaction is required for this test.</p>";
    }
    if (defined($data->{'historyneeded'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support the concept of a session history for this test.</p>";
    }
    if (defined($data->{'dynamic'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support ECMA-262 and DOM Level 2 Core for this test.</p>";
    }
    if (defined($data->{'namespaced'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support namespaces for this test.</p>";
    }
    print FILE "
 </body>
</html>";
    close(FILE);
    open(FILE, ">dist/$destinationType/$testType/html_frames/$name.html") or die "failed to open output file: $!";
    print FILE '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01 Frameset//EN">
<html>
 <head>
  <title>'.&utils::helpers::escape($data->{'def'}).'</title>
  <meta name="author" content="'.&utils::helpers::escape("@{$data->{'author'}}").'"/>';
    if ($testIndex > 0) {
        print FILE "\n  <link rel=\"first\" href=\"".&utils::helpers::escape($tests->[0]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[0]}->{'def'}).'" target="_top">';
        print FILE "\n  <link rel=\"prev\" href=\"".&utils::helpers::escape($tests->[$testIndex-1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'})."\" target=\"_top\">";
    }
    if ($testIndex < $#$tests) {
        print FILE "\n  <link rel=\"next\" href=\"".&utils::helpers::escape($tests->[$testIndex+1]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'})."\" target=\"_top\">";
        print FILE "\n  <link rel=\"last\" href=\"".&utils::helpers::escape($tests->[$#$tests]).".html\" title=\"".&utils::helpers::escape($testDatabase->{$tests->[$#$tests]}->{'def'}).'" target="_top">';
    }
    # XXX shoud list alternates (i.e. mini, flat and the other shells)
    print FILE "
  <link rel=\"up\" href=\"./index.html\" target=\"_top\">
  <link rel=\"top\" href=\"../../../index.html\" target=\"_top\">
 </head>
 <frameset rows=\"35%,*\" cols=\"*\">
  <frame src=\"$topframe\">
  <frame src=\"$bottomframe\">
 </frameset>
</html>";
    close(FILE);
}

sub print_shell_xlink_embed {
    my($destinationType, $testType, $tests, $testDatabase, $testIndex) = @_;
    my $name = $tests->[$testIndex];
    my $data = $testDatabase->{$tests->[$testIndex]};
    open(FILE, ">dist/$destinationType/$testType/xlink_embed/$name.xml") or die "failed to open output file: $!";
    local $" = ', ';
    print FILE '<?xml-stylesheet href="../../../style/xhtml-shell.css" type="text/css"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
 <head>
  <title>'.&utils::helpers::escape($data->{'def'}).'</title>
  <meta name="author" content="'.&utils::helpers::escape("@{$data->{'author'}}").'"/>
  <link rel="stylesheet" type="text/css" href="../../../style/xhtml-shell.css"/> <!-- yes this means compliant UAs get to import this twice -->';
    if ($testIndex > 0) {
        print FILE "\n  <link rel=\"first\" href=\"".&utils::helpers::escape($tests->[0]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[0]}->{'def'}).'"/>';
        print FILE "\n  <link rel=\"prev\" href=\"".&utils::helpers::escape($tests->[$testIndex-1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'}).'"/>';
    }
    if ($testIndex < $#$tests) {
        print FILE "\n  <link rel=\"next\" href=\"".&utils::helpers::escape($tests->[$testIndex+1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'}).'"/>';
        print FILE "\n  <link rel=\"last\" href=\"".&utils::helpers::escape($tests->[$#$tests]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$#$tests]}->{'def'}).'"/>';
    }
    # XXX shoud list alternates (i.e. mini, flat and the other shells)
    print FILE '
  <link rel="up" href="./index.html"/>
  <link rel="top" href="../../../index.html"/>
 </head>
 <body>
  <table class="testDescription">
   <tr>
    <th class="b">CSS 3 Module</th> <!-- XXX hard coded to say CSS 3 -->
    <th class="c" colspan="2">';
    if ($testIndex > 0) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex-1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex-1]}->{'def'})."\">&lt;==</a>";
    } else {
        print FILE "\n     &lt;==";
    }
    print FILE "\n     Test #";
    if ($testIndex < $#$tests) {
        print FILE "\n     <a href=\"".&utils::helpers::escape($tests->[$testIndex+1]).'.xml" title="'.&utils::helpers::escape($testDatabase->{$tests->[$testIndex+1]}->{'def'})."\">==&gt;</a>";
    } else {
        print FILE "\n     ==&gt;";
    }
    print FILE '
    </th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'module'}).'</td>
    <td class="c" colspan="2">'.($testIndex+1).' of '.(scalar(@$tests));
    if ($utils::helpers::TestTypeTitles{$testType} ne '') {
        print FILE ' of the '.&utils::helpers::escape($utils::helpers::TestTypeTitles{$testType});
    }
    print FILE '</td>
   </tr>
   <tr>
    <th class="b">Testing</th>
    <th class="a">Date</th>
    <th class="a">Revision</th>
   </tr>
   <tr>
    <td class="b">'.&utils::helpers::escape($data->{'def'}).' (ID #'.&utils::helpers::escape($data->{'number'}).')</td>
    <td class="a">'.&utils::helpers::escape($data->{'date'}).'</td>
    <td class="a">'.&utils::helpers::escape($data->{'rev'}).'</td>
   </tr>
  </table>';
    if (defined($data->{'interactive'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: User interaction is required for this test.</p>";
    }
    if (defined($data->{'historyneeded'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support the concept of a session history for this test.</p>";
    }
    if (defined($data->{'dynamic'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support ECMA-262 and DOM Level 2 Core for this test.</p>";
    }
    if (defined($data->{'namespaced'})) {
        print FILE "\n  <p class=\"WARNING\">NOTE: The UA must support namespaces for this test.</p>";
    }
    my $extension = &extensions($destinationType); # having the extension in the filename is so wrong...
    print FILE "
  <div xmlns:xlink=\"http://www.w3.org/1999/xlink\" xlink:type=\"simple\" xlink:show=\"embed\" xlink:href=\"../../../tests/$name.$extension\"/>
 </body>
</html>";
    close(FILE);
}

##############################################################################
