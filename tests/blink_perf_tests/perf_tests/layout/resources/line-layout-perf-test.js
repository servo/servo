'use strict';

class LineLayoutPerfTest {
  constructor(container) {
    this.container = container;
    this.spanCount = 0;
    this.lineCount = 10000;
    this.wordCountPerLine = 20;
    this.uniqueWordCount = 100;
    this.wordLength = 8;
    this.wordSeparator = ' ';
  }

  run(description) {
    // Unblock onload event by scheduling with zero delay,
    // in order to avoid perf bot flakiness where the bot checks
    // for timeouts reaching onload event, see crbug.com/457194
    window.setTimeout(() => {
      PerfTestRunner.measureTime({
        description: description,
        run: this.measure.bind(this)
      });
    }, 0);
  }

  measure() {
    var fragment = this.createFragment();
    PerfTestRunner.forceLayout();

    var now = PerfTestRunner.now();
    this.container.appendChild(fragment);
    PerfTestRunner.forceLayout();
    var resultTime = PerfTestRunner.now() - now;

    while (this.container.firstChild)
      this.container.removeChild(this.container.lastChild);
    return resultTime;
  }

  createFragment() {
    return TextGenerator.createFragment(this.spanCount, this.lineCount,
      this.wordCountPerLine, this.wordSeparator,
      this.createWordGenerator());
  }

  createWordGenerator() {
    return TextGenerator.createWordPoolGenerator(this.uniqueWordCount, this.wordLength);
  }
}

class LongWordPerfTest extends LineLayoutPerfTest {
  constructor(container, wordLength) {
    super(container);
    this.lineCount = 1;
    this.wordCountPerLine = 1;
    this.wordLength = wordLength;
  }

  createWordGenerator() {
    return () => TextGenerator.createWord(this.wordLength);
  }
}

class TextGenerator {
  static createFragment(spanCount, lineCount, wordCountPerLine, wordSeparator, nextWord) {
    if (spanCount <= 0)
      return document.createTextNode(TextGenerator.createLines(lineCount, wordCountPerLine, wordSeparator, nextWord));

    var fragment = document.createDocumentFragment();
    for (var elementIndex = 0; elementIndex < spanCount; elementIndex++) {
      var child = document.createElement('span');
      child.textContent = TextGenerator.createLines(lineCount, wordCountPerLine, wordSeparator, nextWord);
      fragment.appendChild(child);
    }
    return fragment;
  }

  static createLines(lineCount, wordCountPerLine, wordSeparator, nextWord) {
    var lines = [];
    for (var lineIndex = 0; lineIndex < lineCount; lineIndex++)
      lines.push(this.createLine(wordCountPerLine, wordSeparator, nextWord));
    return lines.join('\n');
  }

  static createLine(wordCountPerLine, wordSeparator, nextWord) {
    let words = [];
    for (var wordIndex = 0; wordIndex < wordCountPerLine; wordIndex++)
      words.push(nextWord());
    return words.join(wordSeparator);
  }

  static createWordPoolGenerator(wordCount, wordLength) {
    var words = [];
    for (var i = 0; i < wordCount; i++)
      words.push(TextGenerator.createWord(wordLength));
    return () => {
      return words[Math.floor(Math.random() * words.length)];
    };
  }

  static createWord(length) {
    var pieces = [];
    while (length > 0) {
      var piece = Math.random().toString(36).slice(2);
      pieces.push(piece.slice(0, length));
      length -= piece.length;
    }
    return pieces.join('');
  }
}
