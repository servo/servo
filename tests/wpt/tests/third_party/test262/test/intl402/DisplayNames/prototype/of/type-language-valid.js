// Copyright (C) 2023 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-Intl.DisplayNames.prototype.of
description: Returns string value for valid `language` codes
features: [Intl.DisplayNames]
---*/


/*
unicode_language_id = "root"    // not allowed in ES
                      | (unicode_language_subtag
                        (sep unicode_script_subtag)?
                      | unicode_script_subtag)
                      (sep unicode_region_subtag)?
                      (sep unicode_variant_subtag)*
 */

// unicode_language_subtag = alpha{2,3} | alpha{5,8} ;

var languages = [ { subtag: 'ab', description: '2 letter language_subtag' }, 
                  { subtag: 'cde', description: '3 letter language_subtag' },
                  { subtag: 'zzzzzzzz', description: '8 letter language_subtag'} ];

// unicode_script_subtag = alpha{4} ;
var scripts = [  {subtag: 'abcd', description: '4 letter script_subtag' },
                 {subtag: '', description: ''} ];

// unicode_region_subtag = (alpha{2} | digit{3}) ;
var regions = [ {subtag: 'ab', description: '2 letter region_subtag' },
                {subtag: '123', description: '3 digit region_subtag'},
                {subtag: '', description: ''} ];

// unicode_variant_subtag = (alphanum{5, 8} | digit alphanum{3}

var variants = [  {subtag: 'abcde', description: '5 letter variant_subtag'}, 
                  {subtag: 'fghijklm', description: '8 letter variant_subtag'}, 
                  {subtag: '12345', description: '5 digit variant_subtag'},
                  {subtag: '1nopq', description: '5 chararcter leading digit variant_subtag'},
                  {subtag: '12345678', description: '8 digit variant_subtag'}, 
                  {subtag: 'a2345678', description: '8 character trailing digit variant_subtag'}, 
                  {subtag: '1abc', description: 'leading digit 4 character variant_subtag' },
                  {subtag: '2345', description: '4 digit variant_subtag'},
                  {subtag: '6d7e', description: 'leading digit 4 character mixed alphanum variant_subtag'},
                  {subtag: '', description: ''} ];


function notEmpty(subtag) {
  return subtag !== '';
}

var displayNames = new Intl.DisplayNames(undefined, {type: 'language'});

for (var l in languages) {
  for (var s in scripts) {
    for (var r in regions) {
      for (var v in variants) {
        var languageTag = [languages[l].subtag, scripts[s].subtag, regions[r].subtag, variants[v].subtag].filter(notEmpty).join('-');
        var languageDescription = [languages[l].description, scripts[s].description, regions[r].description, variants[v].description].filter(notEmpty).join(', '); 

        assert.sameValue(typeof displayNames.of(languageTag), 'string', languageDescription + ": " + languageTag);
        if (variants[v].subtag !== ''){
          for (var vAdditional in variants){
            if (variants[vAdditional].subtag !== '' && vAdditional !== v){
              languageTag += '-' + variants[vAdditional].subtag;
              languageDescription += ", " + variants[vAdditional].description;
              assert.sameValue(typeof displayNames.of(languageTag), 'string', languageDescription + ": " + languageTag);
            }
          }
        }
      }
    }
  }
}
