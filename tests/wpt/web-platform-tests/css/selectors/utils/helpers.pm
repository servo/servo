package utils::helpers;
use Data::Dumper; # DEPENDENCY
use strict;

# The Test Type Hash
# Note: Adding types to this hash is not enough... you also have to
# add code to the Makefile, the parser, the shortlist functions in
# this file, and the generators in the generator.pm module.
%utils::helpers::types = (
                          'DESTINATION_TYPES' => 'xhtml html xml', # explicitly listed in Makefile, shortlister and generator
                          'SHELL_TYPES' => 'xhtml_iframe xhtml_object xhtml_frames html_iframe html_object html_frames xlink_embed tng', # explicitly listed in generator and just below
                          'TEST_TYPES' => 'full static history interactive', # explicitly listed in shortlister, generator, and just below
                          'TEST_TYPES' => 'full static history interactive dynamic', # explicitly listed in shortlister, generator, and just below
                         );

%utils::helpers::DestinationTypeTitles = (
                                   'xhtml' => 'XHTML',
                                   'html' => 'HTML',
                                   'xml' => 'XML',
                                  );

%utils::helpers::ShellTypeTitles  = (
                                     'xhtml_iframe' => 'XHTML <iframe>',
                                     'xhtml_object' => 'XHTML <object>',
                                     'xhtml_frames' => 'XHTML <frame>',
                                     'html_iframe' => 'HTML <iframe>',
                                     'html_object' => 'HTML <object>',
                                     'html_frames' => 'HTML <frame>',
                                     'xlink_embed' => 'XLink embed',
                                     'tng' => 'TNG Format',
                                 );

%utils::helpers::ShellTypeDescriptions  = (
                                     'xhtml_iframe' => 'Tests consist of an XHTML wrapper page summarising the test and linking to the actual test content using an <iframe> element.',
                                     'xhtml_object' => 'Tests consist of an XHTML wrapper page summarising the test and linking to the actual test content using an <object> element.',
                                     'xhtml_frames' => 'Tests consist of a two frame XHTML frameset, the top frame being an XHTML wrapper page summarising the test and the bottom frame being the actual test content.',
                                     'html_iframe' => 'Tests consist of an HTML wrapper page summarising the test and linking to the actual test content using an <iframe> element.',
                                     'html_object' => 'Tests consist of an HTML wrapper page summarising the test and linking to the actual test content using an <object> element.',
                                     'html_frames' => 'Tests consist of a two frame HTML frameset, the top frame being an HTML wrapper page summarising the test and the bottom frame being the actual test content.',
                                     'xlink_embed' => 'Tests consist of an XML page summarising the test and linking to the actual test content using an XLink with the show axis set to embed.',
                                     'tng' => 'Tests consist of an HTML page with a brief test summary and navigation aids and a link to the test content using an <object> tag. This test format is designed to be stylistically compatible with the TNG test format used for other CSS test suites.',
                                 );

%utils::helpers::TestTypeTitles = (
                                   'full' => '',
                                   'static' => 'static tests category',
                                   'history' => 'history-related tests category',
                                   'interactive' => 'interactive tests category',
                                   'dynamic' => 'dynamic tests category',
                                  );

%utils::helpers::TestTypeDescriptions = (
                                   'full' => 'The complete set of tests.',
                                   'static' => 'The list of static tests (those that involve in scripting and no user interaction).',
                                   'history' => 'Tests requiring that the UA have some sort of session history.',
                                   'interactive' => 'The tests that require user interaction.',
                                   'dynamic' => 'Pages consisting of a script that dynamically modifies the document in order to complete the test.',
                                  );

%utils::helpers::TestTypeShortTitles = (
                                   'full' => 'full',
                                   'static' => 'static',
                                   'history' => 'history-related',
                                   'interactive' => 'interactive',
                                   'dynamic' => 'dynamic',
                                  );

sub qualifyStartTag {
    my($parser, $localTagName, @localAttributes) = @_;

    # get the qualified tag name
    my $qualifiedTagName;
    my $namespace = $parser->namespace($localTagName);
    if (defined($namespace)) {
        $qualifiedTagName = "{$namespace}$localTagName";
    } else {
        $qualifiedTagName = $localTagName;
    }

    # get the qualified attributes
    my @qualifiedAttributes;
    my $isName = 1;
    foreach my $attribute (@localAttributes) {
        if ($isName) {
            $namespace = $parser->namespace($attribute);
            if (defined($namespace)) {
                push(@qualifiedAttributes, "{$namespace}$attribute");
            } else {
                push(@qualifiedAttributes, $attribute);
            }
        } else {
            #my $data = $attribute;
            #if ($data =~ s/^([^:]+)://o) {
            #    $namespace = $parser->expand_ns_prefix($1);
            #} else {
            #    $namespace = $parser->expand_ns_prefix('#default');
            #}
            #if (defined($namespace)) {
            #    push(@qualifiedAttributes, "{$namespace}$data"); # value
            #} else {
                push(@qualifiedAttributes, $attribute); # value
            #}
        }
        $isName = not $isName;
    }

    # add the namespace declarations
    foreach my $prefix ($parser->new_ns_prefixes) {
        if ($prefix eq '#default') {
            push(@qualifiedAttributes, 'xmlns', $parser->expand_ns_prefix($prefix));
        } else {
            push(@qualifiedAttributes, "xmlns:$prefix", $parser->expand_ns_prefix($prefix));
        }
    }

    # return it all
    return ($qualifiedTagName, @qualifiedAttributes);
}

sub matchContext {
    my($parser, $match, $loose) = @_;
    my @context = $parser->context;
    if (defined($loose)) {
        return 0 unless (scalar(@context) >= scalar(@$match));
    } else {
        return 0 unless (scalar(@context) == scalar(@$match));
    }
    foreach my $element (@context[0..$#$match]) {
        my($namespace, $tagName) = @{shift(@$match)};
        return 0 unless ($element eq $tagName);
        my $matchNamespace = $parser->namespace($element);
        return 0 unless ((defined($matchNamespace) == defined($namespace)) and
                         ($matchNamespace eq $namespace));
    }
    return 1;
}

sub shortlistTestsForDestination {
    my($type, $testList, $tests) = @_;
    my @result;
    foreach my $test (@$testList) {
        if (exists($tests->{$test})) {
            if ($type eq 'xhtml') {
                push(@result, $test);
            } elsif ($type eq 'xml') {
                push(@result, $test);
            } elsif ($type eq 'html') {
                if (not $tests->{$test}->{'only-xml'}) {
                    push(@result, $test);
                }
            } else {
                die("Don't know how to shortlist tests for $type");
            }
        }
    }
    return @result;
}

sub shortlistTestsForTypes {
    my($type, $testList, $tests) = @_;
    my @result;
    foreach my $test (@$testList) {
        if (exists($tests->{$test})) {
            if ($type eq 'full') {
                push(@result, $test);
            } elsif ($type eq 'static') {
                if (not ($tests->{$test}->{'dynamic'} or $tests->{$test}->{'interactive'})) {
                    push(@result, $test);
                }
            } elsif ($type eq 'history') {
                if ($tests->{$test}->{'historyneeded'}) {
                    push(@result, $test);
                }
            } elsif ($type eq 'interactive') {
                if ($tests->{$test}->{'interactive'}) {
                    push(@result, $test);
                }
            } elsif ($type eq 'dynamic') {
                if ($tests->{$test}->{'dynamic'}) {
                    push(@result, $test);
                }
            } else {
                die("Don't know how to shortlist $type tests");
            }
        }
    }
    return @result;
}

sub readCache {
    open(CACHE, '<cache') or return {};
    local $/ = undef;
    my $data = <CACHE>;
    close(CACHE);
    if ($data) {
        return eval $data;
    } else {
        return {};
    }
}

sub writeCache {
    open(CACHE, '>cache');
    print CACHE Data::Dumper->new([@_])->Purity(1)->Terse(1)->Indent(0)->Dump;
    close(CACHE);
}

sub escape {
    $_ = shift;
    # because XML::Parser::Expat::escape() doesn't correctly escape "]]>"...
    s/&/&amp;/go;
    s/</&lt;/go;
    s/>/&gt;/go;
    s/"/&quot;/go; #"; # (reset fontlock)
    s/'/&#39;/go; #'; # (reset fontlock) # note -- this would be apos but apos is not in HTML 4.01
    return $_;
}

##############################################################################
