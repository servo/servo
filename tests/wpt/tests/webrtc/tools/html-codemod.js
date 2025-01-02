/*
 * extract script content from a series of html files, run a
 * jscodeshift codemod on them and overwrite the original file.
 *
 * Usage: node html-codemod.js codemod-file list of files to process
 */
const { JSDOM } = require('jsdom');
const fs = require('fs');
const {execFileSync} = require('child_process');

const codemod = process.argv[2];
const filenames = process.argv.slice(3);
filenames.forEach((filename) => {
    const originalContent = fs.readFileSync(filename, 'utf-8');
    const dom = new JSDOM(originalContent);
    const document = dom.window.document;
    const scriptTags = document.querySelectorAll('script');
    const lastTag = scriptTags[scriptTags.length - 1];
    const script = lastTag.innerHTML;
    if (!script) {
        console.log('NO SCRIPT FOUND', filename);
        return;
    }
    const scriptFilename = filename + '.codemod.js';
    const scriptFile = fs.writeFileSync(scriptFilename, script);
    // exec jscodeshift
    const output = execFileSync('./node_modules/.bin/jscodeshift', ['-t', codemod, scriptFilename]);
    console.log(filename, output.toString()); // output jscodeshift output.
    // read back file, resubstitute
    const newScript = fs.readFileSync(scriptFilename, 'utf-8').toString();
    const modifiedContent = originalContent.split(script).join(newScript);
    fs.writeFileSync(filename, modifiedContent);
    fs.unlinkSync(scriptFilename);
});
