## OpenHarmony fontconfig

Current description of the module is written for API level 12, however the structure of json file itself shouldn't change much.
I am leaving the link to newer API version bellow.

Standard location of fontconfig.json in OpenHarmony is:
**/etc/fontconfig.json**

Standard location of fonts in OpenHarmony is:
**/system/fonts/**

Locations above could easily change in future releases. If you will ever encounter any problems with loading this files read about AppSpawn mechanism in OpenHarmony.
It provides sandboxed environments for each application in OpenHarmony. Essentially any folder that you are able to acsess as unpriviledged app is governed by AppSpawn settings that is applied to your app.
It will essentially create mounts between **Host OS folder** and your App's **Sandboxed environment**

### Explanation of information contained in OpenHarmony fontconfig

#### fontdir
Represents array of folder where we will search files;

#### font_file_map
Represents a map that stores associations between **Font Full Name** and **Filename**;
Filenames - should be searched across all folders in fontdir array;
Font Full Names - **{Family name}** + [**{Supported language}**] + **{Font Style}** Is not really usefull for matching and CSS purposes. Ideally you should mmap font file and search for name table in TTF/TTC, then extract PostScript name from there and use it for font matching

#### generic
Describes what font families should be used as generic font families on OS level ([CSS-FONTS-4](https://www.w3.org/TR/css-fonts-4/#generic-family-name-syntax));
Most important information for web engine is **family name** and set of **aliases** (ui-serif, ui-sans-serif, ui-monospace, ui-rounded).
Also it contains set of hints and adjustments for font rendering systems in case they can not natively support **dynamic fonts** and parse information from font tables

#### fallback
Describes what font families specifies usage of segmented font families that OS provides (what font face should be used for specific language and script of the content) ([CSS-FONTS-4](https://www.w3.org/TR/css-fonts-4/#generic-family-name-syntax));
Most important information for web engine is association of **language** and **script** to  **script-specific family name**.
It could provide some unconditional fallbacks like:
***"": "Noto Sans"***.
Such fallbacks should be treated as **GenericFamily::None** by servo engine
Also it contains set of hints and adjustments for font rendering systems in case they can not natively support **dynamic fonts** and parse information from font tables

### Links to language standards:
 - https://standards.iso.org/iso/639/ed-2/en/
 - https://iso639-3.sil.org/sites/iso639-3/files/downloads/iso-639-3_Code_Tables_20250115.zip
 - https://iso639-3.sil.org/sites/iso639-3/files/downloads/iso-639-3.tab
 - https://www.unicode.org/iso15924/index.html
 - https://www.unicode.org/iso15924/codelists.html
 - https://unicode.org/iso15924/iso15924-codes.html

### Explanation on standards handling:
Currently 3 crates is used for standards handling:
 - https://docs.rs/codes-iso-639/latest/codes_iso_639/ /<- verify maintanence | fork candidate
 - https://docs.rs/codes-iso-15924/latest/codes_iso_15924/ /<- verify maintanence | fork candidate
 - https://crates.io/crates/icu_locid



### Small json format glossary:

Link for the full standard:
[The JavaScript Object Notation (JSON) Data Interchange Format. RFC 8259](https://datatracker.ietf.org/doc/html/rfc8259)

JSON_TEXT - serialized JSON_VALUE

JSON_VALUE - deserialized JSON_TEXT;
Represents entity which **MUST** be one of the following:
 - object:
 - array
 - number
 - string
 - literal_names: (**MUST** be lowercase - true, false, null)

I will refer objects above as JSON_OBJECT, JSON_ARRAY, JSON_NUMBER, JSON_STRING
JSON_OBJECT: set of pairs of JSON_STRING and corresponding JSON_VALUE (any of JSON_OBJECT, JSON_ARRAY, JSON_NUMBER, JSON_STRING)

### Structure of JSON_TEXT inside fontconfig.json

Currently Openharmony fontconfig.json contains JSON_TEXT with following sections:
 - fontdir : JSON_ARRAY[JSON_STRING]
 - generic : JSON_ARRAY[JSON_OBJECT]
 - fallback: JSON_ARRAY[JSON_OBJECT]
 - font_file_map: JSON_ARRAY[JSON_OBJECT]

#### single entry of **fontdir** JSON_ARRAY
- JSON_STRING("\<Path to folder where we will search font files\>")

#### single entry of **font_file_map** JSON_ARRAY
- JSON_STRING("\<Font full name\>") : JSON_STRING("\<Font file name\>")

#### **generic**
Represents array of JSON_OBJECTS with following structure:
JSON_OBJECT that will contain:
 - JSON_STRING("family") : JSON_STRING("\<Font family name\>")
 - JSON_STRING("alias") : JSON_ARRAY[JSON_OBJECT]
 - JSON_STRING("adjust") : JSON_ARRAY[JSON_OBJECT]
 - JSON_STRING("font-variations") : JSON_ARRAY[JSON_OBJECT]

##### single entry of **alias** JSON_ARRAY
JSON_OBJECT that will contain:
 - JSON_STRING("\<Some string that represents font\>") : JSON_NUMBER(\<int number that represent font-weight\>)
[CSS-fonts-4, "Syntax of <generic-family>"](https://www.w3.org/TR/css-fonts-4/#generic-family-name-syntax)
two important examples:
 - JSON_STRING("\<Font PostScript name\>") : JSON_NUMBER(\<int number\>)  -  "HarmonyOS-Sans-Light": 100
 - JSON_STRING("\<universal generic font\>") : JSON_NUMBER(\<int number that represent weight\>)  -  "serif": 0

##### single entry of **adjust** JSON_ARRAY
JSON_OBJECT that will contain:
 - JSON_STRING("weight") : JSON_NUMBER(\<Some int number\>)
 - JSON_STRING("to") : JSON_NUMBER(\<Some int number\>)

##### single entry of font-variations JSON_ARRAY
JSON_OBJECT that will contain:
 - JSON_STRING("weight") : JSON_NUMBER(\<Some int number\>)
 - JSON_STRING("wght") : JSON_NUMBER(\<Some int number\>)
 [CSS-fonts-4, "font-variations-settings"](https://www.w3.org/TR/css-fonts-4/#descdef-font-face-font-variation-settings)


#### **fallback**
Represents array of JSON_OBJECTS with following structure:
JSON_OBJECT that will contain:
 - JSON_STRING("\<Some string that names fallback strategy\>") : JSON_ARRAY[JSON_OBJECT]

##### single entry of JSON_ARRAY that represents named fallback strategy
JSON_OBJECT that will contain:
 - JSON_STRING("\<Some string with lang-script pair\>") : JSON_STRING("\<Some string with script-specific generic font family name\>")
 - OPTIONAL_ENTRY(JSON_STRING("font-variations") : JSON_ARRAY[JSON_OBJECT])
[CSS-fonts-4, "Syntax of <generic-family>: script-specific generic font"](https://www.w3.org/TR/css-fonts-4/#generic-family-name-syntax)
[CSS-fonts-4, "font-variations-settings"](https://www.w3.org/TR/css-fonts-4/#descdef-font-face-font-variation-settings)
font-variations entry is essentially the same as in **generic**;


### OpenHarmony SDK releases:
 - OpenHarmony-v5.0.0 - SDK API version 12 [en](https://gitee.com/openharmony/docs/blob/OpenHarmony-5.0.3-Release/en/release-notes/OpenHarmony-v5.0.0-release.md) [zh-cn](https://gitee.com/openharmony/docs/blob/OpenHarmony-5.0.3-Release/zh-cn/release-notes/OpenHarmony-v5.0.0-release.md)
 - OpenHarmony-v5.0.1 - SDK API version 13 [en](https://gitee.com/openharmony/docs/blob/OpenHarmony-5.0.3-Release/en/release-notes/OpenHarmony-v5.0.1-release.md) [zh-cn](https://gitee.com/openharmony/docs/blob/OpenHarmony-5.0.3-Release/zh-cn/release-notes/OpenHarmony-v5.0.1-release.md)
 - OpenHarmony-v5.0.2 - SDK API version 14 [zh-cn](https://gitee.com/openharmony/docs/blob/OpenHarmony-5.0.3-Release/zh-cn/release-notes/OpenHarmony-v5.0.2-release.md)