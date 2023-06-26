"""
A direct translation of the webvtt file parsing algorithm.

See https://w3c.github.io/webvtt/#file-parsing for documentation
"""
import re
import string

SPACE_CHARACTERS = [' ', '\t', '\n', '\f', '\r']
SPACE_SPLIT_PATTERN = r"[{}]*".format(''.join(SPACE_CHARACTERS))
DIGITS = string.digits

class DictInit:
    def __init__(self, **dict):
        self.__dict__.update(dict)

class VTTCue(DictInit): pass
class VTTRegion(DictInit): pass
class Stylesheet(DictInit): pass

class W3CParser:
    input = None
    position = None

    def collect_characters(self, condition):
        result = ""
        while self.position < len(self.input) and condition(self.input[self.position]):
            result += self.input[self.position]
            self.position += 1
        return result

    def skip_whitespace(self):
        self.collect_characters(lambda c: c in SPACE_CHARACTERS)

    def parse_percentage_string(self, input):
        'parse a percentage string'

        # 1.
        input = input

        # 2.
        if not re.match(r'^\d+(\.\d+)?%$', input):
            return None

        # 3.
        percentage = float(input[:-1])

        # 4.
        if percentage < 0 or percentage > 100:
            return None

        # 5.
        return percentage

class VTTParser(W3CParser):
    def __init__(self, input):
        self.input = input
        self.position = 0
        self.seen_cue = False

        self.text_tracks = []
        self.stylesheets = []
        self.regions = []
        self.errors = []

    def parse(self):
        'WebVTT parser algorithm'

        # 1.
        self.input = self.input.replace('\0', '\ufffd').replace('\r\n', '\n').replace('\r', '\n')

        # 2.
        self.position = 0

        # 3.
        self.seen_cue = False

        # 4.
        if len(self.input) < 6:
            self.errors.append('input too small for webvtt')
            return

        # 5.
        if len(self.input) == 6 and self.input != 'WEBVTT':
            self.errors.append('invalid webvtt header')
            return

        # 6.
        if len(self.input) > 6:
            if not (self.input[0:6] == 'WEBVTT' and self.input[6] in ['\u0020', '\u0009', '\u000A']):
                self.errors.append('invalid webvtt header')
                return

        # 7.
        self.collect_characters(lambda c: c != '\n')

        # 8.
        if self.position >= len(self.input):
            return

        # 9.
        if self.input[self.position] == '\n':
            self.position += 1

        # 10.
        if self.position >= len(self.input):
            return

        # 11.
        if self.input[self.position] != '\n':
            self.collect_block(in_header = True)
        else:
            self.position += 1

        # 12.
        self.collect_characters(lambda c: c == '\n')

        # 13.
        self.regions = []

        # 14.
        while self.position < len(self.input):
            # 1.
            block = self.collect_block()

            # 2.
            if isinstance(block, VTTCue):
                self.text_tracks.append(block)

            # 3.
            elif isinstance(block, Stylesheet):
                self.stylesheets.append(block)

            # 4.
            elif isinstance(block, VTTRegion):
                self.regions.append(block)

            # 5.
            self.collect_characters(lambda c: c == '\n')

        # 15.
        return

    def collect_block(self, in_header = False):
        'collect a WebVTT block'

        # 1. (done by class)

        line_count = 0                    # 2.
        previous_position = self.position # 3.
        line = ""                         # 4.
        buffer = ""                       # 5.
        seen_eof = False                  # 6.
        seen_arrow = False                # 7.
        cue = None                        # 8.
        stylesheet = None                 # 9.
        region = None                     # 10.

        # 11.
        while True:
            # 1.
            line = self.collect_characters(lambda c: c != '\n')

            # 2.
            line_count += 1

            # 3.
            if self.position >= len(self.input):
                seen_eof = True
            else:
                self.position += 1

            # 4.
            if '-->' in line:
                # 1.
                if not in_header and (line_count == 1 or line_count == 2 and not seen_arrow):
                    # 1.
                    seen_arrow = True

                    # 2.
                    previous_position = self.position

                    # 3.
                    cue = VTTCue(
                        id = buffer,
                        pause_on_exit = False,
                        region = None,
                        writing_direction = 'horizontal',
                        snap_to_lines = True,
                        line = 'auto',
                        line_alignment = 'start alignment',
                        position = 'auto',
                        position_alignment = 'auto',
                        cue_size = 100,
                        text_alignment = 'center',
                        text = '',
                    )

                    # 4.
                    if not VTTCueParser(self, line, cue).collect_cue_timings_and_settings():
                        cue = None
                    else:
                        buffer = ''
                        self.seen_cue = True # DIFFERENCE

                else:
                    self.errors.append('invalid webvtt cue block')
                    self.position = previous_position
                    break

            # 5.
            elif line == '':
                break

            # 6.
            else:
                # 1.
                if not in_header and line_count == 2:
                    # 1.
                    if not self.seen_cue and re.match(r'^STYLE\s*$', buffer):
                        stylesheet = Stylesheet(
                            location = None,
                            parent = None,
                            owner_node = None,
                            owner_rule = None,
                            media = None,
                            title = None,
                            alternate = False,
                            origin_clean = True,
                            source = None,
                        )
                        buffer = ''
                    # 2.
                    elif not self.seen_cue and re.match(r'^REGION\s*$', buffer):
                        region = VTTRegion(
                            id = '',
                            width = 100,
                            lines = 3,
                            anchor_point = (0, 100),
                            viewport_anchor_point = (0, 100),
                            scroll_value = None,
                        )
                        buffer = ''

                # 2.
                if buffer != '':
                    buffer += '\n'

                # 3.
                buffer += line

                # 4.
                previous_position = self.position

            # 7.
            if seen_eof:
                break

        # 12.
        if cue is not None:
            cue.text = buffer
            return cue

        # 13.
        elif stylesheet is not None:
            stylesheet.source = buffer
            return stylesheet

        # 14.
        elif region is not None:
            self.collect_region_settings(region, buffer)
            return region

        # 15.
        return None

    def collect_region_settings(self, region, input):
        'collect WebVTT region settings'

        # 1.
        settings = re.split(SPACE_SPLIT_PATTERN, input)

        # 2.
        for setting in settings:
            # 1.
            if ':' not in setting:
                continue

            index = setting.index(':')
            if index in [0, len(setting) - 1]:
                continue

            # 2.
            name = setting[:index]

            # 3.
            value = setting[index + 1:]

            # 4.
            if name == "id":
                region.id = value

            elif name == "width":
                percentage = self.parse_percentage_string(value)
                if percentage is not None:
                    region.width = percentage

            elif name == "lines":
                # 1.
                if not re.match(r'^\d+$', value):
                    continue

                # 2.
                number = int(value)

                # 3.
                region.lines = number

            elif name == "regionanchor":
                # 1.
                if ',' not in value:
                    continue

                #. 2.
                index = value.index(',')
                anchorX = value[:index]

                # 3.
                anchorY = value[index + 1:]

                # 4.
                percentageX = self.parse_percentage_string(anchorX)
                percentageY = self.parse_percentage_string(anchorY)
                if None in [percentageX, percentageY]:
                    continue

                # 5.
                region.anchor_point = (percentageX, percentageY)

            elif name == "viewportanchor":
                # 1.
                if ',' not in value:
                    continue

                #. 2.
                index = value.index(',')
                viewportanchorX = value[:index]

                # 3.
                viewportanchorY = value[index + 1:]

                # 4.
                percentageX = self.parse_percentage_string(viewportanchorX)
                percentageY = self.parse_percentage_string(viewportanchorY)
                if None in [percentageX, percentageY]:
                    continue

                # 5.
                region.viewport_anchor_point = (percentageX, percentageY)

            elif name == "scroll":
                # 1.
                if value == "up":
                    region.scroll_value = "up"

            # 5.
            continue


class VTTCueParser(W3CParser):
    def __init__(self, parent, input, cue):
        self.parent = parent
        self.errors = self.parent.errors
        self.input = input
        self.position = 0
        self.cue = cue

    def collect_cue_timings_and_settings(self):
        'collect WebVTT cue timings and settings'

        # 1. (handled by class)

        # 2.
        self.position = 0

        # 3.
        self.skip_whitespace()

        # 4.
        timestamp = self.collect_timestamp()
        if timestamp is None:
            self.errors.append('invalid start time for VTTCue')
            return False
        self.cue.start_time = timestamp

        # 5.
        self.skip_whitespace()

        # 6.
        if self.input[self.position] != '-':
            return False
        self.position += 1

        # 7.
        if self.input[self.position] != '-':
            return False
        self.position += 1

        # 8.
        if self.input[self.position] != '>':
            return False
        self.position += 1

        # 9.
        self.skip_whitespace()

        # 10.
        timestamp = self.collect_timestamp()
        if timestamp is None:
            self.errors.append('invalid end time for VTTCue')
            return False
        self.cue.end_time = timestamp

        # 11.
        remainder = self.input[self.position:]

        # 12.
        self.parse_settings(remainder)

        # Extra
        return True

    def parse_settings(self, input):
        'parse the WebVTT cue settings'

        # 1.

        settings = re.split(SPACE_SPLIT_PATTERN, input)

        # 2.
        for setting in settings:
            # 1.
            if ':' not in setting:
                continue

            index = setting.index(':')
            if index in [0, len(setting) - 1]:
                continue

            # 2.
            name = setting[:index]

            # 3.
            value = setting[index + 1:]

            # 4.
            if name == 'region':
                # 1.
                last_regions = (region for region in reversed(self.parent.regions) if region.id == value)
                self.cue.region = next(last_regions, None)

            elif name == 'vertical':
                # 1. and 2.
                if value in ['rl', 'lr']:
                    self.cue.writing_direction = value

            elif name == 'line':
                # 1.
                if ',' in value:
                    index = value.index(',')
                    linepos = value[:index]
                    linealign = value[index + 1:]

                # 2.
                else:
                    linepos = value
                    linealign = None

                # 3.
                if not re.search(r'\d', linepos):
                    continue

                # 4.
                if linepos[-1] == '%':
                    number = self.parse_percentage_string(linepos)
                    if number is None:
                        continue
                else:
                    # 1.
                    if not re.match(r'^[-\.\d]*$', linepos):
                        continue

                    # 2.
                    if '-' in linepos[1:]:
                        continue

                    # 3.
                    if linepos.count('.') > 1:
                        continue

                    # 4.
                    if '.' in linepos:
                        if not re.search(r'\d\.\d', linepos):
                            continue

                    # 5.
                    number = float(linepos)

                # 5.
                if linealign == "start":
                    self.cue.line_alignment = 'start'

                # 6.
                elif linealign == "center":
                    self.cue.line_alignment = 'center'

                # 7.
                elif linealign == "end":
                    self.cue.line_alignment = 'end'

                # 8.
                elif linealign != None:
                    continue

                # 9.
                self.cue.line = number

                # 10.
                if linepos[-1] == '%':
                    self.cue.snap_to_lines = False
                else:
                    self.cue.snap_to_lines = True

            elif name == 'position':
                # 1.
                if ',' in value:
                    index = value.index(',')
                    colpos = value[:index]
                    colalign = value[index + 1:]

                # 2.
                else:
                    colpos = value
                    colalign = None

                # 3.
                number = self.parse_percentage_string(colpos)
                if number is None:
                    continue

                # 4.
                if colalign == "line-left":
                    self.cue.line_alignment = 'line-left'

                # 5.
                elif colalign == "center":
                    self.cue.line_alignment = 'center'

                # 6.
                elif colalign == "line-right":
                    self.cue.line_alignment = 'line-right'

                # 7.
                elif colalign != None:
                    continue

                # 8.
                self.cue.position = number

            elif name == 'size':
                # 1.
                number = self.parse_percentage_string(value)
                if number is None:
                    continue

                # 2.
                self.cue.cue_size = number

            elif name == 'align':
                # 1.
                if value == 'start':
                    self.cue.text_alignment = 'start'

                # 2.
                if value == 'center':
                    self.cue.text_alignment = 'center'

                # 3.
                if value == 'end':
                    self.cue.text_alignment = 'end'

                # 4.
                if value == 'left':
                    self.cue.text_alignment = 'left'

                # 5.
                if value == 'right':
                    self.cue.text_alignment = 'right'

            # 5.
            continue

    def collect_timestamp(self):
        'collect a WebVTT timestamp'

        # 1. (handled by class)

        # 2.
        most_significant_units = 'minutes'

        # 3.
        if self.position >= len(self.input):
            return None

        # 4.
        if self.input[self.position] not in DIGITS:
            return None

        # 5.
        string = self.collect_characters(lambda c: c in DIGITS)

        # 6.
        value_1 = int(string)

        # 7.
        if len(string) != 2 or value_1 > 59:
            most_significant_units = 'hours'

        # 8.
        if self.position >= len(self.input) or self.input[self.position] != ':':
            return None
        self.position += 1

        # 9.
        string = self.collect_characters(lambda c: c in DIGITS)

        # 10.
        if len(string) != 2:
            return None

        # 11.
        value_2 = int(string)

        # 12.
        if most_significant_units == 'hours' or self.position < len(self.input) and self.input[self.position] == ':':
            # 1.
            if self.position >= len(self.input) or self.input[self.position] != ':':
                return None
            self.position += 1

            # 2.
            string = self.collect_characters(lambda c: c in DIGITS)

            # 3.
            if len(string) != 2:
                return None

            # 4.
            value_3 = int(string)
        else:
            value_3 = value_2
            value_2 = value_1
            value_1 = 0

        # 13.
        if self.position >= len(self.input) or self.input[self.position] != '.':
            return None
        self.position += 1

        # 14.
        string = self.collect_characters(lambda c: c in DIGITS)

        # 15.
        if len(string) != 3:
            return None

        # 16.
        value_4 = int(string)

        # 17.
        if value_2 >= 59 or value_3 >= 59:
            return None

        # 18.
        result = value_1 * 60 * 60 + value_2 * 60 + value_3 + value_4 / 1000

        # 19.
        return result


def main(argv):
    files = [open(path, 'r') for path in argv[1:]]

    try:
        for file in files:
            parser = VTTParser(file.read())
            parser.parse()

            print("Results: {}".format(file))
            print("  Cues: {}".format(parser.text_tracks))
            print("  StyleSheets: {}".format(parser.stylesheets))
            print("  Regions: {}".format(parser.regions))
            print("  Errors: {}".format(parser.errors))
    finally:
        for file in files:
            file.close()

if __name__ == '__main__':
    import sys
    main(sys.argv);
