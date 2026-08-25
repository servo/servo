#!/usr/bin/perl
#
#  make_tests.pl - generate WPT test cases from the testable statements wiki
#
#  This script assumes that a wiki has testable statement entries
#  in the format described by the specification at
#  https://spec-ops.github.io/atta-api/index.html
#
#  usage: make_tests.pl -f file | -w wiki_title | -s spec -d dir

use strict;

use IO::String ;
use JSON ;
use MediaWiki::API ;
use Getopt::Long;

my %specs = (
    "aria11" => {
      title => "ARIA_1.1_Testable_Statements",
      specURL => "https://www.w3.org/TR/wai-aria11/",
      dir => "aria11"
    },
    "svg" => {
      title => "SVG_Accessibility/Testing/Test_Assertions_with_Tables_for_ATTA",
      specURL => "https://www.w3.org/TR/svg-aam-1.0/",
      dir => "svg",
      fragment => '<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">%code%</svg>'
    }
);

my @apiNames = qw(UIA MSAA ATK IAccessible2 AXAPI);
my $apiNamesRegex = "(" . join("|", @apiNames) . ")";

# the suffix to attach to the automatically generated test case names
my $theSuffix = "-manual.html";

# dir is determined based upon the short name of the spec and is defined
# by the input or on the command line

my $file = undef ;
my $spec = undef ;
my $wiki_title = undef ;
my $dir = undef;
my $theSpecFragment = "%code%";
my $preserveWiki = "";
my $fake = 0;

my $result = GetOptions(
    "f|file=s"   => \$file,
    "p=s" => \$preserveWiki,
    "w|wiki=s"   => \$wiki_title,
    "s|spec=s"   => \$spec,
    "f|fake"    => \$fake,
    "d|dir=s"   => \$dir) || usage();

my $wiki_config = {
  "api_url" => "https://www.w3.org/wiki/api.php"
};

my $io ;
our $theSpecURL = "";

if ($spec) {
  print "Processing spec $spec\n";
  $wiki_title = $specs{$spec}->{title};
  $theSpecURL = $specs{$spec}->{specURL};
  if (!$dir) {
    $dir = "../" . $specs{$spec}->{dir};
  }
  $theSpecFragment = $specs{$spec}->{fragment};
}

if (!$dir) {
  $dir = "../raw";
}

if (!-d $dir) {
  print STDERR "No such directory: $dir\n";
  exit 1;
}

if ($file) {
  open($io, "<", $file) || die("Failed to open $file: " . $@);
} elsif ($wiki_title) {
  my $MW = MediaWiki::API->new( $wiki_config );

  $MW->{config}->{on_error} = \&on_error;

  sub on_error {
    print "Error code: " . $MW->{error}->{code} . "\n";
    print $MW->{error}->{stacktrace}."\n";
    die;
  }
  my $page = $MW->get_page( { title => $wiki_title } );
  my $theContent = $page->{'*'};
  print "Loaded " . length($theContent) . " from $wiki_title\n";
  if ($preserveWiki) {
    if (open(OUTPUT, ">$preserveWiki")) {
      print OUTPUT $theContent;
      close OUTPUT;
      print "Wiki preserved in $preserveWiki\n";
      exit 0;
    } else {
      print "Failed to create $preserveWiki. Terminating.\n";
      exit 1;
    }
  }
  $io = IO::String->new($theContent);
} else {
  usage() ;
}



# Now let's walk through the content and build a test page for every item
#

# iterate over the content

# my $io ;
# open($io, "<", "raw") ;

# data structure:
#
# steps is a list of steps to be performed.
# Each step is an object that has a type property and other properties based upon that type.
#
# Types include:
#
# 'test' - has a property for each ATAPI for which there are tests
# 'attribute' - has a property for the target id, attribute name, and value
# 'event' - has a property for the target id and event name
my $state = 0;   # between items
my $theStep = undef;
my $current = "";
my $theCode = "";
my $theAttributes = {};
my @steps ;
my $theAsserts = {} ;
my $theAssertCount = 0;
my $theAPI = "";
my $typeRows = 0;
my $theType = "";
my $theName = "";
my $theRef = "";
my $lineCounter = 0;
my $skipping = 0;

our $testNames = {} ;

while (<$io>) {
  if (m/<!-- END OF TESTS -->/) {
    last;
  }
  $lineCounter++;
  # look for state
  if (m/^SpecURL: (.*)$/) {
    $theSpecURL = $1;
    $theSpecURL =~ s/^ *//;
    $theSpecURL =~ s/ *$//;
  }
  if ($state == 5 && m/^; \/\/ (.*)/) {
    # we found another test inside a block
    # we were in an item; dump it
    build_test($current, $theAttributes, $theCode, \@steps, $theSpecFragment) ;
    # print "Finished $current and new subblock $1\n";
    $state = 1;
    $theAttributes = {} ;
    $theAPI = "";
    @steps = ();
    $theCode = "";
    $theAsserts = undef;
    $theName = "";
  } elsif (m/^=== +(.*[^ ]) +===/) {
    if ($state != 0) {
      if ($skipping) {
        print STDERR "Flag on assertion $current; skipping\n";
      } else {
        # we were in an item; dump it
        build_test($current, $theAttributes, $theCode, \@steps, $theSpecFragment) ;
        # print "Finished $current\n";
      }
    }
    $state = 1;
    $current = $1;
    $theAttributes = {} ;
    @steps = ();
    $theCode = "";
    $theAsserts = undef;
    $theAPI = "";
    $theName = "";
    if ($current =~ m/\(/) {
      # there is a paren in the name -skip it
      $skipping = 1;
    } else {
      $skipping = 0;
    }
  }

  if ($state == 1) {
    if (m/<pre>/) {
      # we are now in the code block
      $state = 2;
      next;
    } elsif (m/==== +(.*) +====/) {
      # we are in some other block
      $theName = lc($1);
      $theAttributes->{$theName} = "";
      next;
    }
    if (m/^Reference: +(.*)$/) {
      $theAttributes->{reference} = $theSpecURL . "#" . $1;
    } elsif ($theName ne "") {
      # accumulate whatever was in the block under the data for it
      chomp();
      $theAttributes->{$theName} .= $_;
    } elsif (m/TODO/) {
      $state = 0;
    }
  }

  if ($state == 2) {
    if (m/<\/pre>/) {
      # we are done with the code block
      $state = 3;
    } else  {
      if (m/^\s/ && !m/if given/) {
        # trim any trailing whitespace
        $theCode =~ s/ +$//;
        $theCode =~ s/\t/ /g;
        $theCode .= $_;
        # In MediaWiki, to display & symbol escapes as literal text, one
        # must use "&amp;&" for the "&" character. We need to undo that.
        $theCode =~ s/&amp;(\S)/&$1/g;
      }
    }
  } elsif ($state == 3) {
    # look for a table
    if (m/^\{\|/) {
      # table started
      $state = 4;
    }
  } elsif ($state == 4) {
    if (m/^\|-/) {
      if ($theAPI
        && exists($theAsserts->{$theAPI}->[$theAssertCount])
        && scalar(@{$theAsserts->{$theAPI}->[$theAssertCount]})) {
        $theAssertCount++;
      }
      # start of a table row
      if ($theType ne "" && $typeRows) {
        # print qq($theType typeRows was $typeRows\n);
        # we are still processing items for a type
        $typeRows--;
        # populate the first cell
        $theAsserts->{$theAPI}->[$theAssertCount] = [ $theType ] ;
      } else {
        $theType = "";
      }
    } elsif (m/^\|\}/) {
      # ran out of table
      $state = 5;
    # adding processing for additional block types
    # a colspan followed by a keyword triggers a start
    # so |colspan=5|element triggers a new collection
    # |colspan=5|attribute triggers the setting of an attribute
    } elsif (m/^\|colspan="*([0-9])"*\|([^ ]+) (.*)$/) {
      my $type = $2;
      my $params = $3;

      my $obj = {} ;
      if ($type eq "attribute") {
        if ($params =~ m/([^:]+):([^ ]+) +(.*)$/) {
          $obj = {
            type => $type,
            element => $1,
            attribute => $2,
            value => $3
          };
          $theStep = undef;
          push(@steps, $obj);
        } else {
          print STDERR "Malformed attribute instruction at line $lineCounter: " . $_ . "\n";
        }
      } elsif ($type eq "event") {
        if ($params =~ m/([^:]+):([^ ]+).*$/) {
          $obj = {
            type => $type,
            element => $1,
            value => $2
          };
          $theStep = undef;
          push(@steps, $obj);
        } else {
          print STDERR "Malformed event instruction at line $lineCounter: " . $_ . "\n";
        }
      } elsif ($type eq "element") {
        $obj = {
          type => "test",
          element => $3
        };
        push(@steps, $obj);
        $theStep = scalar(@steps) - 1;
        $theAsserts = $steps[$theStep];
      } else {
        print STDERR "Unknown operation type: $type at line " . $lineCounter . "; skipping.\n";
      }
    } elsif (m/($apiNamesRegex)$/) {
      my $theString = $1;
      $theString =~ s/ +$//;
      $theString =~ s/^ +//;
      if ($theString eq "IA2") {
        $theString = "IAccessible2" ;
      }
      my $rows = 1;
      if (m/^\|rowspan="*([0-9])"*\|(.*)$/) {
        $rows = $1
      }
      if (grep { $_ eq $theString } @apiNames) {
        # we found an API name - were we already processing assertions?
        if (!$theAsserts) {
          # nope - now what?
          $theAsserts = {
            type => "test",
            element => "test"
          };
          push(@steps, $theAsserts);
        }
        $theAssertCount = 0;
        # this is a new API section
        $theAPI = $theString ;
        $theAsserts->{$theAPI} = [ [] ] ;
        $theType = "";
      } else {
        # this is a multi-row type
        $theType = $theString;
        $typeRows = $rows;
        # print qq(Found multi-row $theString for $theAPI with $typeRows rows\n);
        $typeRows--;
        # populate the first cell
        if ($theAPI
          && exists($theAsserts->{$theAPI}->[$theAssertCount])
          && scalar(@{$theAsserts->{$theAPI}->[$theAssertCount]})) {
          $theAssertCount++;
        }
        $theAsserts->{$theAPI}->[$theAssertCount] = [ $theType ] ;
      }
    } elsif (m/^\|(.*)$/) {
      my $item = $1;
      $item =~ s/^ *//;
      $item =~ s/ *$//;
      $item =~ s/^['"]//;
      $item =~ s/['"]$//;
      # add into the data structure for the API
      if (!exists $theAsserts->{$theAPI}->[$theAssertCount]) {
        $theAsserts->{$theAPI}->[$theAssertCount] = [ $item ] ;
      } else {
        push(@{$theAsserts->{$theAPI}->[$theAssertCount]}, $item);
      }
    }
  }
};

if ($state != 0) {
  build_test($current, $theAttributes, $theCode, \@steps, $theSpecFragment) ;
  print "Finished $current\n";
}

exit 0;

# build_test
#
# create a test file
#
# attempts to create unique test names

sub build_test() {
  my $title = shift ;
  my $attrs = shift ;
  my $code = shift ;
  my $steps = shift;
  my $frag = shift ;

  if ($title eq "") {
    print STDERR "No name provided!";
    return;
  }

  if ($frag ne "") {
    $frag =~ s/%code%/$code/;
    $code = $frag;
  }

  $code =~ s/ +$//m;
  $code =~ s/\t/ /g;

  my $title_reference = $title;

  if ($code eq "") {
    print STDERR "No code for $title; skipping.\n";
    return;
  }
  if ( $steps eq {}) {
    print STDERR "No assertions for $title; skipping.\n";
    return;
  }

  my $testDef =
  { "title" => $title,
    "steps" => []
  };
  my $stepCount = 0;
  foreach my $asserts (@$steps) {
    $stepCount++;
    my $step =
      {
        "type" => $asserts->{"type"},
        "title"=> "step " . $stepCount,
      };

    if ($asserts->{type} eq "test") {
      # everything in the block is about testing an element
      $step->{"element"} = ( $asserts->{"element"} || "test" );

      my $tests = {};
      if ($fake) {
        $tests->{"WAIFAKE"} = [ [ "property", "role", "is", "ROLE_TABLE_CELL" ], [ "property", "interfaces", "contains", "TableCell" ] ];
      }
      foreach my $name (@apiNames) {
        if (exists $asserts->{$name} && scalar(@{$asserts->{$name}})) {
          $tests->{$name} = $asserts->{$name};
        }
      };

      $step->{test} = $tests;

    } elsif ($asserts->{type} eq "attribute") {
      $step->{type} = "attribute";
      $step->{element} = $asserts->{"element"};
      $step->{attribute} = $asserts->{"attribute"};
      $step->{value} = $asserts->{value};
    } elsif ($asserts->{type} eq "event") {
      $step->{type} = "event";
      $step->{element} = $asserts->{"element"};
      $step->{event} = $asserts->{value};
    } else {
      print STDERR "Invalid step type: " . $asserts->{type} . "\n";
      next;
    }
    push(@{$testDef->{steps}}, $step);
  }


  # populate the rest of the test definition

  if (scalar(keys(%$attrs))) {
    while (my $key = each(%$attrs)) {
      # print "Copying $key \n";
      $testDef->{$key} = $attrs->{$key};
    }
  }

  if (exists $attrs->{reference}) {
    $title_reference = "<a href='" . $attrs->{reference} . "'>" . $title_reference . "</a>" ;
  }

  my $testDef_json = to_json($testDef, { canonical => 1, pretty => 1, utf8 => 1});

  my $fileName = $title;
  $fileName =~ s/\s*$//;
  $fileName =~ s/\///g;
  $fileName =~ s/\s+/_/g;
  $fileName =~ s/[,=:]/_/g;
  $fileName =~ s/['"]//g;

  my $count = 2;
  if ($testNames->{$fileName}) {
    while (exists $testNames->{$fileName . "_$count"}) {
      $count++;
    }
    $fileName .= "_$count";
  }

  $fileName = lc($fileName);

  $testNames->{$fileName} = 1;

  $fileName .= $theSuffix;

  my $template = qq(<!doctype html>
<html>
  <head>
    <title>$title</title>
    <meta content="text/html; charset=utf-8" http-equiv="Content-Type"/>
    <link rel="stylesheet" href="/wai-aria/scripts/manual.css">
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
    <script src="/wai-aria/scripts/ATTAcomm.js"></script>
    <script>
    setup({explicit_timeout: true, explicit_done: true });

    var theTest = new ATTAcomm(
    $testDef_json
    ) ;
    </script>
  </head>
  <body>
  <p>This test examines the ARIA properties for $title_reference.</p>
  $code
  <div id="manualMode"></div>
  <div id="log"></div>
  <div id="ATTAmessages"></div>
  </body>
</html>);

  my $file ;

  if (open($file, ">", "$dir/$fileName")) {
    print $file $template;
    print $file "\n";
    close $file;
  } else {
    print STDERR qq(Failed to create file "$dir/$fileName" $!\n);
  }

  return;
}

sub usage() {
  print STDERR q(usage: make_tests.pl -f file | -w wiki_title | -s spec [-n -v -d dir ]

  -s specname   - the name of a spec known to the system
  -w wiki_title - the TITLE of a wiki page with testable statements
  -f file       - the file from which to read

  -n            - do nothing
  -v            - be verbose
  -d dir        - put generated tests in directory dir
  );
  exit 1;
}

# vim: ts=2 sw=2 ai:
