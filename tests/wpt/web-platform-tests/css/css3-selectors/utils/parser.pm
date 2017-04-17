##############################################################################
# the processor                                                              #
##############################################################################

# This code is a mess and has numerous subtle bugs in its namespace
# handling. Do not expect it to pass any tests of its own.

package utils::parser;
use utils::helpers;
use strict;

my $NAMESPACE = 'http://www.example.org/css3tests';
my %months = (
              'january' => 1,
              'february' => 2,
              'march' => 3,
              'april' => 4,
              'may' => 5,
              'june' => 6,
              'july' => 7,
              'august' => 8,
              'september' => 9,
              'october' => 10,
              'november' => 11,
              'december' => 12,
             );

sub Init {
    my $parser = shift;
    $parser->{'Walker Data'} = {};
}

# This is the big workhorse -- it gets called for each start tag.
sub Start {
    my $parser = shift;
    my($tagName, @attrs) = @_;
    my @context = $parser->context;
    my($qualifiedTagName, @qualifiedAttrs) = &utils::helpers::qualifyStartTag($parser, $tagName, @attrs);
    my %qualifiedAttrs = (@qualifiedAttrs);

    # The root element
    if ((scalar(@context) == 0) and ($qualifiedTagName eq "{$NAMESPACE}csstest")) {
        foreach my $name (qw(def module modulename number rev)) {
            if (defined($qualifiedAttrs{$name})) {
                $parser->{'Walker Data'}->{$name} = $qualifiedAttrs{$name};
            }
        }
        if (defined($qualifiedAttrs{date})) {
          my $date = $qualifiedAttrs{date};
          $date =~ s/(.+)-(.+)-(.+)/sprintf('%02d-%02d-%02d', $3, $months{$2}, $1)/gose;
          $parser->{'Walker Data'}->{date} = $date;
        }
    } elsif (&utils::helpers::matchContext($parser, [[$NAMESPACE, 'csstest']]) and
             ($qualifiedTagName eq "{$NAMESPACE}author")) {
        if (defined($parser->{'Walker Data'}->{'author'})) {
            push(@{$parser->{'Walker Data'}->{'author'}}, '');
        } else {
            $parser->{'Walker Data'}->{'author'} = [''];
        }
    } elsif (&utils::helpers::matchContext($parser, [[$NAMESPACE, 'csstest']]) and
             ($qualifiedTagName eq "{$NAMESPACE}cssrules")) {
        # ok
    } elsif (&utils::helpers::matchContext($parser, [[$NAMESPACE, 'csstest']]) and
             ($qualifiedTagName eq "{$NAMESPACE}userinteraction")) {
        $parser->{'Walker Data'}->{'interactive'} = 1;
    } elsif (&utils::helpers::matchContext($parser, [[$NAMESPACE, 'csstest']]) and
             ($qualifiedTagName eq "{$NAMESPACE}dynamic")) {
        $parser->{'Walker Data'}->{'dynamic'} = 1;
    } elsif (&utils::helpers::matchContext($parser, [[$NAMESPACE, 'csstest']]) and
             ($qualifiedTagName eq "{$NAMESPACE}historyneeded")) {
        $parser->{'Walker Data'}->{'historyneeded'} = 1;
    } elsif (&utils::helpers::matchContext($parser, [[$NAMESPACE, 'csstest']]) and
             ($qualifiedTagName eq "{$NAMESPACE}code") and
             (not defined($parser->{'Walker Data'}->{'prefixes'}))) {
        # here we must begin to take stuff into account
        $parser->{'Walker Data'}->{'code-xml'} = '';
        $parser->{'Walker Data'}->{'code-xhtml'} = '';
        $parser->{'Walker Data'}->{'code-html'} = '';
        # first, all the namespace prefixes in scope
        $parser->{'Walker Data'}->{'prefixes'} = {};
        $parser->{'Walker Data'}->{'prefixesUsed'} = {};
        foreach my $prefix ($parser->current_ns_prefixes) {
            if ($prefix ne '#default') {
                $parser->{'Walker Data'}->{'prefixes'}->{$prefix} = $parser->expand_ns_prefix($prefix);
                $parser->{'Walker Data'}->{'prefixesUsed'}->{$prefix} = 0;
            }
        }
    } elsif (&utils::helpers::matchContext($parser, [[$NAMESPACE, 'csstest'],
                                                     [$NAMESPACE, 'code']])) { # child of code element
        $parser->xpcroak('restrict cannot be a child of code') if $qualifiedTagName eq "{$NAMESPACE}restrict";
        &processElement($parser, $tagName, 1, @attrs);
    } elsif (&utils::helpers::matchContext($parser, [[$NAMESPACE, 'csstest'],
                                                     [$NAMESPACE, 'code']], 1) and
             ($qualifiedTagName eq "{$NAMESPACE}restrict")) { # <restrict>, descendant of code element (must not be child)
        if (defined($parser->{'Walker Data'}->{'restrict'})) {
            $parser->xpcroak('<restrict> may not be nested');
        }
        if (defined($qualifiedAttrs{'for'})) {
            $parser->{'Walker Data'}->{'restrict'} = $qualifiedAttrs{'for'};
        } else {
            $parser->xpcroak('required attribute \'for\' missing');
        }
    } elsif (&utils::helpers::matchContext($parser, [[$NAMESPACE, 'csstest'],
                                                     [$NAMESPACE, 'code']], 1)) { # descendant of code element
        &processElement($parser, $tagName, 0, @attrs);
    } else {
        $parser->xpcroak("unexpected element $tagName in namespace ".$parser->namespace($tagName));
    }
}

sub CdataStart {
    my $parser = shift;
    if (&utils::helpers::matchContext($parser, [
             [$NAMESPACE, 'csstest'], [$NAMESPACE, 'code']], 1)) {
        $parser->{'Walker Data'}->{'code-xml'} .= '<![CDATA[' if applicable($parser, 'xml');
        $parser->{'Walker Data'}->{'code-xhtml'} .= '<![CDATA[' if applicable($parser, 'xhtml');
        # $parser->{'Walker Data'}->{'code-html'} .= '' if applicable($parser, 'html'); # HTML has no CDATA blocks
        $parser->{'Walker Data'}->{'cdata'} = 1;
    } else {
        # not technically invalid...
    }
}

sub CdataEnd {
    my $parser = shift;
    if (&utils::helpers::matchContext($parser, [
             [$NAMESPACE, 'csstest'], [$NAMESPACE, 'code']], 1)) {
        $parser->{'Walker Data'}->{'code-xml'} .= ']]>' if applicable($parser, 'xml');
        $parser->{'Walker Data'}->{'code-xhtml'} .= ']]>' if applicable($parser, 'xhtml');
        # $parser->{'Walker Data'}->{'code-html'} .= '' if applicable($parser, 'html'); # HTML has no CDATA blocks
        $parser->{'Walker Data'}->{'cdata'} = 0;
    } else {
        # not technically invalid...
    }
}

sub Comment {
    my $parser = shift;
    my($comment) = @_;
    if (&utils::helpers::matchContext($parser, [
             [$NAMESPACE, 'csstest'], [$NAMESPACE, 'code']], 1)) {
        $parser->{'Walker Data'}->{'code-xml'} .= "<!--$comment-->" if applicable($parser, 'xml');
        $parser->{'Walker Data'}->{'code-xhtml'} .= "<!--$comment-->" if applicable($parser, 'xhtml');
        $parser->{'Walker Data'}->{'code-html'} .= "<!--$comment-->" if applicable($parser, 'html');
    } else {
        # not technically invalid...
    }
}

sub Proc {
    my $parser = shift;
    my($target, $data) = @_;
    if (&utils::helpers::matchContext($parser, [
             [$NAMESPACE, 'csstest'], [$NAMESPACE, 'code']], 1)) {
        $parser->{'Walker Data'}->{'code-xml'} .= "<?$target $data?>" if applicable($parser, 'xml');
        $parser->{'Walker Data'}->{'code-xhtml'} .= "<?$target $data?>" if applicable($parser, 'xhtml');
        $parser->{'Walker Data'}->{'code-html'} .= "<?$target $data>" if applicable($parser, 'html');
    } else {
        # not technically invalid...
    }
}

# This is called for each line of a string of text (as well as the contents of any CDATA blocks, etc)
sub Char {
    my $parser = shift;
    my($text) = @_;
    if (&utils::helpers::matchContext($parser, [
             [$NAMESPACE, 'csstest'], [$NAMESPACE, 'author']])) {
        $parser->{'Walker Data'}->{'author'}->[$#{$parser->{'Walker Data'}->{'author'}}] .= $text;
    } elsif (&utils::helpers::matchContext($parser, [
             [$NAMESPACE, 'csstest'], [$NAMESPACE, 'cssrules']])) {
        $parser->{'Walker Data'}->{'cssrules'} .= $text;
    } elsif (&utils::helpers::matchContext($parser, [
             [$NAMESPACE, 'csstest']]) and ($text =~ /^\s+$/os)) {
        # ok
    } elsif (&utils::helpers::matchContext($parser, [
             [$NAMESPACE, 'csstest'], [$NAMESPACE, 'code']], 1)) {
        if (not $parser->{'Walker Data'}->{'cdata'}) {
            $text = &utils::helpers::escape($text);
        }
        $parser->{'Walker Data'}->{'code-xml'} .= $text if applicable($parser, 'xml');
        $parser->{'Walker Data'}->{'code-xhtml'} .= $text if applicable($parser, 'xhtml');
        $parser->{'Walker Data'}->{'code-html'} .= $text if applicable($parser, 'html');
    } else {
        $parser->xpcroak("found unexpected text");
    }
}

sub End {
    my $parser = shift;
    my($tagName) = @_;
    if (&utils::helpers::matchContext($parser, [[$NAMESPACE, 'csstest'],
                                                [$NAMESPACE, 'code']], 1) and
        (($tagName eq 'restrict') and ($parser->namespace($tagName) eq $NAMESPACE))) { # <restrict>, descendant of code element
        delete($parser->{'Walker Data'}->{'restrict'});
    } elsif (&utils::helpers::matchContext($parser, [[$NAMESPACE, 'csstest'],
                                               [$NAMESPACE, 'code']], 1)) {
        if ($parser->recognized_string ne '') {
            $parser->{'Walker Data'}->{'endTag'} = $parser->recognized_string;
        } else {
            # This was an empty tag with the short form <foo/>. This
            # guarentees that the element can have no children, so we
            # don't need to ensure the endTag bit is propagated
            # correctly across children.
        }
        # XML output
        $parser->{'Walker Data'}->{'code-xml'} .= $parser->{'Walker Data'}->{'endTag'} if applicable($parser, 'xml');
        # XHTML output
        $parser->{'Walker Data'}->{'code-xhtml'} .= $parser->{'Walker Data'}->{'endTag'} if applicable($parser, 'xhtml');
        # HTML output
        if (($parser->{'Walker Data'}->{'endTag'} ne '</input>') and
            ($parser->{'Walker Data'}->{'endTag'} ne '</br>')) {
            $parser->{'Walker Data'}->{'code-html'} .= $parser->{'Walker Data'}->{'endTag'} if applicable($parser, 'html');
        } # else HTML doesn't allow end tags for those
    } else {
        # ok
    }
}

sub Final {
    my $parser = shift;
    my $data = $parser->{'Walker Data'};
    $data->{'escapedcode-xml'} = &utils::helpers::escape($data->{'code-xml'});
    $data->{'escapedcode-xhtml'} = &utils::helpers::escape($data->{'code-xhtml'});
    $data->{'escapedcode-html'} = &utils::helpers::escape($data->{'code-html'});
    $data->{'escapedcode-css'} = &utils::helpers::escape($data->{'cssrules'});
    $data->{'namespaces'} = '';
    foreach my $prefix (keys %{$data->{'prefixes'}}) {
        if ($data->{'prefixesUsed'}->{$prefix}) {
            $data->{'namespaces'} .= " xmlns:${prefix}=\"$data->{'prefixes'}->{$prefix}\"";
        }
    }
    delete($parser->{'Walker Data'});
    return $data;
}

sub processElement {
    my $parser = shift;
    my($tagName, $child, @attrs) = @_;
    # $child is true if the element should declare its own default namespace if needed
    # (i.e. if element is a direct child of the <code> element)
    my @prefixes = $parser->current_ns_prefixes;
    # get the element stuff
    my $prefix = '';
    if ($parser->recognized_string =~ m/<([^\s:]+):/o) {
        $prefix = $1;
    }
    $parser->{'Walker Data'}->{'prefixesUsed'}->{$prefix} += 1 if exists $parser->{'Walker Data'}->{'prefixesUsed'}->{$prefix};
    my $default = $parser->expand_ns_prefix('#default');
    my $defaultXML = '';
    my $defaultXHTML = '';
    if ($child) {
        if (defined($default)) {
            if ($default ne 'http://www.w3.org/1999/xhtml') {
                $defaultXHTML = ' xmlns="'.&utils::helpers::escape($default).'"';
            }
            $defaultXML = ' xmlns="'.&utils::helpers::escape($default).'"';
        } else {
            $defaultXHTML = ' xmlns=""';
        }
    } # else handled as part of the new_ns_prefix fixup
    my $newNamespaces = '';
    my $newNamespacePrefixes = {};
    foreach my $newPrefix ($parser->new_ns_prefixes) {
        my $namespace = $parser->expand_ns_prefix($newPrefix);
        if (not defined($namespace)) {
            $namespace = '';
        }
        if ($newPrefix ne '#default') {
            $newNamespaces .= " xmlns:$newPrefix=\"".&utils::helpers::escape($namespace).'"';
            $newNamespacePrefixes->{$newPrefix} = $namespace;
        } elsif (not $child) {
            $newNamespaces .= ' xmlns="'.&utils::helpers::escape($namespace).'"';
        }
    }
    my %prefixLookup = map { if ($_ ne '#default') { $parser->expand_ns_prefix($_) => $_ } else { (); } } @prefixes;
    my $attributes = '';
    my $isName = 1;
    foreach my $attribute (@attrs) {
        if ($isName) {
            # we currently lose the actual prefix used and look it back up... this can be wrong if
            # there are multiple prefixes defined for the same namespace.
            my $attrNamespace;
            if ($attribute =~ s/^\|//o) {
                # this handles a bug in XML::Parser::Expat with attributes of the form:
                # <element xmlns="" xmlns:none="" none:this="will be called '|this' and not 'this' in $attribute" />
                # XXX actually the bug is that that doesn't throw a well-formedness exception XXX
                $attrNamespace = '';
            } else {
                $attrNamespace = $parser->namespace($attribute);
            }
            my $attrPrefix;
            if (defined($attrNamespace)) {
                $parser->{'Walker Data'}->{'namespaced'} = 1;
                $parser->{'Walker Data'}->{'only-xml'} = 1 if applicable($parser, 'html');
                if ($attrNamespace eq 'http://www.w3.org/XML/1998/namespace') {
                    $attrPrefix = 'xml';
                } else {
                    $attrPrefix = $prefixLookup{$attrNamespace};
                    $parser->{'Walker Data'}->{'prefixesUsed'}->{$attrPrefix} += 1 if exists $parser->{'Walker Data'}->{'prefixesUsed'}->{$attrPrefix};
                }
                $attrPrefix .= ':';
            } else {
                $attrPrefix = '';
            }
            $attributes .= " $attrPrefix$attribute=\"";
        } else {
            $attributes .= &utils::helpers::escape($attribute).'"';
        }
        $isName = not($isName);
    }
    $prefix .= ':' if $prefix ne '';
    # XML output:
    $parser->{'Walker Data'}->{'code-xml'} .= "<$prefix$tagName$defaultXML$newNamespaces$attributes>" if applicable($parser, 'xml');
    # XHTML output
    $parser->{'Walker Data'}->{'code-xhtml'} .= "<$prefix$tagName$defaultXHTML$newNamespaces$attributes>" if applicable($parser, 'xhtml');
    # HTML output is same as XHTML output except for namespaces - flag if there are any
    $parser->{'Walker Data'}->{'code-html'} .= "<$tagName$attributes>" if applicable($parser, 'html');

    if ($prefix ne '' or $defaultXHTML ne '' or $newNamespaces ne '') {
        $parser->{'Walker Data'}->{'namespaced'} = 1;
        $parser->{'Walker Data'}->{'only-xml'} = 1 if applicable($parser, 'html');
    }
    $parser->{'Walker Data'}->{'endTag'} = "</$prefix$tagName>"; # used to regenerate the end tag if required (i.e. if this was originally an empty start tag)
}

sub applicable {
    my($parser, $for) = @_;
    return ((not defined($parser->{'Walker Data'}->{'restrict'})) or
            ($parser->{'Walker Data'}->{'restrict'} =~ m/\b # word boundary
                                                   \Q$for\E # quote $for string (so that $for is not treated as regexp)
                                                         \b # word boundary
                                                          /x));
}
