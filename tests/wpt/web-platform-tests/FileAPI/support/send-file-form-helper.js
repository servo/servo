'use strict';

// Rationale for this particular test character sequence, which is
// used in filenames and also in file contents:
//
// - ABC~ ensures the string starts with something we can read to
//   ensure it is from the correct source; ~ is used because even
//   some 1-byte otherwise-ASCII-like parts of ISO-2022-JP
//   interpret it differently.
// - ‾¥ are inside a single-byte range of ISO-2022-JP and help
//   diagnose problems due to filesystem encoding or locale
// - ≈ is inside IBM437 and helps diagnose problems due to filesystem
//   encoding or locale
// - ¤ is inside Latin-1 and helps diagnose problems due to
//   filesystem encoding or locale; it is also the "simplest" case
//   needing substitution in ISO-2022-JP
// - ･ is inside a single-byte range of ISO-2022-JP in some variants
//   and helps diagnose problems due to filesystem encoding or locale;
//   on the web it is distinct when decoding but unified when encoding
// - ・ is inside a double-byte range of ISO-2022-JP and helps
//   diagnose problems due to filesystem encoding or locale
// - • is inside Windows-1252 and helps diagnose problems due to
//   filesystem encoding or locale and also ensures these aren't
//   accidentally turned into e.g. control codes
// - ∙ is inside IBM437 and helps diagnose problems due to filesystem
//   encoding or locale
// - · is inside Latin-1 and helps diagnose problems due to
//   filesystem encoding or locale and also ensures HTML named
//   character references (e.g. &middot;) are not used
// - ☼ is inside IBM437 shadowing C0 and helps diagnose problems due to
//   filesystem encoding or locale and also ensures these aren't
//   accidentally turned into e.g. control codes
// - ★ is inside ISO-2022-JP on a non-Kanji page and makes correct
//   output easier to spot
// - 星 is inside ISO-2022-JP on a Kanji page and makes correct
//   output easier to spot
// - 🌟 is outside the BMP and makes incorrect surrogate pair
//   substitution detectable and ensures substitutions work
//   correctly immediately after Kanji 2-byte ISO-2022-JP
// - 星 repeated here ensures the correct codec state is used
//   after a non-BMP substitution
// - ★ repeated here also makes correct output easier to spot
// - ☼ is inside IBM437 shadowing C0 and helps diagnose problems due to
//   filesystem encoding or locale and also ensures these aren't
//   accidentally turned into e.g. control codes and also ensures
//   substitutions work correctly immediately after non-Kanji
//   2-byte ISO-2022-JP
// - · is inside Latin-1 and helps diagnose problems due to
//   filesystem encoding or locale and also ensures HTML named
//   character references (e.g. &middot;) are not used
// - ∙ is inside IBM437 and helps diagnose problems due to filesystem
//   encoding or locale
// - • is inside Windows-1252 and again helps diagnose problems
//   due to filesystem encoding or locale
// - ・ is inside a double-byte range of ISO-2022-JP and helps
//   diagnose problems due to filesystem encoding or locale
// - ･ is inside a single-byte range of ISO-2022-JP in some variants
//   and helps diagnose problems due to filesystem encoding or locale;
//   on the web it is distinct when decoding but unified when encoding
// - ¤ is inside Latin-1 and helps diagnose problems due to
//   filesystem encoding or locale; again it is a "simple"
//   substitution case
// - ≈ is inside IBM437 and helps diagnose problems due to filesystem
//   encoding or locale
// - ¥‾ are inside a single-byte range of ISO-2022-JP and help
//   diagnose problems due to filesystem encoding or locale
// - ~XYZ ensures earlier errors don't lead to misencoding of
//   simple ASCII
//
// Overall the near-symmetry makes common I18N mistakes like
// off-by-1-after-non-BMP easier to spot. All the characters
// are also allowed in Windows Unicode filenames.
const kTestChars = 'ABC~‾¥≈¤･・•∙·☼★星🌟星★☼·∙•・･¤≈¥‾~XYZ';

// NOTE: The expected interpretation of ISO-2022-JP according to
// https://encoding.spec.whatwg.org/#iso-2022-jp-encoder unifies
// single-byte and double-byte katakana.
const kTestFallbackIso2022jp =
      ('ABC~\x1B(J~\\≈¤\x1B$B!&!&\x1B(B•∙·☼\x1B$B!z@1\x1B(B🌟' +
       '\x1B$B@1!z\x1B(B☼·∙•\x1B$B!&!&\x1B(B¤≈\x1B(J\\~\x1B(B~XYZ').replace(
             /[^\0-\x7F]/gu,
           x => `&#${x.codePointAt(0)};`);

// NOTE: \uFFFD is used here to replace Windows-1252 bytes to match
// how we will see them in the reflected POST bytes in a frame using
// UTF-8 byte interpretation. The bytes will actually be intact, but
// this code cannot tell and does not really care.
const kTestFallbackWindows1252 =
      'ABC~‾\xA5≈\xA4･・\x95∙\xB7☼★星🌟星★☼\xB7∙\x95・･\xA4≈\xA5‾~XYZ'.replace(
            /[^\0-\xFF]/gu,
          x => `&#${x.codePointAt(0)};`).replace(/[\x80-\xFF]/g, '\uFFFD');

const kTestFallbackXUserDefined =
      kTestChars.replace(/[^\0-\x7F]/gu, x => `&#${x.codePointAt(0)};`);

// formPostFileUploadTest - verifies multipart upload structure and
// numeric character reference replacement for filenames, field names,
// and field values.
//
// Uses /fetch/api/resources/echo-content.py to echo the upload
// POST with UTF-8 byte interpretation, leading to the "UTF-8 goggles"
// behavior documented below for expectedEncodedBaseName when non-
// UTF-8-compatible byte sequences appear in the formEncoding-encoded
// uploaded data.
//
// Fields in the parameter object:
//
// - fileNameSource: purely explanatory and gives a clue about which
//   character encoding is the source for the non-7-bit-ASCII parts of
//   the fileBaseName, or Unicode if no smaller-than-Unicode source
//   contains all the characters. Used in the test name.
// - fileBaseName: the not-necessarily-just-7-bit-ASCII file basename
//   used for the constructed test file. Used in the test name.
// - formEncoding: the acceptCharset of the form used to submit the
//   test file. Used in the test name.
// - expectedEncodedBaseName: the expected formEncoding-encoded
//   version of fileBaseName with unencodable characters replaced by
//   numeric character references and non-7-bit-ASCII bytes seen
//   through UTF-8 goggles; subsequences not interpretable as UTF-8
//   have each byte represented here by \uFFFD REPLACEMENT CHARACTER.
const formPostFileUploadTest = ({
  fileNameSource,
  fileBaseName,
  formEncoding,
  expectedEncodedBaseName,
}) => {
  promise_test(async testCase => {

    if (document.readyState !== 'complete') {
      await new Promise(resolve => addEventListener('load', resolve));
    }

    const formTargetFrame = Object.assign(document.createElement('iframe'), {
      name: 'formtargetframe',
    });
    document.body.append(formTargetFrame);
    testCase.add_cleanup(() => {
      document.body.removeChild(formTargetFrame);
    });

    const form = Object.assign(document.createElement('form'), {
      acceptCharset: formEncoding,
      action: '/fetch/api/resources/echo-content.py',
      method: 'POST',
      enctype: 'multipart/form-data',
      target: formTargetFrame.name,
    });
    document.body.append(form);
    testCase.add_cleanup(() => {
      document.body.removeChild(form);
    });

    // Used to verify that the browser agrees with the test about
    // which form charset is used.
    form.append(Object.assign(document.createElement('input'), {
      type: 'hidden',
      name: '_charset_',
    }));

    // Used to verify that the browser agrees with the test about
    // field value replacement and encoding independently of file system
    // idiosyncracies.
    form.append(Object.assign(document.createElement('input'), {
      type: 'hidden',
      name: 'filename',
      value: fileBaseName,
    }));

    // Same, but with name and value reversed to ensure field names
    // get the same treatment.
    form.append(Object.assign(document.createElement('input'), {
      type: 'hidden',
      name: fileBaseName,
      value: 'filename',
    }));

    const fileInput = Object.assign(document.createElement('input'), {
      type: 'file',
      name: 'file',
    });
    form.append(fileInput);

    // Removes c:\fakepath\ or other pseudofolder and returns just the
    // final component of filePath; allows both / and \ as segment
    // delimiters.
    const baseNameOfFilePath = filePath => filePath.split(/[\/\\]/).pop();
    await new Promise(resolve => {
      const dataTransfer = new DataTransfer;
      dataTransfer.items.add(
          new File([kTestChars], fileBaseName, {type: 'text/plain'}));
      fileInput.files = dataTransfer.files;
      // For historical reasons .value will be prefixed with
      // c:\fakepath\, but the basename should match the file name
      // exposed through the newer .files[0].name API. This check
      // verifies that assumption.
      assert_equals(
          fileInput.files[0].name,
          baseNameOfFilePath(fileInput.value),
          `The basename of the field's value should match its files[0].name`);
      form.submit();
      formTargetFrame.onload = resolve;
    });

    const formDataText = formTargetFrame.contentDocument.body.textContent;
    const formDataLines = formDataText.split('\n');
    if (formDataLines.length && !formDataLines[formDataLines.length - 1]) {
      --formDataLines.length;
    }
    assert_greater_than(
        formDataLines.length,
        2,
        `${fileBaseName}: multipart form data must have at least 3 lines: ${
             JSON.stringify(formDataText)
           }`);
    const boundary = formDataLines[0];
    assert_equals(
        formDataLines[formDataLines.length - 1],
        boundary + '--',
        `${fileBaseName}: multipart form data must end with ${boundary}--: ${
             JSON.stringify(formDataText)
           }`);
    const expectedText = [
      boundary,
      'Content-Disposition: form-data; name="_charset_"',
      '',
      formEncoding,
      boundary,
      'Content-Disposition: form-data; name="filename"',
      '',
      expectedEncodedBaseName,
      boundary,
      `Content-Disposition: form-data; name="${expectedEncodedBaseName}"`,
      '',
      'filename',
      boundary,
      `Content-Disposition: form-data; name="file"; ` +
          `filename="${expectedEncodedBaseName}"`,
      'Content-Type: text/plain',
      '',
      kTestChars,
      boundary + '--',
    ].join('\n');
    assert_true(
        formDataText.startsWith(expectedText),
        `Unexpected multipart-shaped form data received:\n${
             formDataText
           }\nExpected:\n${expectedText}`);
  }, `Upload ${fileBaseName} (${fileNameSource}) in ${formEncoding} form`);
};
