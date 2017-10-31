#!/usr/bin/perl -w
##############################################################################
# W3C Test Suite Generator                                                   #
##############################################################################
package main;
use strict;
use diagnostics;
use XML::Parser; # DEPENDENCY
use lib '.';
use utils::parser;
use utils::helpers;
use utils::generators;

# check arguments
# if argument 1 is '-v' then print out the value of second argument, which will be one of:
#  - DESTINATION_TYPES
#  - SHELL_TYPES
#  - TEST_TYPES

if (scalar(@ARGV) == 1 and ($ARGV[0] eq '-h' or $ARGV[0] eq '--help')) {
    print "Syntax: generateTests.pl -v VARIABLE_NAME or generateTests.pl test1.xml test2,xml ...\n";
    return 0;
} elsif (scalar(@ARGV) > 0 and $ARGV[0] eq '-v') {
    if (scalar(@ARGV) == 2) {
        print $utils::helpers::types{$ARGV[1]};
        exit 0;
    } else {
        my @vars = keys(%utils::helpers::types);
        local $" = '\', \'';
        print "You must specify which variable to display in the form '-v VARIABLE_NAME',\nwhere VARIABLE_NAME is one of '@vars'.\n";
        exit 1;
    }
}

# otherwise, process arguments as filenames:
my %cache = %{&utils::helpers::readCache()};
while (scalar(@ARGV)) {
    # read file
    local $/ = undef;
    my $file = <>;
    close(ARGV);
    # print status
    my $filename = $ARGV;
    $filename =~ s/\.[a-z]+$//o; # remove extension
    print "parsing $filename...\n";
    # process file
    $cache{$filename} = XML::Parser->new(Style => 'utils::parser', Namespaces => 1, ErrorContext => 1)->parse($file);
    die "$filename: modulename/number attributes wrong ('$cache{$filename}->{modulename}-$cache{$filename}->{number}')\n" if $filename ne "$cache{$filename}->{modulename}-$cache{$filename}->{number}";
}
&utils::helpers::writeCache(\%cache);

print "generating tests...\n";
# ...and generate the tests
&utils::generators::generateTopIndex(\%cache); # points to mini test index and all test type indexes
foreach my $destinationType (split ' ', $utils::helpers::types{'DESTINATION_TYPES'}) {
    my @destinationTests = &utils::helpers::shortlistTestsForDestination($destinationType,
                                                                         [ sort {
                                                                             my $na = $cache{$a}->{'number'};
                                                                             my $nb = $cache{$b}->{'number'};
                                                                             for my $n ($na, $nb) {
                                                                                 $n =~ m/^([0-9]*(?:\.[0-9]+)?)/o;
                                                                                 $n = $1;
                                                                             }
                                                                             if (($na ne '') and ($nb ne '')) {
                                                                                 return (($na <=> $nb) or ($cache{$a}->{'number'} cmp $cache{$b}->{'number'}) or ($a cmp $b));
                                                                             } else {
                                                                                 return (($cache{$a}->{'number'} cmp $cache{$b}->{'number'}) or ($a cmp $b));
                                                                             }
                                                                         } keys(%cache) ], \%cache);
    # generate primary index
    &utils::generators::generateSubIndex($destinationType, \@destinationTests, \%cache); # points to mini test index and all test type indexes
    # generate complete mini test index
    &utils::generators::generateMiniTestIndex($destinationType, \@destinationTests, \%cache); # points to all mini tests
    # generate mini tests
    foreach my $testIndex (0..$#destinationTests) {
        # generate mini test and CSS if needed
        &utils::generators::generateMiniTest($destinationType, \@destinationTests, \%cache, $testIndex);
    }
    # generate flat tests and shells
    foreach my $testType (split ' ', $utils::helpers::types{'TEST_TYPES'}) {
        my @finalTestList = &utils::helpers::shortlistTestsForTypes($testType, \@destinationTests, \%cache);
        # generate test type index
        &utils::generators::generateTestTypeIndex($destinationType, $testType, \@finalTestList, \%cache); # points to flat test index and each shell index
        # generate flat test index
        &utils::generators::generateFlatTestIndex($destinationType, $testType, \@finalTestList, \%cache); # points to flat tests
        foreach my $shell (split ' ', $utils::helpers::types{'SHELL_TYPES'}) {
            # generate shell index
            &utils::generators::generateShellTestIndex($destinationType, $testType, $shell, \@finalTestList, \%cache); # points to shell tests
        }
        foreach my $testIndex (0..$#finalTestList) {
            # generate flat test
            &utils::generators::generateFlatTest($destinationType, $testType, \@finalTestList, \%cache, $testIndex);
            foreach my $shell (split ' ', $utils::helpers::types{'SHELL_TYPES'}) {
                # generate shell
                &utils::generators::generateShell($destinationType, $testType, $shell, \@finalTestList, \%cache, $testIndex);
            }
        }
    }
}
# generate latest changes log
foreach my $test (sort { $a->{date} cmp $b->{date} } values %cache) {
  print "$test->{date} ($test->{rev}): $test->{modulename}-$test->{number} - $test->{def}\n";
}
print "done\n";

##############################################################################
