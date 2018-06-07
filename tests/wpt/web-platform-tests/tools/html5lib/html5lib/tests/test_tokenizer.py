from __future__ import absolute_import, division, unicode_literals

import codecs
import json
import warnings
import re

import pytest
from six import unichr

from html5lib._tokenizer import HTMLTokenizer
from html5lib import constants, _utils


class TokenizerTestParser(object):
    def __init__(self, initialState, lastStartTag=None):
        self.tokenizer = HTMLTokenizer
        self._state = initialState
        self._lastStartTag = lastStartTag

    def parse(self, stream, encoding=None, innerHTML=False):
        # pylint:disable=unused-argument
        tokenizer = self.tokenizer(stream, encoding)
        self.outputTokens = []

        tokenizer.state = getattr(tokenizer, self._state)
        if self._lastStartTag is not None:
            tokenizer.currentToken = {"type": "startTag",
                                      "name": self._lastStartTag}

        types = dict((v, k) for k, v in constants.tokenTypes.items())
        for token in tokenizer:
            getattr(self, 'process%s' % types[token["type"]])(token)

        return self.outputTokens

    def processDoctype(self, token):
        self.outputTokens.append(["DOCTYPE", token["name"], token["publicId"],
                                  token["systemId"], token["correct"]])

    def processStartTag(self, token):
        self.outputTokens.append(["StartTag", token["name"],
                                  dict(token["data"][::-1]), token["selfClosing"]])

    def processEmptyTag(self, token):
        if token["name"] not in constants.voidElements:
            self.outputTokens.append("ParseError")
        self.outputTokens.append(["StartTag", token["name"], dict(token["data"][::-1])])

    def processEndTag(self, token):
        self.outputTokens.append(["EndTag", token["name"],
                                  token["selfClosing"]])

    def processComment(self, token):
        self.outputTokens.append(["Comment", token["data"]])

    def processSpaceCharacters(self, token):
        self.outputTokens.append(["Character", token["data"]])
        self.processSpaceCharacters = self.processCharacters

    def processCharacters(self, token):
        self.outputTokens.append(["Character", token["data"]])

    def processEOF(self, token):
        pass

    def processParseError(self, token):
        self.outputTokens.append(["ParseError", token["data"]])


def concatenateCharacterTokens(tokens):
    outputTokens = []
    for token in tokens:
        if "ParseError" not in token and token[0] == "Character":
            if (outputTokens and "ParseError" not in outputTokens[-1] and
                    outputTokens[-1][0] == "Character"):
                outputTokens[-1][1] += token[1]
            else:
                outputTokens.append(token)
        else:
            outputTokens.append(token)
    return outputTokens


def normalizeTokens(tokens):
    # TODO: convert tests to reflect arrays
    for i, token in enumerate(tokens):
        if token[0] == 'ParseError':
            tokens[i] = token[0]
    return tokens


def tokensMatch(expectedTokens, receivedTokens, ignoreErrorOrder,
                ignoreErrors=False):
    """Test whether the test has passed or failed

    If the ignoreErrorOrder flag is set to true we don't test the relative
    positions of parse errors and non parse errors
    """
    checkSelfClosing = False
    for token in expectedTokens:
        if (token[0] == "StartTag" and len(token) == 4 or
                token[0] == "EndTag" and len(token) == 3):
            checkSelfClosing = True
            break

    if not checkSelfClosing:
        for token in receivedTokens:
            if token[0] == "StartTag" or token[0] == "EndTag":
                token.pop()

    if not ignoreErrorOrder and not ignoreErrors:
        expectedTokens = concatenateCharacterTokens(expectedTokens)
        return expectedTokens == receivedTokens
    else:
        # Sort the tokens into two groups; non-parse errors and parse errors
        tokens = {"expected": [[], []], "received": [[], []]}
        for tokenType, tokenList in zip(list(tokens.keys()),
                                        (expectedTokens, receivedTokens)):
            for token in tokenList:
                if token != "ParseError":
                    tokens[tokenType][0].append(token)
                else:
                    if not ignoreErrors:
                        tokens[tokenType][1].append(token)
            tokens[tokenType][0] = concatenateCharacterTokens(tokens[tokenType][0])
        return tokens["expected"] == tokens["received"]


_surrogateRe = re.compile(r"\\u([0-9A-Fa-f]{4})(?:\\u([0-9A-Fa-f]{4}))?")


def unescape(test):
    def decode(inp):
        """Decode \\uXXXX escapes

        This decodes \\uXXXX escapes, possibly into non-BMP characters when
        two surrogate character escapes are adjacent to each other.
        """
        # This cannot be implemented using the unicode_escape codec
        # because that requires its input be ISO-8859-1, and we need
        # arbitrary unicode as input.
        def repl(m):
            if m.group(2) is not None:
                high = int(m.group(1), 16)
                low = int(m.group(2), 16)
                if 0xD800 <= high <= 0xDBFF and 0xDC00 <= low <= 0xDFFF:
                    cp = ((high - 0xD800) << 10) + (low - 0xDC00) + 0x10000
                    return unichr(cp)
                else:
                    return unichr(high) + unichr(low)
            else:
                return unichr(int(m.group(1), 16))
        try:
            return _surrogateRe.sub(repl, inp)
        except ValueError:
            # This occurs when unichr throws ValueError, which should
            # only be for a lone-surrogate.
            if _utils.supports_lone_surrogates:
                raise
            return None

    test["input"] = decode(test["input"])
    for token in test["output"]:
        if token == "ParseError":
            continue
        else:
            token[1] = decode(token[1])
            if len(token) > 2:
                for key, value in token[2]:
                    del token[2][key]
                    token[2][decode(key)] = decode(value)
    return test


def _doCapitalize(match):
    return match.group(1).upper()

_capitalizeRe = re.compile(r"\W+(\w)").sub


def capitalize(s):
    s = s.lower()
    s = _capitalizeRe(_doCapitalize, s)
    return s


class TokenizerFile(pytest.File):
    def collect(self):
        with codecs.open(str(self.fspath), "r", encoding="utf-8") as fp:
            tests = json.load(fp)
        if 'tests' in tests:
            for i, test in enumerate(tests['tests']):
                yield TokenizerTestCollector(str(i), self, testdata=test)


class TokenizerTestCollector(pytest.Collector):
    def __init__(self, name, parent=None, config=None, session=None, testdata=None):
        super(TokenizerTestCollector, self).__init__(name, parent, config, session)
        if 'initialStates' not in testdata:
            testdata["initialStates"] = ["Data state"]
        if 'doubleEscaped' in testdata:
            testdata = unescape(testdata)
        self.testdata = testdata

    def collect(self):
        for initialState in self.testdata["initialStates"]:
            initialState = capitalize(initialState)
            item = TokenizerTest(initialState,
                                 self,
                                 self.testdata,
                                 initialState)
            if self.testdata["input"] is None:
                item.add_marker(pytest.mark.skipif(True, reason="Relies on lone surrogates"))
            yield item


class TokenizerTest(pytest.Item):
    def __init__(self, name, parent, test, initialState):
        super(TokenizerTest, self).__init__(name, parent)
        self.obj = lambda: 1  # this is to hack around skipif needing a function!
        self.test = test
        self.initialState = initialState

    def runtest(self):
        warnings.resetwarnings()
        warnings.simplefilter("error")

        expected = self.test['output']
        if 'lastStartTag' not in self.test:
            self.test['lastStartTag'] = None
        parser = TokenizerTestParser(self.initialState,
                                     self.test['lastStartTag'])
        tokens = parser.parse(self.test['input'])
        received = normalizeTokens(tokens)
        errorMsg = "\n".join(["\n\nInitial state:",
                              self.initialState,
                              "\nInput:", self.test['input'],
                              "\nExpected:", repr(expected),
                              "\nreceived:", repr(tokens)])
        errorMsg = errorMsg
        ignoreErrorOrder = self.test.get('ignoreErrorOrder', False)
        assert tokensMatch(expected, received, ignoreErrorOrder, True), errorMsg

    def repr_failure(self, excinfo):
        traceback = excinfo.traceback
        ntraceback = traceback.cut(path=__file__)
        excinfo.traceback = ntraceback.filter()

        return excinfo.getrepr(funcargs=True,
                               showlocals=False,
                               style="short", tbfilter=False)
