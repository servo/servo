=pod
Copyright (c) 2007 Philip Taylor

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
=cut

use strict;
use warnings;

use Compress::Zlib();
use Cairo;
use PDL qw(float byte);

sub chunk {
    my ($name, $data) = @_;
    (pack 'N', length $data) . $name . $data . (pack 'N', Compress::Zlib::crc32($name . $data));
}

sub IHDR {
    my ($img) = @_;
    return chunk 'IHDR', pack 'NNCCCCC',
        $img->{width}, $img->{height},
        $img->{bit_depth}, $img->{color_type},
        $img->{compression_method}, $img->{filter_method}, $img->{interlace_method};
}

sub gAMA {
    my ($img) = @_;
    return chunk 'gAMA', pack 'N', $img->{gamma};
}

sub sRGB {
    my ($img) = @_;
    return chunk 'sRGB', pack 'C', $img->{rendering_intent};
}

sub PLTE {
    my ($img) = @_;
    return chunk 'PLTE', pack 'C*', @{$img->{colours}};
}

sub tRNS {
    my ($img) = @_;
    return chunk 'tRNS', pack 'C*', @{$img->{values}};
}

sub IDAT {
    my ($img) = @_;
    return chunk 'IDAT', xdat_data($img);
}

sub IDAT_split {
    my ($img, $blocksize) = @_;
    my $c = xdat_data($img);
    my @out;
    while (length $c) {
        push @out, chunk 'IDAT', substr $c, 0, $blocksize, '';
    }
    return @out;
}

sub IEND {
    my ($img) = @_;
    return chunk 'IEND', '';
}

sub acTL {
    my ($img) = @_;
    return chunk 'acTL', pack 'NN', $img->{num_frames}, $img->{num_plays};
}

sub fcTL {
    my ($img) = @_;
    return chunk 'fcTL', pack 'NNNNNnnCC',
        $img->{sequence_number},
        $img->{width}, $img->{height},
        $img->{x_offset}, $img->{y_offset},
        $img->{delay_num}, $img->{delay_den},
        $img->{dispose_op}, $img->{blend_op};
}

sub fdAT {
    my ($img) = @_;
    return chunk 'fdAT', (pack 'N', $img->{sequence_number}) . xdat_data($img);
}

sub xdat_data {
    my ($img) = @_;
    return compress(filter($img->{image_data}, $img->{width}, $img->{height}, $img->{depth}));
}

use constant DISPOSE_NONE => 0;
use constant DISPOSE_BACKGROUND => 1;
use constant DISPOSE_PREVIOUS => 2;
use constant BLEND_SOURCE => 0;
use constant BLEND_OVER => 1;

sub filter {
    my ($imagedata, $width, $height, $depth) = @_;
    my $out = '';
    for my $scanline (0..$height-1) {
        $out .= pack 'C', 0;
        $out .= substr($imagedata, $scanline*$width*$depth/8, $width*$depth/8);
    }
    return $out;
}

sub compress {
    my ($filtered) = @_;
    return Compress::Zlib::compress($filtered);
}


sub fix_bitmap {
    my ($d) = @_;
    # Flip BGRA->RGBA, and undo premultiplication

    my $pdl = float unpack 'C*', $d;
    my $pdl2 = byte $pdl;
    my $a = 255 / $pdl->mslice([3, -1, 4]);
    $pdl2->mslice([0, -1, 4]) .= $pdl->mslice([2, -1, 4])*$a;
    $pdl2->mslice([1, -1, 4]) .= $pdl->mslice([1, -1, 4])*$a;
    $pdl2->mslice([2, -1, 4]) .= $pdl->mslice([0, -1, 4])*$a;
    return ${(byte $pdl2)->get_dataref};
=pod
    my @d = unpack 'C*', $d;
    my $a;
    for (map $_*4, 0..$#d/4) {
        if ($a = $d[$_+3]) {
            $a = 255 / $a;
            @d[$_, $_+1, $_+2] = ($d[$_+2]*$a, $d[$_+1]*$a, $d[$_]*$a);
        } # else alpha=0 hence r=g=b=0, so nothing to do
    }
    return pack 'C*', @d;
=cut
}

sub create_surface {
    my ($w, $h, $type, @data) = @_;
    my $surface = Cairo::ImageSurface->create('argb32', $w, $h);
    my $cr = Cairo::Context->create($surface);

    if ($type eq 'red') {
        ($type, @data) = ('solid', 1, 0, 0, 1);
    } elsif ($type eq 'green') {
        ($type, @data) = ('solid', 0, 1, 0, 1);
    } elsif ($type eq 'blue') {
        ($type, @data) = ('solid', 0, 0, 1, 1);
    } elsif ($type eq 'cyan') {
        ($type, @data) = ('solid', 0, 1, 1, 1);
    } elsif ($type eq 'magenta') {
        ($type, @data) = ('solid', 1, 0, 1, 1);
    } elsif ($type eq 'yellow') {
        ($type, @data) = ('solid', 1, 1, 0, 1);
    } elsif ($type eq 'transparent') {
        ($type, @data) = ('solid', 0, 0, 0, 0);
    }

    if ($type eq 'solid') {
        $cr->rectangle(0, 0, $w, $h);
        $cr->set_source_rgba(@data);
        $cr->fill;
    } elsif ($type eq 'doublerect') {
        $cr->rectangle(0, 0, $w, $h);
        $cr->set_source_rgba(@data[0..3]);
        $cr->fill;
        $cr->rectangle(int($w/4), int($h/4), int($w/2), int($h/2));
        $cr->set_source_rgba(@data[4..7]);
        $cr->fill;
    } else {
        die "Invalid create_surface type '$type'";
    }
    return { width => $w, height => $h, depth => 32, data => fix_bitmap($surface->get_data) };
}

sub create_raw_surface {
    my ($w, $h, $d, $data) = @_;
    return { width => $w, height => $h, depth => $d, data => $data };
}

sub find_errors {
    my (@img) = @_;
    my @chunks;
    {
        my @img2 = @img;
        push @chunks, [ splice @img2, 0, 2 ] while @img2;
    }

    my $chunknames = join '', map "<$_->[0]>", @chunks;

    my @errors;

    my $has_actl = ($chunknames =~ /<acTL>/);

    if ($has_actl) {
        # acTL must be before IDAT
        if ($chunknames =~ /<IDAT>.*<acTL>/) {
            push @errors, "acTL after IDAT";
        }

        # Must have only one acTL (TODO: in spec?)
        if ($chunknames =~ /<acTL>.*<acTL>/) {
            push @errors, "More than one acTL";
        }

        my $num_frames = {@img}->{acTL}[0];

        # num_frames > 0
        if ($num_frames <= 0) {
            push @errors, "num_frames <= 0";
        }

        # num_frames = count(fcTL)
        my $num_fctls = grep $_->[0] eq 'fcTL', @chunks;
        if ($num_frames != $num_fctls) {
            push @errors, "num_frames ($num_frames) != number of fcTLs ($num_fctls)";
        }
    }

    # Check sequence numbers (start from 0, no duplicates or gaps)
    my @seqnos;
    for (grep { $_->[0] =~ /^(fcTL|fdAT|fdAT_split)$/ } @chunks) {
        push @seqnos, $_->[1][0];
    }
    if (@seqnos and (join ',', @seqnos) ne (join ',', 0..$#seqnos)) {
        push @errors, "Incorrect sequence numbers";
    }

    return @errors;
}

sub create_image {
    my ($filename, @img) = @_;
    my @chunks;
    while (@img) {
        my ($chunk, $data) = splice @img, 0, 2;
        if ($chunk eq 'IHDR') {
            push @chunks, IHDR {
                width => $data->[0],
                height => $data->[1],
                bit_depth => defined $data->[2] ? $data->[2] : 8,
                color_type => defined $data->[3] ? $data->[3] : 6,
                compression_method => 0,
                filter_method => 0,
                interlace_method => 0,
            };
        } elsif ($chunk eq 'IEND') {
            push @chunks, IEND { }
        } elsif ($chunk eq 'gAMA') {
            push @chunks, gAMA {
                gamma => int(100_000*$data->[0]),
            };
        } elsif ($chunk eq 'sRGB') {
            push @chunks, sRGB {
                rendering_intent => $data->[0],
            };
        } elsif ($chunk eq 'PLTE') {
            push @chunks, PLTE {
                colours => $data,
            };
        } elsif ($chunk eq 'tRNS') {
            push @chunks, tRNS {
                values => $data,
            };
        } elsif ($chunk eq 'acTL') {
            push @chunks, acTL {
                num_frames => $data->[0],
                num_plays => $data->[1],
            };
        } elsif ($chunk eq 'fcTL') {
            push @chunks, fcTL {
                sequence_number => $data->[0],
                width => $data->[1],
                height => $data->[2],
                x_offset => $data->[3],
                y_offset => $data->[4],
                delay_num => $data->[5],
                delay_den => $data->[6],
                dispose_op => $data->[7],
                blend_op => $data->[8],
            };
        } elsif ($chunk eq 'IDAT') {
            push @chunks, IDAT {
                depth => $data->[0]{depth},
                width => $data->[0]{width},
                height => $data->[0]{height},
                image_data => $data->[0]{data},
            }
        } elsif ($chunk eq 'IDAT_split') {
            my $c = xdat_data {
                depth => $data->[2]{depth},
                width => $data->[2]{width},
                height => $data->[2]{height},
                image_data => $data->[2]{data},
            };
            if ($data->[1] == -1) {
                $c = substr $c, $data->[0];
            } else {
                $c = substr $c, $data->[0], $data->[1] - $data->[0];
            }
            push @chunks, chunk 'IDAT', $c;
        } elsif ($chunk eq 'fdAT') {
            push @chunks, fdAT {
                sequence_number => $data->[0],
                depth => $data->[1]{depth},
                width => $data->[1]{width},
                height => $data->[1]{height},
                image_data => $data->[1]{data},
            }
        } elsif ($chunk eq 'fdAT_split') {
            my $c = xdat_data {
                depth => $data->[3]{depth},
                width => $data->[3]{width},
                height => $data->[3]{height},
                image_data => $data->[3]{data},
            };
            if ($data->[2] == -1) {
                $c = substr $c, $data->[1];
            } else {
                $c = substr $c, $data->[1], $data->[2] - $data->[1];
            }
            push @chunks, chunk 'fdAT', (pack 'N', $data->[0]) . $c;
        } else {
            die "Invalid create_image chunk '$chunk'";
        }
    }
    open my $fh, '>', "images/$filename.png" or die $!;
    binmode $fh;
    print $fh "\211PNG\r\n\032\n", @chunks;
}

use constant W => 128;
use constant H => 64;

sub escape_html {
    my ($t) = @_;
    $t =~ s/&/&amp;/g;
    $t =~ s/</&lt;/g;
    return $t;
}

my $img_id = '000';
sub handle_html_png {
    my ($code) = @_;
    my $name = $img_id++;
    my @img = eval '(' . $code . ')';
    die $@ if $@;
    create_image($name, @img);
    my $data = $code;
    $data =~ s/^\s*(.*?)\s*$/$1/sg;
    $data =~ s/([^a-zA-Z0-9])/sprintf('%%%02X', ord $1)/eg;
    my $errors = (join '; ', map escape_html($_), find_errors(@img)) || 'None';
    return qq{<p>}
        #. qq{<object data="$name.png" class="testimage"><strong>Did not load image.</strong></object>} # IE doesn't like this
        . qq{<img src="$name.png" alt="Did not load image" class="testimage">\n}
        . qq{<p><a href="data:text/plain,$data">(source)</a>\n}
        #. qq{<p>Expected errors: $errors\n}
        ;
}
# TODO: regexping HTML is nasty - should use a better input data format instead
sub handle_html_case {
    my ($title) = @_;
    my $id = lc $title;
    $id =~ s/[^a-z0-9]+/-/g;
    $id =~ s/^-*(.*?)-*$/$1/g;
    return qq{<div class="case" id="$id">\n<p><a href="#$id">#</a> $title\n};
}

open my $in, 'source.html' or die $!;
my $html = do { local $/; <$in> };
$html =~ s/<png>(.*?)<\/png>/handle_html_png($1)/seg;
$html =~ s/<div class="case">\n<p>(.*?)\n/handle_html_case($1)/eg;
open my $out, '>', 'images/tests.html' or die $!;
print $out $html;