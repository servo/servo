const HtmlAamAccnameUtils = {
  /*
  Builds a collection of WPT subtests for verifying the computation steps
  documented in https://www.w3.org/TR/html-aam-1.0/#accname-computation.

  For each element in elements[], a collection of subtests will be generated to
  test every potential name source in nameSources[].

  Each entry in elements[] must have a key that is a valid CSS selector
  conforming to either:

  - `tagname`
  - `tagname[attrname="attrvalue"]`

  This key will be parsed to create the DOM element which will be used to
  test the computed name.

  Each entry in elements[] may have an optional properties object specifying
  any of the following:

  - descendants[]: elements to be appended as descendants of the test element.
    This array may include the string 'textNode' to append a text node.
  - ancestors[]: elements to be prepended as ancestors of the test element.
  - attrs[]: attributes to be added to the element for every subtest. For
    example, an `a` element may have an href attribute.
  - specialContext: a string indicating a special case that requires complex
    test markup. Currently supported values are 'area', 'ruby', 'rp', and 'rt'.

  nameSources[] is necessarily order-sensitive, matching the computation steps
  in HTML-AAM. Each string in nameSources[] is a unique identifier and should
  have a matching key in the nameValues{} dictionary later in this file.

  The first subtest generated will include *every* potential name source and
  verify that the first source “wins”. Each subsequent subtest eliminates the
  name source from the previous test, then performs the same verification.
  In the final subtest, all potential name sources have been eliminated and the
  computed name is expected to be empty.

  The following example test configuration will generate 24 WPT subtests:
  3 elements * 8 potential name sources = 24 subtests

  Example:
    const testConfig = {
      elements: {
        'input[type="text"]': { attrs: ['value'] },
        'input[type="search"]': { attrs: ['value'] },
        'textarea': { descendants: ['textNode'] }
      },
      nameSources: [
        'from 2 aria-labelledby refs',
        'from 1 aria-labelledby ref',
        'from aria-label',
        'from 2 labels',
        'from 1 label',
        'from title',
        'from placeholder',
        'to be empty'
      ]
    }
    HtmlAamAccnameUtils.buildNameComputationTests(testConfig);

  */
  buildNameComputationTests: function (testConfig) {
    const elBody = document.querySelector('body');
    this.buildPageHeading(testConfig);
    Object.entries(testConfig.elements).forEach(([selector, properties]) => {
      let nameSources = [...testConfig.nameSources];
      while (nameSources.length > 0) {
        let html = this.buildNameComputationTest(selector, nameSources, properties);
        elBody.insertAdjacentHTML('beforeend', html);
        this.previewTestMarkup(html);
        nameSources.shift();
      }
    });
  },

  /*
  Builds a complete markup block for an individual subtest. Depending on the
  subtest being permuted (see above), the markup may include some  “preamble”
  on which the test element itself depends.

  For example, a subtest may be checking the computed name of an
  <input type="text"> element that depends on a <label> in the preamble.

  */
  buildNameComputationTest: function (selector, nameSources, properties) {
    const id = this.getUniqueTestId();
    const { element, attrName, attrValue } = this.parseSelector(selector);
    let nameTest = `${selector} expecting name ${nameSources[0]}`;
    let htmlPreamble = '';
    let tmpSource = '';
    let expSource = nameSources[0];
    let expName = '';
    let elReturn = null;

    // Generate test case preamble and supporting elements
    htmlPreamble += `<h2>${nameTest}</h2>`;

    // Create a test element based on the selector and any additional properties
    let elTest = document.createElement(element);
    if (attrName) {
      elTest.setAttribute(attrName, attrValue);
    }

    elTest.id = `el-${id}`;
    elTest.className = 'ex-label';
    elTest.setAttribute('data-testname', nameTest);

    // Add any attributes that should always be present in the test element
    if (properties?.attrs?.includes('value')) {
      elTest.setAttribute('value', elTest.tagName.toLowerCase() + ' value attribute value');
    }
    if (properties?.attrs?.includes('href')) {
      elTest.setAttribute('href', '#');
    }
    if (properties?.attrs?.includes('img src')) {
      elTest.src = this.srcImgSample;
    }

    // If the element desires DOM ancestors, build them.
    if (properties?.ancestors) {
      elReturn = properties.ancestors.reduceRight((descendant, tag) => {
        let elAncestor = document.createElement(tag);
        elAncestor.appendChild(descendant);
        return elAncestor;
      }, elTest);
    }

    // If the element desires DOM descendants, build them.
    if (properties?.descendants) {
      let tagPrevious = element;
      properties.descendants.reduce((ancestor, tag) => {
        let elDescendant = tag === 'textNode' ?
          document.createTextNode(`${tagPrevious} text node contents`) :
          document.createElement(tag);
        ancestor.appendChild(elDescendant);
        tagPrevious = tag;
        return elDescendant;
      }, elTest);
    }

    if (properties?.specialContext) {
      elReturn = this.handleSpecialCases(elTest, properties.specialContext, id);
    }

    // If the test element needed no ancestor(s), return it
    if (!elReturn) {
      elReturn = elTest;
    }

    // Build ancestor-based name source
    tmpSource = 'from ancestor figure figcaption';
    if (expSource === tmpSource) {
      let elFigure = document.createElement('figure');
      let elFigcaption = document.createElement('figcaption');
      elFigcaption.appendChild(document.createTextNode(this.nameValues[tmpSource]));
      elFigure.appendChild(elReturn);
      elFigure.appendChild(elFigcaption);
      elReturn = elFigure;
      expName = this.nameValues[tmpSource];
    }

    // Build encapsulating label source, wrapping the test element in a label
    tmpSource = 'from encapsulating label';
    if (expSource === tmpSource) {
      let elLabel = document.createElement('label');
      elLabel.appendChild(document.createTextNode(this.nameValues[tmpSource]));
      elLabel.appendChild(elReturn);
      elReturn = elLabel;
      expName = this.nameValues[tmpSource];
    }

    // Build label source reference(s)
    tmpSource = 'from 1 label';
    if (nameSources.includes(tmpSource)) {
      htmlPreamble += `<label for="el-${id}">${this.nameValues[tmpSource]}</label>`;
      if (expSource === tmpSource) expName = this.nameValues[tmpSource];
    }

    tmpSource = 'from 2 labels';
    if (nameSources.includes(tmpSource)) {
      htmlPreamble += `<label for="el-${id}">${this.nameValues[tmpSource]}</label>`;
      if (expSource === tmpSource) expName = this.nameValues['from 1 label'] + ' ' + this.nameValues[tmpSource];
    }

    // Build aria-labelledby source reference(s)
    tmpSource = 'from 1 aria-labelledby ref';
    if (nameSources.includes(tmpSource)) {
      htmlPreamble += `<p id="el-${id}-label-1">${this.nameValues[tmpSource]}</p>`;
      elTest.setAttribute('aria-labelledby', `el-${id}-label-1`);
      if (expSource === tmpSource) expName = this.nameValues[tmpSource];
    }

    tmpSource = 'from 2 aria-labelledby refs';
    if (nameSources.includes(tmpSource)) {
      htmlPreamble += `<p id="el-${id}-label-2">${this.nameValues[tmpSource]}</p>`;
      elTest.setAttribute('aria-labelledby', `el-${id}-label-1 el-${id}-label-2`);
      if (expSource === tmpSource) expName = this.nameValues['from 1 aria-labelledby ref'] + ' ' + this.nameValues[tmpSource];
    }

    // Build aria-labelledby source reference(s)
    tmpSource = 'from subtree';
    if (nameSources.includes(tmpSource)) {
      let name = elTest.tagName.toLowerCase() + ' ' + this.nameValues[tmpSource];
      elTest.appendChild(document.createTextNode(name));
      if (expSource === tmpSource) expName = name;
    }

    // Build child-based name sources
    [
      'from first legend',
      'from first caption'
    ].forEach(tmpSource => {
      if (nameSources.includes(tmpSource)) {
        let child = tmpSource.replace('from first ', '');
        elTest.insertAdjacentHTML('afterbegin', `<${child}>Duplicate ${this.nameValues[tmpSource]}</${child}>`);
        elTest.insertAdjacentHTML('afterbegin', `<${child}>${this.nameValues[tmpSource]}</${child}>`);
        if (expSource === tmpSource) expName = this.nameValues[tmpSource];
      }
    });

    // Build attribute-based name sources
    [
      'from aria-label',
      'from aria-placeholder',
      'from value',
      'from alt',
      'from title',
      'from placeholder',
      'from label attribute'
    ].forEach(tmpSource => {
      if (nameSources.includes(tmpSource)) {
        let attr = tmpSource.replace('from ', '').replace(' attribute', '');
        let name = tmpSource === 'from alt' ?
          this.nameValues[tmpSource] :
          elTest.tagName.toLowerCase() + ' ' + this.nameValues[tmpSource];
        elTest.setAttribute(attr, name);
        if (expSource === tmpSource) expName = name;
      }
    });

    // Build empty alt attribute name source
    tmpSource = 'from empty alt';
    if (expSource === tmpSource) {
      elTest.setAttribute('alt', this.nameValues[tmpSource]);
      expName = this.nameValues[tmpSource];
    }

    // If the test element is empty but can contain descendants, add an
    // unnamed image element to render something that is visible.
    if (!elTest.hasChildNodes() && this.canContainDescendants(elTest)) {
      let elImg = document.createElement('img');
      elImg.src = this.srcImgSample;
      elTest.appendChild(elImg);
    }

    elTest.setAttribute('data-expectedlabel', expName);

    return htmlPreamble + elReturn.outerHTML;
  },

  /*
    Some subtests require complex markup context that is not easily broken
    down into configuration primitives. This function handles them.
  */

  handleSpecialCases: function (elTest, specialContext, id) {
    let elReturn = null;

    if (specialContext === 'area') {
      let elMap = document.createElement('map');
      let elImg = document.createElement('img');
      elMap.setAttribute('name', `map-${id}`);
      elMap.appendChild(elTest);
      elImg.setAttribute('usemap', `#map-${id}`);
      elImg.src = this.srcImgSample;
      elTest.setAttribute('alt', 'Mapped image alt attribute value');
      elTest.setAttribute('shape', 'rect');
      elTest.setAttribute('coords', '0,0,20,20');
      elReturn = document.createElement('div');
      elReturn.appendChild(elMap)
      elReturn.appendChild(elImg);
    }

    else if (specialContext === 'ruby') {
      elTest.appendChild(document.createTextNode('ruby text node contents'));
      elTest.appendChild(document.createElement('rp')).appendChild(document.createTextNode('rp #1'));
      elTest.appendChild(document.createElement('rt')).appendChild(document.createTextNode('rt'));
      elTest.appendChild(document.createElement('rp')).appendChild(document.createTextNode('rp #2'));
    }

    else if (specialContext === 'rp') {
      let elRuby = document.createElement('ruby');
      elRuby.appendChild(document.createTextNode('ruby text node contents'));
      elRuby.appendChild(elTest);
      elTest.appendChild(document.createTextNode('rp #1'));
      elRuby.appendChild(document.createElement('rt')).appendChild(document.createTextNode('rt'));
      elRuby.appendChild(document.createElement('rp')).appendChild(document.createTextNode('rp #2'));
      elReturn = elRuby;
    }

    else if (specialContext === 'rt') {
      let elRuby = document.createElement('ruby');
      elRuby.appendChild(document.createTextNode('ruby text node contents'));
      elRuby.appendChild(document.createElement('rp')).appendChild(document.createTextNode('rp #1'));
      elRuby.appendChild(elTest);
      elTest.appendChild(document.createTextNode('rt'));
      elRuby.appendChild(document.createElement('rp')).appendChild(document.createTextNode('rp #2'));
      elReturn = elRuby;
    }

    return elReturn;
  },

  getUniqueTestId: function () {
    return this.testId += 1;
  },

  parseSelector: function (selector) {
    const regex = /^(\w+)(?:\[(\w+)="([\w-]+)"\])?$/;
    const [_, element, attrName = null, attrValue = null] = selector.match(regex);
    return { element, attrName, attrValue };
  },

  buildPageHeading: function (testConfig) {
    const elBody = document.querySelector('body');

    if (!document.querySelector('h1')) {
      const elH1 = document.createElement('h1');
      elH1.textContent = document.title;
      elBody.appendChild(elH1);
    }

    if (testConfig.urlSpec) {
      elBody.insertAdjacentHTML('beforeend', `<p><a href="${testConfig.urlSpec}">HTML-AAM computation steps</a></p>`);
    }
  },

  previewTestMarkup: function (html) {
    const elBody = document.querySelector('body');
    let elTestMarkup = document.createElement('pre');
    elTestMarkup.style.border = '1px solid green';
    elTestMarkup.style.backgroundColor = 'palegreen';
    elTestMarkup.style.padding = '1rem';
    elTestMarkup.style.whiteSpace = 'pre-wrap';
    elTestMarkup.style.overflow = 'auto';
    elTestMarkup.appendChild(document.createTextNode(html.replaceAll('><', '>\n<').trim()));
    elBody.appendChild(elTestMarkup);
  },

  canContainDescendants: function (element) {
    const voidElements = new Set(['AREA', 'BASE', 'BR', 'COL', 'EMBED', 'HR', 'IMG', 'INPUT', 'LINK', 'META', 'PARAM', 'SOURCE', 'TRACK', 'WBR']);
    return !voidElements.has(element.nodeName);
  },

  testId: 0,
  srcImgSample: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAyCAMAAAAp4XiDAAAAP1BMVEWqAKoAAACrAKuqAKqrAKuvAK+vAK+nAKepAKmsAKyrAKutAK2oAKixALGqAKqqAKqqAKqqAKqqAKqqAKqqAKqx+4SNAAAAFHRSTlMzAL9gQBAgIKCfKxQZBu/fr4BwUHCcwvYAAAEASURBVEjHvdbLEoMgDAVQbkKg9f3I/39rpdiZutHAgrtywZkxESEOxTmJCLwqA6SK9PgCXqoeUCWAVR3kQvZpRbwnETzzHyFd8BhS+iNxG4fO3aYLI8cfYQronSE9Au1fMisnYTGbTnIQYRqdMVhWybVEKxlyLS6gs5IOwR9EFc4cqNYRZjvJqwGUEKCOEJUQojYda1a+9yXEt9sw77edpNVtOlbxYm06VvEp23SsTfkVP3Kb8iuOvnYbpuyycLn8wUpCLl/WBVYyEksik269TQysMw6CnaL1Eo/EiaQEHuPTqDBgC5eBhPCY5TKQ8MzPY8867SfJEbh74iGCkxTnA3SZDugOkmV1AAAAAElFTkSuQmCC",
  nameValues: {
    'from 1 aria-labelledby ref': 'aria-labelledby ref element #1 text contents',
    'from 2 aria-labelledby refs': 'aria-labelledby ref element #2 text contents',
    'from aria-label': 'aria-label attribute value',
    'from 1 label': 'associated label #1 text contents',
    'from 2 labels': 'associated label #2 text contents',
    'from subtree': 'subtree text contents',
    'from encapsulating label': 'encapsulating label text contents',
    'from first legend': 'legend text contents',
    'from first caption': 'caption text contents',
    'from value': 'value attribute value',
    'from label attribute': 'label attribute value',
    'from alt': 'alt attribute value',
    'from empty alt': '',
    'from title': 'title attribute value',
    'from placeholder': 'placeholder attribute value',
    'from aria-placeholder': 'aria-placeholder attribute value',
    'from ancestor figure figcaption': 'ancestor figure figcaption text contents',
  }
};
