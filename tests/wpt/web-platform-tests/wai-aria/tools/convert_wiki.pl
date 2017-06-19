#!/usr/bin/perl
#
#  convert_wiki.pl - Transform an old-style wiki into the new format
#
#  This script assumes that a wiki has testable statement entries
#  with varying lemgth lines. Those lines will be converted into
#  the format described by the specification at
#  https://spec-ops.github.io/atta-api/index.html
#
#  usage: convert_wiki.pl -f file | -w wiki_title -o outFile

use strict;

use IO::String ;
use JSON ;
use MediaWiki::API ;
use Getopt::Long;

my @apiNames = qw(UIA MSAA ATK IAccessible2 AXAPI);

# dir is determined based upon the short name of the spec and is defined
# by the input or on the command line

my $file = undef ;
my $spec = undef ;
my $wiki_title = undef ;
my $dir = undef;
my $outFile = undef;

my $result = GetOptions(
    "f|file=s"   => \$file,
    "w|wiki=s"   => \$wiki_title,
    "s|spec=s"   => \$spec,
    "o|output=s"   => \$outFile);

my $wiki_config = {
  "api_url" => "https://www.w3.org/wiki/api.php"
};

my %specs = (
    "aria11" => {
      title => "ARIA_1.1_Testable_Statements",
      specURL => "https://www.w3.org/TR/wai-aria11"
    },
    "svg" => {
      title => "SVG_Accessibility/Testing/Test_Assertions_with_Tables_for_ATTA",
      specURL => "https://www.w3.org/TR/svg-aam-1.0/"
    }
);

my $io ;
our $theSpecURL = "";

if ($spec) {
  $wiki_title = $specs{$spec}->{title};
  $theSpecURL = $specs{$spec}->{specURL};
}

if ($wiki_title) {
  my $MW = MediaWiki::API->new( $wiki_config );
  my $page = $MW->get_page( { title => $wiki_title } );
  my $theContent = $page->{'*'};
  $io = IO::String->new($theContent);
} elsif ($file) {
  open($io, "<", $file) || die("Failed to open $file: " . $@);
} else {
  usage() ;
}

my $outH ;
if (defined $outFile) {
  open($outH, ">", $outFile) || die("Failed to create file $outFile: $@");
} else {
  $outH = new IO::Handle;
  $outH->fdopen(fileno(STDOUT), "w");
}


# Now let's walk through the content and spit it back out
# transformed
#

# iterate over the content

# my $io ;
# open($io, "<", "raw") ;

my $state = 0;   # between items
my $theCode = "";
my $theAttributes = {};
my $theAsserts = {} ;
my $theAssertCount = 0;
my $theAPI = "";
my $typeRows = 0;
my $theType = "";
my $theName = "";
my $theRef = "";

my $before = "" ;
my $after = "" ;

my @errors = () ;
my $linecount = 0;

while (<$io>) {
  $linecount++;
  # look for state
  if ($state == 0) {
    if (scalar(keys(%$theAsserts))) {
      # we were in an item; dump it
      print $outH  dump_table($theAsserts) ;
      $theAsserts = {};
    }
    print $outH $_;
  }
  if (m/^\{\|/) {
    # table started
    $state = 4;
    $theAPI = "";
  }
  if ($state == 4) {
    if (m/^\|-/) {
      if ($theAPI
          && exists($theAsserts->{$theAPI}->[$theAssertCount])
          && scalar(@{$theAsserts->{$theAPI}->[$theAssertCount]})) {
        $theAssertCount++;
      }
      # start of a table row
      if ($theType ne "" && $typeRows) {
        # we are still processing items for a type
        $typeRows--;
        # populate the first cell
        $theAsserts->{$theAPI}->[$theAssertCount] = [ $theType ] ;
      } else {
        $theType = "";
      }
    } elsif (m/^\|\}/) {
      # ran out of table
      $state = 0;
    } elsif (m/^\|rowspan="*([0-9])"*\|(.*)$/) {
      # someone put a rowspan in here..  is ht an API?
      my $rows = $1;
      my $theString = $2;
      $theString =~ s/ +$//;
      $theString =~ s/^ +//;
      $theString = "IAccessible2" if ($theString eq "IA2") ;
      if (grep { $_ eq $theString } @apiNames) {
        $theAssertCount = 0;
        # this is a new API section
        $theAPI = $theString ;
        $theAsserts->{$theAPI} = [ [] ] ;
        $theType = "";
      } else {
        # nope, this is a multi-row type
        $theType = $theString;
        $typeRows = $rows;
        $typeRows--;
        # populate the first cell
        if ($theAPI
            && exists($theAsserts->{$theAPI}->[$theAssertCount])
            && scalar(@{$theAsserts->{$theAPI}->[$theAssertCount]})) {
          $theAssertCount++;
        }
        $theAsserts->{$theAPI}->[$theAssertCount] = [ $theType ] ;
      }
    } elsif (m/^\|note/) {
      # there is a note in the table...  throw it out
      # and the next line too
      my $l = <$io>;
    } elsif (m/^\|(MSAA|UIA|IA2|IAccessible2|ATK|AXAPI) *$/) {
      # they FORGOT a rowspan on an API.  That means there is only 1
      # row
      my $theString = $1;
      $theString =~ s/ +$//;
      $theString =~ s/^ +//;
      $theString = "IAccessible2" if ($theString eq "IA2") ;
      if (grep { $_ eq $theString } @apiNames) {
        $theAssertCount = 0;
        # this is a new API section
        $theAPI = $theString ;
        $theAsserts->{$theAPI} = [ [] ] ;
        $theType = "";
      } else {
        push(@errors, "Bad API Name at $linecount: $theString");
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
    next;
  }
};

if ($state == 0) {
  if (scalar(keys(%$theAsserts))) {
    # we were in an item; dump it
    print $outH  dump_table($theAsserts) ;
  }
}

if (@errors) {
  print "There were the following errors:\n";
  foreach my $err (@errors) {
    print $err . "\n";
  }
}

exit 0;


sub dump_table() {
  my $asserts = shift;

  if (!scalar(keys(%$asserts)) )  {
    # no actual assertions
    return "";
  }

  my $output = "" ;

  my @keywords = qw(property result event);

  foreach my $API (sort(keys(%$asserts))) {
    # looking at each API in turn
    my $ref = $asserts->{$API};
    my $rowcount = scalar(@$ref) ;
    # $output .= "|-\n|rowspan=$count|$API\n" ;
    # now we are in the assertions; special case each API
    my @conditions = @$ref;
    for (my $i = 0; $i < scalar(@conditions); $i++) {
      my (@new, @additional) ;
      if ($i) {
        $output .= "|-\n";
      }
      if ($API eq "ATK") {
        my $start = 0;
        my $assert = "is";
        if ($conditions[$i]->[0] =~ m/^NOT/) {
          $start = 1;
          $assert = "isNot";
        }

        if ($conditions[$i]->[$start] =~ m/^ROLE_/) {
          $new[0] = "property";
          $new[1] = "role";
          $new[2] = $assert;
          $new[3] = $conditions[$i]->[$start];
        } elsif ($conditions[$i]->[$start] =~ m/^xml-roles/) {
          $new[0] = "property";
          $new[1] = "role";
          $new[2] = $assert;
          $new[3] = $conditions[$i]->[$start+1];
        } elsif ($conditions[$i]->[$start] =~ m/^description/) {
          my $id = $conditions[$i]->[$start+1];
          $new[0] = "property";
          $new[1] = "description";
          $new[2] = $assert;
          $new[3] = $id;
          push(@{$additional[0]}, ("relation", "RELATION_DESCRIBED_BY", $assert, $id));
          push(@{$additional[1]}, ("relation", "RELATION_DESCRIPTION_FOR", $assert, "test"));
        } elsif ($conditions[$i]->[$start] =~ m/not in accessibility tree/i) {
          @new = qw(property accessible exists false);
        } elsif ($conditions[$i]->[$start] =~ m/^RELATION/) {
          $new[0] = "relation";
          $new[1] = $conditions[$i]->[$start];
          $new[2] = $assert;
          $new[3] = $conditions[$i]->[$start+1];
        } elsif ($conditions[$i]->[$start] =~ m/(.*) interface/i) {
          $new[0] = "property";
          $new[1] = "interfaces";
          $new[3] = $1;
          if ($conditions[$i]->[$start+1] ne '<shown>'
            && $conditions[$i]->[$start+1] !~ m/true/i ) {
            $assert = "doesNotContain";
          } else {
            $assert = "contains";
          }
          $new[2] = $assert;
        } elsif ($conditions[$i]->[$start] eq "object" || $conditions[$i]->[$start] eq "attribute" ) {
          $new[0] = "property";
          $new[1] = "objectAttributes";
          my $val = $conditions[$i]->[2];
          $val =~ s/"//g;
          $new[3] = $conditions[$i]->[1] . ":" . $val;
          if ($conditions[$i]->[1] eq "not exposed"
            || $conditions[$i]->[2] eq "false") {
            $new[2] = "doesNotContain";
          } else {
            $new[2] = "contains";
          }
        } elsif ($conditions[$i]->[$start] =~ m/^STATE_/) {
          $new[0] = "property";
          $new[1] = "states";
          $new[3] = $conditions[$i]->[$start];
          if ($assert eq "true") {
            $new[2] = "contains";
          } else {
            $new[2] = "doesNotContain";
          }
        } elsif ($conditions[$i]->[$start] =~ m/^object attribute (.*)/) {
          my $name = $1;
          $new[0] = "property";
          $new[0] = "objectAttributes";
          my $val = $conditions[$i]->[1];
          $val =~ s/"//g;
          if ($val eq "not exposed" || $val eq "not mapped") {
            $new[3] = $name;
            $new[2] = "doesNotContain";
          } else {
            $new[3] = $name . ":" . $val;
            $new[2] = "contains";
          }
        } elsif ($conditions[$i]->[$start] =~ m/^name/) {
          my $name = $conditions[$i]->[1];
          my $cond = "is" ;
          if ($name eq "<empty>" ) {
            $cond = "empty";
            $name = "true"
          } elsif ($name eq "<not empty>") {
            $cond = "empty";
            $name = "false";
          }
          $new[0] = "property";
          $new[1] = "name";
          $new[2] = $cond;
          $new[3] = $name;
        } else {
          @new = @{$conditions[$i]};
          if ($conditions[$i]->[2] eq '<shown>') {
            $new[2] = "contains";
          }
        }
        $conditions[$i] = \@new;
      } elsif ($API eq "UIA") {
        my $start = 0;
        my $assert = "is";
        if ($conditions[$i]->[$start] =~ m/\./) {
          my $val = $conditions[$i]->[$start+1];
          $val =~ s/"//g;
          $val =~ s/'//g;
          $new[0] = "result";
          $new[1] = $conditions[$i]->[$start];
          $new[2] = $assert;
          $new[3] = $conditions[$i]->[$start+1];
        } elsif ($conditions[$i]->[$start] =~ m/not in accessibility tree/i) {
          @new = qw(property accessible exists false);
        } elsif ($conditions[$i]->[$start] =~ m/^(AriaProperties|Toggle|ExpandCollapse)/) {
          my $name = $conditions[$i]->[1];
          $new[0] = "property";
          $new[1] = $1;
          my $val = $conditions[$i]->[2];
          $val =~ s/"//g;
          if ($val eq "not exposed" || $val eq "not mapped") {
            $new[3] = $name;
            $new[2] = "doesNotContain";
          } else {
            $new[3] = $name . ":" . $val;
            $new[2] = "contains";
          }
        } elsif ($conditions[$i]->[$start] =~ m/^LabeledBy/i) {
          $new[0] = "property";
          $new[1] = $conditions[$i]->[$start];
          $new[2] = $assert;
          $new[3] = $conditions[$i]->[$start+1];
        } elsif ($conditions[$i]->[$start] =~ m/^Name/) {
          my $name = $conditions[$i]->[1];
          my $cond = "is" ;
          if ($name eq "<empty>" ) {
            $cond = "empty";
            $name = "true"
          } elsif ($name eq "<not empty>") {
            $cond = "empty";
            $name = "false";
          }
          $new[0] = "property";
          $new[1] = "Name";
          $new[2] = $cond;
          $new[3] = $name;
        } elsif ($conditions[$i]->[$start] =~ m/^TBD/) {
          $new[0] = "TBD";
          $new[1] = $new[2] = $new[3] = "";
        } else {
          if ($conditions[$i]->[1] ne '<shown>'
            && $conditions[$i]->[1] !~ m/true/i ) {
            $assert = "isNot";
          } else {
            $assert = "is";
          }
          $new[0] = "property";
          $new[1] = $conditions[$i]->[$start];
          $new[2] = $assert;
          $new[3] = $conditions[$i]->[$start+1];
        }
      } elsif ($API eq "MSAA") {
        my $start = 0;
        my $assert = "is";
        if ($conditions[$i]->[0] =~ m/^NOT/) {
          $start = 1;
          $assert = "isNot";
        }

        if ($conditions[$i]->[$start] =~ m/^role/) {
          $new[0] = "property";
          $new[1] = "role";
          $new[2] = $assert;
          $new[3] = $conditions[$i]->[$start+1];
        } elsif ($conditions[$i]->[$start] =~ m/^xml-roles/) {
          $new[0] = "property";
          $new[1] = "role";
          $new[2] = $assert;
          $new[3] = $conditions[$i]->[$start+1];
        } elsif ($conditions[$i]->[$start] =~ m/not in accessibility tree/i) {
          @new = qw(property accessible exists false);
        } elsif ($conditions[$i]->[$start] =~ m/^(accName|accDescription)/) {
          my $name = $conditions[$i]->[$start+1];
          my $cond = "is" ;
          if ($name eq "<empty>" ) {
            $cond = "empty";
            $name = "true"
          } elsif ($name eq "<not empty>") {
            $cond = "empty";
            $name = "false";
          }
          $new[0] = "property";
          $new[1] = $conditions[$i]->[$start];
          $new[2] = $cond;
          $new[3] = $name;
        } elsif ($conditions[$i]->[$start] =~ m/^ROLE_/) {
          $new[0] = "property";
          $new[1] = "role";
          $new[2] = $assert;
          $new[3] = $conditions[$i]->[$start];
        } elsif ($conditions[$i]->[$start] =~ m/^(STATE_.*) *([^ ]*)/) {
          $new[0] = "property";
          $new[1] = "states";
          $new[3] = $1;
          if ($2 && $2 eq "cleared") {
            print "MATCHED $1, $2\n";
            $new[2] = "doesNotContain";
          } else {
            $new[2] = "contains";
          }
        } elsif ($conditions[$i]->[$start] =~ m/^TBD/) {
          $new[0] = "TBD";
          $new[1] = $new[2] = $new[3] = "";
        }
      } elsif ($API eq "IAccessible2") {
        my $start = 0;
        my $assert = "is";
        if ($conditions[$i]->[0] =~ m/^NOT/) {
          $start = 1;
          $assert = "isNot";
        }
        if ($conditions[$i]->[$start] =~ m/^IA2_ROLE_/) {
          $new[0] = "property";
          $new[1] = "role";
          $new[2] = $assert;
          $new[3] = $conditions[$i]->[$start];
        } elsif ($conditions[$i]->[$start] =~ m/not in accessibility tree/i) {
          @new = qw(property accessible exists false);
        } elsif ($conditions[$i]->[$start] =~ m/^IA2_RELATION_/) {
          $new[0] = "relation";
          $new[1] = $conditions[$i]->[$start];
          $new[2] = $assert;
          $new[3] = $conditions[$i]->[$start+1];
        } elsif ($conditions[$i]->[$start] =~ m/^IA2_STATE_/) {
          $new[0] = "property";
          $new[1] = "states";
          $new[3] = $conditions[$i]->[$start];
          if ($assert eq "true") {
            $new[2] = "contains";
          } else {
            $new[2] = "doesNotContain";
          }
        } elsif ($conditions[$i]->[$start] =~ m/^IA2_/) {
          $new[0] = "property";
          $new[1] = "states";
          $new[3] = $conditions[$i]->[$start];
          if ($assert eq "true") {
            $new[2] = "contains";
          } else {
            $new[2] = "doesNotContain";
          }
        } elsif ($conditions[$i]->[$start] =~ m/(IAccessibleTable2)/i) {
          $new[0] = "property";
          $new[1] = "interfaces";
          $new[3] = $1;
          if ($conditions[$i]->[$start+1] ne '<shown>'
            && $conditions[$i]->[$start+1] !~ m/true/i ) {
            $assert = "doesNotContain";
          } else {
            $assert = "contains";
          }
          $new[2] = $assert;
        } elsif ($conditions[$i]->[$start] =~ m/(.*) interface/i) {
          $new[0] = "property";
          $new[1] = "interfaces";
          $new[3] = $1;
          if ($conditions[$i]->[$start+1] ne '<shown>'
            && $conditions[$i]->[$start+1] !~ m/true/i ) {
            $assert = "doesNotContain";
          } else {
            $assert = "contains";
          }
          $new[2] = $assert;
        } elsif ($conditions[$i]->[$start] =~ m/(.*)\(\)/) {
          $new[0] = "result";
          $new[1] = $conditions[$i]->[$start];
          my $val = $conditions[$i]->[2];
          $val =~ s/"//g;
          $new[3] = $conditions[$i]->[1] . ":" . $val;
          if ($conditions[$i]->[1] eq "not exposed"
            || $conditions[$i]->[2] eq "false") {
            $new[2] = "doesNotContain";
          } else {
            $new[2] = "contains";
          }
        } elsif ($conditions[$i]->[$start] =~ m/(.*localizedExtendedRole)/) {
          $new[0] = "result";
          $new[1] = $conditions[$i]->[$start];
          my $val = $conditions[$i]->[2];
          $val =~ s/"//g;
          $new[3] = $conditions[$i]->[1] . ":" . $val;
          if ($conditions[$i]->[1] eq "not exposed"
            || $conditions[$i]->[2] eq "false") {
            $new[2] = "doesNotContain";
          } else {
            $new[2] = "contains";
          }
        } elsif ($conditions[$i]->[$start] eq "object" || $conditions[$i]->[$start] eq "attribute" ) {
          $new[0] = "property";
          $new[1] = "objectAttributes";
          my $val = $conditions[$i]->[2];
          $val =~ s/"//g;
          $new[3] = $conditions[$i]->[1] . ":" . $val;
          if ($conditions[$i]->[1] eq "not exposed"
            || $conditions[$i]->[2] eq "false") {
            $new[2] = "doesNotContain";
          } else {
            $new[2] = "contains";
          }
        } elsif ($conditions[$i]->[$start] =~ m/^object attribute (.*)/) {
          my $name = $1;
          $new[0] = "property";
          $new[1] = "objectAttributes";
          my $val = $conditions[$i]->[1];
          $val =~ s/"//g;
          if ($val eq "not exposed" || $val eq "not mapped") {
            $new[3] = $name;
            $new[2] = "doesNotContain";
          } else {
            $new[3] = $name . ":" . $val;
            $new[2] = "contains";
          }
        } else {
          @new = @{$conditions[$i]};
          if ($conditions[$i]->[2] eq '<shown>') {
            $new[2] = "contains";
          }
        }
        $conditions[$i] = \@new;
      } elsif ($API eq "AXAPI") {
        my $start = 0;
        my $assert = "is";
        if ($conditions[$i]->[0] =~ m/^NOT/) {
          $start = 1;
          $assert = "isNot";
        }
        if ($conditions[$i]->[$start] =~ m/^AXElementBusy/) {
          if ($conditions[$i]->[$start+1] =~ m/yes/i) {
            $new[3] = "true";
          } else {
            $new[3] = "false";
          }
          $new[0] = "property";
          $new[1] = $conditions[$i]->[$start];
          $new[2] = $assert;
        } elsif ($conditions[$i]->[$start] =~ m/not in accessibility tree/i) {
          @new = qw(property accessible exists false);
        } elsif ($conditions[$i]->[$start] =~ m/^AX/) {
          $new[0] = "property";
          $new[1] = $conditions[$i]->[$start];
          $new[2] = $assert;
          $new[3] = $conditions[$i]->[$start+1];
        } elsif ($conditions[$i]->[$start] =~ m/^TBD/) {
          $new[0] = "TBD";
          $new[1] = $new[2] = $new[3] = "";
        } else {
          if ($conditions[$i]->[1] ne '<shown>'
            && $conditions[$i]->[1] !~ m/true/i ) {
            $assert = "isNot";
          } else {
            $assert = "is";
          }
          $new[0] = "result";
          $new[1] = $conditions[$i]->[0];
          $new[2] = $assert;
          $new[3] = "true";
        }
      }
      if ($i == 0) {
        if (scalar(@additional)) {
          $rowcount += scalar(@additional);
        }
        $output .= "|-\n|rowspan=$rowcount|$API\n";
      }
      foreach my $row (@new) {
        $output .= "|$row\n";
      }
      if (scalar(@additional)) {
        foreach my $arow (@additional) {
          $output .= "|-\n" ;
          foreach my $aItem (@$arow) {
            $output .= "|$aItem\n";
          }
        }
      }
    }
  }
  $output .= "|}\n";

  return $output;
}

sub usage() {
  print STDERR q(usage: make_tests.pl -f file | -w wiki_title | -s spec [-n -v -d dir ]

  -s specname   - the name of a spec known to the system
  -w wiki_title - the TITLE of a wiki page with testable statements
  -f file       - the file from which to read
  -o outFile    - the file to fill with  the converted wiki; defaults to STDOUT

  -n            - do nothing
  -v            - be verbose
  );
  exit 1;
}

# vim: ts=2 sw=2 ai:
