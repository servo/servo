use gecko_bindings::structs::nsIAtom;

use Atom;

pub fn unsafe_atom_from_static(ptr: *mut nsIAtom) -> Atom { unsafe { Atom::from_static(ptr) } }

extern { pub static _ZN9nsGkAtoms6_emptyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3mozE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12mozframetypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11_moz_absposE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14_moz_activatedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13_moz_resizingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18mozallowfullscreenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7moztypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8mozdirtyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms25mozdisallowselectionprintE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12mozdonotsendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18mozeditorbogusnodeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms25mozgeneratedcontentbeforeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24mozgeneratedcontentafterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24mozgeneratedcontentimageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8mozquoteE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12mozsignatureE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13_moz_is_glyphE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18_moz_original_sizeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11_moz_targetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10menuactiveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13_poundDefaultE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9_asteriskE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1aE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4abbrE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5abortE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5aboveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9acceltextE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6acceptE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13acceptcharsetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9accesskeyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7acronymE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6actionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6activeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19activetitlebarcolorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13activateontabE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7actuateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7addressE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5afterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9after_endE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11after_startE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5alignE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5alinkE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3allE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11alloweventsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23allownegativeassertionsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10allowformsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15allowfullscreenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20alloworientationlockE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16allowpointerlockE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11allowpopupsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15allowsameoriginE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12allowscriptsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18allowtopnavigationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14allowuntrustedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3altE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9alternateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6alwaysE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8ancestorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14ancestorOrSelfE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6anchorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4_andE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10animationsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6anonidE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12anonlocationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3anyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6mozappE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9mozwidgetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6appletE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12applyImportsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14applyTemplatesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10mozapptypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7archiveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4areaE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5arrowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7articleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9ascendingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5asideE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11aspectRatioE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6assignE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5asyncE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9attributeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10attributesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12attributeSetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5auralE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5_autoE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9autocheckE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12autocompleteE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9autofocusE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8autoplayE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16autorepeatbuttonE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4axisE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1bE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13backdropFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10backgroundE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4baseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8basefontE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8baselineE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3bdiE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3bdoE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6beforeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10before_endE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12before_startE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5belowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7bgcolorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7bgsoundE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3bigE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7bindingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8bindingsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22bindToUntrustedContentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8blankrowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5blockE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10blockquoteE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4blurE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4bodyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7booleanE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6borderE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11bordercolorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4bothE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6bottomE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9bottomendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11bottomstartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10bottomleftE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12bottommarginE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13bottompaddingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11bottomrightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3boxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2brE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7brailleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9broadcastE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11broadcasterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14broadcastersetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7browserE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10mozbrowserE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13bulletinboardE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6buttonE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24brighttitlebarforegroundE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12callTemplateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6cancelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6canvasE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7captionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7captureE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9caseOrderE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20cdataSectionElementsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7ceilingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4cellE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11cellpaddingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11cellspacingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6centerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2chE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6changeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5_charE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13characterDataE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8charcodeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7charoffE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7charsetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8checkboxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7checkedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5childE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8childrenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9childListE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6chooseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12chromemarginE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17chromeOnlyContentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24exposeToUntrustedContentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4circE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6circleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4citeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6_classE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7classidE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5clearE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5clickE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10clickcountE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12clickthroughE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11movetoclickE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4clipE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5closeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6closedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9closemenuE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21coalesceduplicatearcsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4codeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8codebaseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8codetypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3colE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8colgroupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8collapseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9collapsedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5colorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10colorIndexE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4colsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7colspanE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6columnE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7columnsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8comboboxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7commandE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8commandsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10commandsetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13commandupdateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14commandupdaterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7commentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7compactE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6concatE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10conditionsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11constructorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20consumeoutsideclicksE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9containerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11containmentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8containsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7contentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15contenteditableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24headerContentDispositionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21headerContentLanguageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15contentLocationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23headerContentScriptTypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22headerContentStyleTypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17headerContentTypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13consumeanchorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7contextE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11contextmenuE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7controlE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8controlsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6coordsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4copyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6copyOfE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5countE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4cropE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11crossoriginE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6curposE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7currentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6cyclerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4dataE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8datalistE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8dataTypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8dateTimeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11datasourcesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8datetimeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8dblclickE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2ddE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5debugE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13decimalFormatE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16decimalSeparatorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4deckE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7declareE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13decoderDoctorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9decrementE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8_defaultE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18headerDefaultStyleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13defaultActionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14defaultcheckedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12defaultLabelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15defaultselectedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12defaultvalueE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19defaultplaybackrateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5deferE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3delE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10descendantE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16descendantOrSelfE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10descendingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11descriptionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10destructorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7detailsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17deviceAspectRatioE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12deviceHeightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16devicePixelRatioE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11deviceWidthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3dfnE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6dialogE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10differenceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5digitE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3dirE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12dirAutoSetByE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14directionalityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9directoryE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21disableOutputEscapingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8disabledE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20disableglobalhistoryE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14disablehistoryE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7displayE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11displayModeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8distinctE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3divE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2dlE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13doctypePublicE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13doctypeSystemE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8documentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8downloadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15DOMAttrModifiedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24DOMCharacterDataModifiedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15DOMNodeInsertedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms27DOMNodeInsertedIntoDocumentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14DOMNodeRemovedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms26DOMNodeRemovedFromDocumentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18DOMSubtreeModifiedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7double_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4dragE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8dragdropE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7dragendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9dragenterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9drageventE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8dragexitE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9draggableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11draggestureE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8draggingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9dragleaveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8dragoverE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11dragSessionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9dragstartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14drawintitlebarE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9drawtitleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4dropE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9dropAfterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10dropBeforeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6dropOnE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10dropMarkerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2dtE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8editableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7editingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6editorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17editorDisplayListE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7elementE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16elementAvailableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8elementsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2emE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5embedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8embossedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5emptyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8encodingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7enctypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3endE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8endEventE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9end_afterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10end_beforeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9equalsizeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5errorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4evenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5eventE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6eventsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21excludeResultPrefixesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8excludesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4exprE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22expectingSystemMessageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7extendsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24extensionElementPrefixesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4faceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8fallbackE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6_falseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8farthestE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5fieldE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8fieldsetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10figcaptionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6figureE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5fixedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5flagsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4flexE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9flexgroupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4flipE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8floatingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5floorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10flowlengthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5focusE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7focusedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9followingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16followingSiblingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4fontE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10fontWeightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10fontpickerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6footerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4_forE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7forEachE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21forceOwnRefreshDriverE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4formE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10formactionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6formatE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12formatNumberE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11formenctypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10formmethodE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14formnovalidateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10formtargetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5frameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11frameborderE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8framesetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4fromE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16fullscreenchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15fullscreenerrorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17functionAvailableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10generateIdE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6getterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9glyphcharE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7glyphidE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4gridE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6grippyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5groupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17groupingSeparatorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12groupingSizeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4growE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6gutterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2h1E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2h2E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2h3E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2h4E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2h5E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2h6E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8handheldE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16handheldFriendlyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7handlerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8handlersE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4HARDE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11hasSameNodeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4hboxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4headE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6headerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7headersE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6heightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6hgroupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6hiddenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10hidechromeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16hidecolumnpickerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4highE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7highestE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10horizontalE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5hoverE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2hrE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4hrefE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8hreflangE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6hspaceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4htmlE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9httpEquivE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1iE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4iconE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2idE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3_ifE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6iframeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10ignorecaseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10ignorekeysE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15ignoreuserfocusE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6ilayerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5imageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17imageClickedPointE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3imgE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14implementationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10implementsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6importE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21inactivetitlebarcolorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7includeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8includesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9incrementE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6indentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13indeterminateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5indexE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5inferE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8infinityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7inheritE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8inheritsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12inheritstyleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13initial_scaleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5inputE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9inputmodeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3insE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11insertafterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12insertbeforeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10instanceOfE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5int32E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5int64E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7integerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9integrityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12intersectionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2isE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11iscontainerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7isemptyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5ismapE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6itemidE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8itempropE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7itemrefE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9itemscopeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8itemtypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3kbdE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17keepcurrentinviewE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16keepobjectsaliveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3keyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7keycodeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17keystatuseschangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7keydownE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6keygenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8keypressE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6keysetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9keysystemE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7keytextE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5keyupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4kindE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5labelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4langE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8languageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4lastE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5layerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13LayerActivityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6layoutE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7leadingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4leafE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4leftE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10leftmarginE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11leftpaddingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6legendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6lengthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11letterValueE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5levelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2liE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4lineE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4linkE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4listE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7listboxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11listboxbodyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8listcellE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7listcolE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8listcolsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8listenerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8listheadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10listheaderE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7listingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8listitemE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8listrowsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4loadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9localedirE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9localNameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8longdescE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4loopE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3lowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10lowerFirstE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6lowestE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6lowsrcE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3ltrE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7lwthemeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16lwthemetextcolorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4mainE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3mapE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8manifestE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12marginBottomE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10marginLeftE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11marginRightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9marginTopE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12marginheightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11marginwidthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4markE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7marqueeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5matchE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3maxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9maxheightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13maximum_scaleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9maxlengthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6maxposE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8maxwidthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9mayscriptE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5mediaE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9mediaTypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6memberE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4menuE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7menubarE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10menubuttonE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10menuButtonE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9menugroupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8menuitemE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8menulistE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9menupopupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13menuseparatorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7messageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4metaE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8referrerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14referrerpolicyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5meterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6methodE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19microdataPropertiesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6middleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3minE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9minheightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13minimum_scaleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6minposE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9minusSignE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8minwidthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6_mixedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19messagemanagergroupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3modE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4modeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9modifiersE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10monochromeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9mousedownE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9mousemoveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8mouseoutE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9mouseoverE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12mousethroughE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7mouseupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15mozaudiochannelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19mozfullscreenchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18mozfullscreenerrorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20mozpasspointereventsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20mozpointerlockchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19mozpointerlockerrorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18mozprivatebrowsingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10moz_opaqueE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15moz_action_hintE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18x_moz_errormessageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17msthemecompatibleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8multicolE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8multipleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5mutedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4nameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10_namespaceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14namespaceAliasE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12namespaceUriE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3NaNE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24nativeAnonymousChildListE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3navE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6negateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5neverE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4_newE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7newlineE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8nextBidiE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2noE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11noautofocusE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10noautohideE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16norolluponanchorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4nobrE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4nodeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12nodefaultsrcE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7nodeSetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7noembedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8noframesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6nohrefE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11noisolationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5nonceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4noneE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8noresizeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6normalE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14normalizeSpaceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8noscriptE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7noshadeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10novalidateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4_notE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6nowrapE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6numberE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4nullE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6objectE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10objectTypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8observerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8observesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3oddE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3OFFE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2olE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18omitXmlDeclarationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19ona2dpstatuschangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onabortE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onactivateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14onadapteraddedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16onadapterremovedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onafterprintE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onafterscriptexecuteE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onalertingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14onanimationendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onanimationiterationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16onanimationstartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24onantennaavailablechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onAppCommandE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onattributechangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onattributereadreqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19onattributewritereqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14onaudioprocessE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onbeforecopyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onbeforecutE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onbeforepasteE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15onbeforeevictedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onbeforeprintE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21onbeforescriptexecuteE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14onbeforeunloadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onblockedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6onblurE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onbroadcastE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6onbusyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19onbufferedamountlowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8oncachedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14oncallschangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8oncancelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17oncardstatechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15oncfstatechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8onchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23oncharacteristicchangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16onchargingchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onchargingtimechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10oncheckingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onclickE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16onclirmodechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7oncloseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9oncommandE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15oncommandupdateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10oncompleteE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16oncompositionendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18oncompositionstartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19oncompositionupdateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21onconfigurationchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onconnectE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onconnectedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onconnectingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21onconnectionavailableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24onconnectionstatechangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13oncontextmenuE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6oncopyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23oncurrentchannelchangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22oncurrentsourcechangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5oncutE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12ondatachangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11ondataerrorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10ondblclickE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9ondeletedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17ondeliverysuccessE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15ondeliveryerrorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13ondevicefoundE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14ondevicepairedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16ondeviceunpairedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9ondialingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10ondisabledE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23ondischargingtimechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12ondisconnectE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14ondisconnectedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15ondisconnectingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19ondisplaypasskeyreqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13ondownloadingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onDOMActivateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17onDOMAttrModifiedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms26onDOMCharacterDataModifiedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onDOMFocusInE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onDOMFocusOutE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16onDOMMouseScrollE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17onDOMNodeInsertedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms29onDOMNodeInsertedIntoDocumentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16onDOMNodeRemovedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms28onDOMNodeRemovedFromDocumentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onDOMSubtreeModifiedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6ondataE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6ondragE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10ondragdropE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9ondragendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11ondragenterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10ondragexitE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13ondraggestureE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11ondragleaveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10ondragoverE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11ondragstartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7ondrainE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6ondropE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16oneitbroadcastedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onenabledE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17onenterpincodereqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23onemergencycbmodechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onerrorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onevictedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15onfacesdetectedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8onfailedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onfetchE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8onfinishE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onfocusE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17onfrequencychangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onfullscreenchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17onfullscreenerrorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21onspeakerforcedchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5ongetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13ongroupchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onhashchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onheadphoneschangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6onheldE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onhfpstatuschangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onhidstatuschangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onholdingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11oniccchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13oniccdetectedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15oniccinfochangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15oniccundetectedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onincomingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7oninputE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9oninstallE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9oninvalidE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onkeydownE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onkeypressE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onkeyupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16onlanguagechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onlevelchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6onLoadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6onloadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onloadingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onloadingdoneE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14onloadingerrorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onpopstateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4onlyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onmessageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onmousedownE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onmouseenterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onmouseleaveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onmousemoveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onmouseoutE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onmouseoverE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17onMozMouseHittestE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onmouseupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15onMozAfterPaintE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24onmozbrowserafterkeydownE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22onmozbrowserafterkeyupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms25onmozbrowserbeforekeydownE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23onmozbrowserbeforekeyupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21onmozfullscreenchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onmozfullscreenerrorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onmozkeydownonpluginE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onmozkeyuponpluginE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22onmozpointerlockchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21onmozpointerlockerrorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15onmoztimechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21onMozMousePixelScrollE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24onMozScrolledAreaChangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onmoznetworkuploadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onmoznetworkdownloadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21onmapfolderlistingreqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23onmapmessageslistingreqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onmapgetmessagereqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24onmapsetmessagestatusreqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19onmapsendmessagereqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21onmapmessageupdatereqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onnewrdsgroupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19onnotificationclickE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onnoupdateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17onobexpasswordreqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onobsoleteE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8ononlineE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onofflineE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6onopenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19onorientationchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17onotastatuschangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onoverflowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17onoverflowchangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onpagehideE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onpageshowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onpaintE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16onpairingabortedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24onpairingconfirmationreqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19onpairingconsentreqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onpasteE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15onpendingchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onpichangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onpictureE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onpopuphiddenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onpopuphidingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14onpopupshowingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onpopupshownE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8onposterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onpreviewstatechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onpullphonebookreqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19onpullvcardentryreqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21onpullvcardlistingreqE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6onpushE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24onpushsubscriptionchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onpschangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onptychangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onradiostatechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onrdsdisabledE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onrdsenabledE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onreaderrorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onreadsuccessE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onreadyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onreadystatechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onreceivedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21onrecorderstatechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onremoteheldE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15onremoteresumedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms26onresourcetimingbufferfullE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onretrievingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onRequestE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24onrequestmediaplaystatusE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onresetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onresumingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8onresizeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onrtchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22onscanningstatechangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onscostatuschangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8onscrollE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8onselectE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17onselectionchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onselectstartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onsendingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6onsentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5onsetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6onshowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onshutterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onstatechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15onstatuschangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onstkcommandE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15onstksessionendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onstorageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onstorageareachangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8onsubmitE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onsuccessE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12ontypechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6ontextE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8ontoggleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12ontouchstartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10ontouchendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11ontouchmoveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13ontouchcancelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15ontransitionendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onunderflowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8onunloadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onupdatefoundE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onupdatereadyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15onupgradeneededE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14onussdreceivedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15onversionchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onvoicechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15onvoiceschangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onwebkitAnimationEndE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms26onwebkitAnimationIterationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22onwebkitAnimationStartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21onwebkitTransitionEndE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onwheelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4openE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8optgroupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7optimumE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6optionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3_orE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5orderE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7ordinalE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6orientE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11orientationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9otherwiseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6outputE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8overflowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15overflowchangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7overlayE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7overlapE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1pE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4packE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4pageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13pageincrementE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5pagexE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5pageyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11paint_orderE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11palettenameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5panelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5paramE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9parameterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6parentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9parentappE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13parentfocusedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9parsetypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8passwordE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7patternE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16patternSeparatorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8perMilleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7percentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7persistE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5phaseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7pictureE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4pingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6pinnedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11placeholderE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9plaintextE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12playbackrateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9pointSizeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4polyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7polygonE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5popupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10popupalignE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11popupanchorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10popupgroupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11popuphiddenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11popuphidingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8popupsetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12popupshowingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10popupshownE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20popupsinherittooltipE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8positionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6posterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3preE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9precedingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16precedingSiblingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9predicateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6prefixE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7preloadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11prerenderedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8preserveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13preserveSpaceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14preventdefaultE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7primaryE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5printE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8priorityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21processingInstructionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7profileE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8progressE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13progressmeterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14progressNormalE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20progressUndeterminedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10projectionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6promptE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9propagateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10propertiesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8propertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7pubdateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1qE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5queryE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8querysetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9querytypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5radioE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10radiogroupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5rangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8readonlyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4rectE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9rectangleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3refE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7refreshE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3relE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onreloadpageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3remE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13removeelementE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21renderingobserverlistE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6repeatE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7replaceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8requiredE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8reservedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5resetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11resizeafterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12resizebeforeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7resizerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10resolutionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8resourceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9resourcesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6resultE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12resultPrefixE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21retargetdocumentfocusE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3revE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7reverseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8reversedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11richlistboxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12richlistitemE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5rightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11rightmarginE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12rightpaddingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4roleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18rolluponmousewheelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5roundE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3rowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4rowsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7rowspanE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2rbE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2rpE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2rtE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3rtcE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3rtlE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4rubyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8rubyBaseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17rubyBaseContainerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8rubyTextE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17rubyTextContainerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4ruleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5rulesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1sE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4sampE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7sandboxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6sbattrE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5scaleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4scanE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6schemeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5scopeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6scopedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6screenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7screenXE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7screenYE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6scriptE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms33scriptEnabledBeforePrintOrPreviewE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9scrollbarE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15scrollbarbuttonE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19scrollbarDownBottomE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16scrollbarDownTopE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17scrollbarUpBottomE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14scrollbarUpTopE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9scrollboxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12scrollcornerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9scrollingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7sectionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6selectE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10selectableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8selectedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13selectedIndexE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13selectedindexE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4selfE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7seltypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9setcookieE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6setterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5shapeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4showE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9showcaretE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11showresizerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6simpleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6singleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4sizeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5sizesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8sizemodeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11sizetopopupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6sliderE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5smallE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6smoothE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4snapE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4sortE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10sortActiveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13sortDirectionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6sortedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9sorthintsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10sortLockedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12sortResourceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13sortResource2E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14sortSeparatorsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15sortStaticsLastE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6sourceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5spaceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6spacerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4spanE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10spellcheckE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7spinnerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5splitE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9splitmenuE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8splitterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6springE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3srcE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6srcdocE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7srclangE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6srcsetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5stackE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10standaloneE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7standbyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5startE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11start_afterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12start_beforeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10startsWithE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5stateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15statedatasourceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10staticHintE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9statusbarE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10statustextE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4stepE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4stopE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7stretchE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6strikeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6stringE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12stringLengthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10stripSpaceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6strongE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5styleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10stylesheetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16stylesheetPrefixE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7subjectE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6submitE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8substateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9substringE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14substringAfterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15substringBeforeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3subE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3sumE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3supE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7summaryE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14systemPropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3tabE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6tabboxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8tabindexE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5tableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8tabpanelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9tabpanelsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3tagE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6targetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7targetsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5tbodyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2tdE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9_templateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15text_decorationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9terminateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4testE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4textE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9textAlignE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8textareaE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7textboxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8textnodeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms25textNodeDirectionalityMapE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5tfootE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2thE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5theadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5thumbE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4timeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5titleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8titlebarE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8titletipE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7toggledE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5tokenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8tokenizeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7toolbarE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13toolbarbuttonE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11toolbaritemE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7toolboxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7tooltipE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11tooltiptextE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3topE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7topleftE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9topmarginE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10toppaddingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8toprightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2trE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5trackE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8trailingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9transformE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12transform_3dE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12transformiixE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9translateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11transparentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4treeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8treecellE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12treechildrenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7treecolE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13treecolpickerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8treecolsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8treeitemE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7treerowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13treeseparatorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6tripleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5_trueE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2ttE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3ttyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2tvE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4typeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13typemustmatchE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1uE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2ulE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9underflowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12undeterminedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9undoscopeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6unloadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17unparsedEntityUriE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10upperFirstE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3uriE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3useE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16useAttributeSetsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6usemapE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13user_scalableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9userInputE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8validateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6valignE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5valueE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6valuesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7valueOfE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9valuetypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3varE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8variableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4vboxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10vcard_nameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6vendorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9vendorUrlE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7versionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4vertE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8verticalE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5audioE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5videoE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13videocontrolsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8viewportE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15viewport_heightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22viewport_initial_scaleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22viewport_maximum_scaleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22viewport_minimum_scaleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22viewport_user_scalableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14viewport_widthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10visibilityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16visuallyselectedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5vlinkE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6vspaceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3wbrE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4whenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5whereE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6widgetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5widthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6windowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18headerWindowTargetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10windowtypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9withParamE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6wizardE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4wrapE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24headerDNSPrefetchControlE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9headerCSPE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19headerCSPReportOnlyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9headerXFOE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9x_westernE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3xmlE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14xml_stylesheetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5xmlnsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3xmpE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20xulcontentsgeneratedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3yesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7z_indexE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9zeroDigitE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10percentageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1AE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18alignment_baselineE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12allowReorderE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8altGlyphE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11altGlyphDefE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12altGlyphItemE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9amplitudeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7animateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12animateColorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13animateMotionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16animateTransformE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10arithmeticE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4atopE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7azimuthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1BE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15backgroundColorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16background_imageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13baseFrequencyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14baseline_shiftE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4biasE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12caption_sideE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9clip_pathE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9clip_ruleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8clipPathE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13clipPathUnitsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2cmE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9colorBurnE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10colorDodgeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18colorInterpolationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms25colorInterpolationFiltersE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12colorProfileE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6cursorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2cxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2cyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1dE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6darkenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4defsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3degE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4descE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15diffuseConstantE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6dilateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9directionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7disableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8discreteE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7divisorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17dominant_baselineE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9duplicateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2dxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2dyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8edgeModeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7ellipseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9elevationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5erodeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2exE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5exactE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9exclusionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8exponentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7feBlendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13feColorMatrixE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19feComponentTransferE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11feCompositeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16feConvolveMatrixE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17feDiffuseLightingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17feDisplacementMapE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14feDistantLightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12feDropShadowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7feFloodE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7feFuncAE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7feFuncBE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7feFuncGE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7feFuncRE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14feGaussianBlurE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7feImageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7feMergeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11feMergeNodeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12feMorphologyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8feOffsetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12fePointLightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18feSpecularLightingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11feSpotLightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6feTileE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12feTurbulenceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4fillE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12fill_opacityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9fill_ruleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6filterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11filterUnitsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6_floatE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11flood_colorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13flood_opacityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9font_faceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16font_face_formatE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14font_face_nameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13font_face_srcE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13font_face_uriE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11font_familyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9font_sizeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16font_size_adjustE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12font_stretchE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10font_styleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12font_variantE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13foreignObjectE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12fractalNoiseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2fxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2fyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1GE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1gE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5gammaE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8generic_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8glyphRefE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4gradE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17gradientTransformE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13gradientUnitsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9hardLightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3hueE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9hueRotateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8identityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15image_renderingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2inE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3in2E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9interceptE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2k1E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2k2E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2k3E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2k4E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12kernelMatrixE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16kernelUnitLengthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12lengthAdjustE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14letter_spacingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7lightenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14lighting_colorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17limitingConeAngleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6linearE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14linearGradientE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9linearRGBE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15list_style_typeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16luminanceToAlphaE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10luminosityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7magnifyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6markerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10marker_endE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10marker_midE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12marker_startE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12markerHeightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11markerUnitsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11markerWidthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4maskE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16maskContentUnitsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9mask_typeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9maskUnitsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6matrixE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8metadataE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12missingGlyphE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2mmE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5mpathE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8noStitchE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10numOctavesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8multiplyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17objectBoundingBoxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6offsetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onSVGLoadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onSVGResizeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onSVGScrollE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onSVGUnloadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onSVGZoomE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6onzoomE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7opacityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9_operatorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3outE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4overE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms27overridePreserveAspectRatioE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3padE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4pathE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10pathLengthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19patternContentUnitsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16patternTransformE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12patternUnitsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2pcE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14pointer_eventsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6pointsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9pointsAtXE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9pointsAtYE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9pointsAtZE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8polylineE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13preserveAlphaE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19preserveAspectRatioE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14primitiveUnitsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2ptE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2pxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1RE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1rE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3radE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14radialGradientE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6radiusE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7reflectE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4refXE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4refYE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18requiredExtensionsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16requiredFeaturesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6rotateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2rxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2ryE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8saturateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10saturationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3setE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4seedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6shadowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15shape_renderingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5skewXE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5skewYE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5slopeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9softLightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7spacingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16spacingAndGlyphsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16specularConstantE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16specularExponentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12spreadMethodE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4sRGBE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11startOffsetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12stdDeviationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6stitchE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11stitchTilesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10stop_colorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12stop_opacityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6strokeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16stroke_dasharrayE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17stroke_dashoffsetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14stroke_linecapE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15stroke_linejoinE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17stroke_miterlimitE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14stroke_opacityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12stroke_widthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11strokeWidthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12surfaceScaleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3svgE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9svgSwitchE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6symbolE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14systemLanguageE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11tableValuesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7targetXE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7targetYE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11text_anchorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14text_renderingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10textLengthE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8textPathE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4trefE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5tspanE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10turbulenceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12unicode_bidiE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14userSpaceOnUseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4viewE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7viewBoxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10viewTargetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11white_spaceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12word_spacingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12writing_modeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1xE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2x1E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2x2E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16xChannelSelectorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4xor_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1yE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2y1E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2y2E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16yChannelSelectorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms1zE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10zoomAndPanE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13vector_effectE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14vertical_alignE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10accumulateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8additiveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13attributeNameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13attributeTypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12auto_reverseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5beginE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10beginEventE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2byE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8calcModeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3cssE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3durE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9keyPointsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10keySplinesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8keyTimesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms25mozAnimateMotionDummyAttrE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onbeginE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onbeginEventE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5onendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onendEventE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8onrepeatE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onrepeatEventE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11repeatCountE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9repeatDurE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11repeatEventE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7restartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2toE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3XMLE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4abs_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7accent_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12accentunder_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11actiontype_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15alignmentscope_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7altimg_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14altimg_height_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14altimg_valign_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13altimg_width_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11annotation_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15annotation_xml_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6apply_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7approx_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7arccos_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8arccosh_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7arccot_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8arccoth_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7arccsc_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8arccsch_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7arcsec_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8arcsech_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7arcsin_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8arcsinh_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7arctan_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8arctanh_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4arg_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9bevelled_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5bind_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5bvar_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5card_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17cartesianproduct_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7cbytes_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3cd_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8cdgroup_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7cerror_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10charalign_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3ci_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8closure_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3cn_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9codomain_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12columnalign_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16columnalignment_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12columnlines_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14columnspacing_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11columnspan_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12columnwidth_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10complexes_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8compose_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10condition_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10conjugate_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4cos_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5cosh_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4cot_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5coth_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9crossout_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4csc_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5csch_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3cs_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8csymbol_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5curl_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13decimalpoint_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14definitionURL_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7degree_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11denomalign_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6depth_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12determinant_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5diff_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13displaystyle_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11divergence_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7divide_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7domain_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20domainofapplication_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5edge_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3el_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9emptyset_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3eq_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13equalcolumns_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10equalrows_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11equivalent_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11eulergamma_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7exists_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4exp_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13exponentiale_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10factorial_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9factorof_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6fence_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3fn_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11fontfamily_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9fontsize_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10fontstyle_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11fontweight_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7forall_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13framespacing_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4gcd_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4geq_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11groupalign_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3gt_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6ident_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11imaginaryi_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10imaginary_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8implies_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17indentalignfirst_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12indentalign_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16indentalignlast_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17indentshiftfirst_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12indentshift_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13indenttarget_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9integers_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10intersect_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9interval_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4int_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8inverse_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7lambda_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10laplacian_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8largeop_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4lcm_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4leq_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6limit_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10linebreak_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18linebreakmultchar_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15linebreakstyle_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14linethickness_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5list_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3ln_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9location_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8logbase_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4log_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13longdivstyle_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9lowlimit_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7lquote_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7lspace_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3lt_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8maction_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12maligngroup_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11malignmark_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15mathbackground_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10mathcolor_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9mathsize_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12mathvariant_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10matrixrow_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8maxsize_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5mean_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7median_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9menclose_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7merror_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8mfenced_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6mfrac_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7mglyph_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3mi_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16minlabelspacing_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8minsize_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6minus_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11mlabeledtr_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9mlongdiv_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14mmultiscripts_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3mn_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12momentabout_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7moment_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3mo_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14movablelimits_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6mover_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8mpadded_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9mphantom_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12mprescripts_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6mroot_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5mrow_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10mscarries_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8mscarry_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8msgroup_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7msline_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3ms_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7mspace_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6msqrt_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6msrow_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7mstack_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7mstyle_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5msub_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8msubsup_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5msup_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7mtable_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4mtd_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6mtext_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4mtr_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7munder_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11munderover_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15naturalnumbers_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4neq_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11notanumber_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9notation_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5note_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6notin_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12notprsubset_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10notsubset_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9numalign_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6other_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13outerproduct_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12partialdiff_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6piece_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10piecewise_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3pi_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5plus_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6power_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7primes_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8product_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9prsubset_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9quotient_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10rationals_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5real_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6reals_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5reln_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5root_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9rowalign_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9rowlines_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11rowspacing_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7rquote_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7rspace_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14scalarproduct_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15schemaLocation_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12scriptlevel_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14scriptminsize_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21scriptsizemultiplier_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11scriptsize_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5sdev_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5sech_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4sec_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10selection_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9selector_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10semantics_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10separator_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11separators_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4sep_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8setdiff_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4set_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6share_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6shift_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5side_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5sinh_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4sin_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11stackalign_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9stretchy_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15subscriptshift_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7subset_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17superscriptshift_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10symmetric_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5tanh_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4tan_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8tendsto_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6times_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10transpose_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6union_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8uplimit_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9variance_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14vectorproduct_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7vector_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8voffset_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5xref_E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4mathE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3avgE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17booleanFromStringE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13countNonEmptyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12daysFromDateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4initE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8instanceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6monthsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3nowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7secondsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19secondsFromDateTimeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms25onMozSwipeGestureMayStartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22onMozSwipeGestureStartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23onMozSwipeGestureUpdateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onMozSwipeGestureEndE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17onMozSwipeGestureE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24onMozMagnifyGestureStartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms25onMozMagnifyGestureUpdateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19onMozMagnifyGestureE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23onMozRotateGestureStartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24onMozRotateGestureUpdateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onMozRotateGestureE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15onMozTapGestureE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onMozPressTapGestureE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18onMozEdgeUIStartedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19onMozEdgeUICanceledE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onMozEdgeUICompletedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onpointerdownE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onpointermoveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onpointerupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15onpointercancelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onpointeroverE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onpointeroutE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14onpointerenterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14onpointerleaveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19ongotpointercaptureE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onlostpointercaptureE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14ondevicemotionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19ondeviceorientationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms27onabsolutedeviceorientationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17ondeviceproximityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22onmozorientationchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15onuserproximityE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13ondevicelightE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19onmozinterruptbeginE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17onmozinterruptendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12cdataTagNameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14commentTagNameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16documentNodeNameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24documentFragmentNodeNameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20documentTypeNodeNameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms28processingInstructionTagNameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11textTagNameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16bcTableCellFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10blockFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8boxFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7brFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11bulletFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17colorControlFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14columnSetFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20comboboxControlFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20comboboxDisplayFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9deckFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12detailsFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13fieldSetFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18flexContainerFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16formControlFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13frameSetFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21gfxButtonControlFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18gridContainerFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22HTMLButtonControlFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15HTMLCanvasFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16subDocumentFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13imageBoxFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10imageFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17imageControlFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11inlineFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12leafBoxFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11legendFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11letterFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9lineFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16listControlFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9menuFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10meterFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14menuPopupFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18numberControlFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11objectFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9pageFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14pageBreakFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16pageContentFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16placeholderFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13popupSetFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13progressFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11canvasFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10rangeFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9rootFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22rubyBaseContainerFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13rubyBaseFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9rubyFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22rubyTextContainerFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13rubyTextFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11scrollFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14scrollbarFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13sequenceFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11sliderFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14tableCellFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13tableColFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18tableColGroupFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10tableFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15tableOuterFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18tableRowGroupFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13tableRowFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14textInputFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9textFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13viewportFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13XULLabelFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9svgAFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16svgClipPathFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12svgDefsFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19svgFEContainerFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15svgFEImageFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14svgFELeafFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22svgFEUnstyledLeafFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14svgFilterFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21svgForeignObjectFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24svgGenericContainerFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9svgGFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16svgGradientFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13svgImageFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16svgInnerSVGFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22svgLinearGradientFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14svgMarkerFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23svgMarkerAnonChildFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12svgMaskFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16svgOuterSVGFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms25svgOuterSVGAnonChildFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20svgPathGeometryFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15svgPatternFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22svgRadialGradientFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12svgStopFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14svgSwitchFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12svgTextFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11svgUseFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12svgViewFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14HTMLVideoFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onloadendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onloadstartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onprogressE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onsuspendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onemptiedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onstalledE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6onplayE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onpauseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16onloadedmetadataE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onloadeddataE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onwaitingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onplayingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9oncanplayE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16oncanplaythroughE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onseekingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8onseekedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9ontimeoutE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12ontimeupdateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onendedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onratechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16ondurationchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14onvolumechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onaddtrackE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18oncontrollerchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11oncuechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onenterE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6onexitE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onencryptedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9encryptedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onremovetrackE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9loadstartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7suspendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7emptiedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7stalledE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4playE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5pauseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14loadedmetadataE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10loadeddataE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7waitingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7playingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7seekingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6seekedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10timeupdateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5endedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7canplayE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14canplaythroughE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10ratechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14durationchangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12volumechangeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15ondataavailableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onwarningE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onstartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6onstopE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7onphotoE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20onactivestatechangedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19ongamepadbuttondownE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17ongamepadbuttonupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17ongamepadaxismoveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18ongamepadconnectedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21ongamepaddisconnectedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18animationsPropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms26animationsOfBeforePropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms25animationsOfAfterPropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24animationEffectsPropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms33animationEffectsForBeforePropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms32animationEffectsForAfterPropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms30cssPseudoElementBeforePropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms29cssPseudoElementAfterPropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19transitionsPropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms27transitionsOfBeforePropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms26transitionsOfAfterPropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms25genConInitializerPropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24labelMouseDownPtPropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15baseURIPropertyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17lockedStyleStatesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20apzCallbackTransformE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23restylableAnonymousNodeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16paintRequestTimeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8JapaneseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7ChineseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9TaiwaneseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15HongKongChineseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7UnicodeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2koE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5zh_cnE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5zh_hkE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5zh_twE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10x_cyrillicE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2heE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2arE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12x_devanagariE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7x_tamilE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_armnE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_bengE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_cansE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_ethiE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_georE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_gujrE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_guruE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_khmrE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_kndaE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_mlymE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_oryaE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_sinhE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_teluE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_tibtE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2azE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2baE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3crhE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2elE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2gaE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms2nlE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6x_mathE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13TypingTxnNameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10IMETxnNameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13DeleteTxnNameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5serifE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10sans_serifE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7cursiveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7fantasyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9monospaceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6RemoteE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8RemoteIdE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11DisplayPortE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18DisplayPortMarginsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15DisplayPortBaseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms30AsyncScrollLayerCreationFailedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19forcemessagemanagerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22color_picker_availableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24scrollbar_start_backwardE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23scrollbar_start_forwardE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22scrollbar_end_backwardE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21scrollbar_end_forwardE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms28scrollbar_thumb_proportionalE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15images_in_menusE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17images_in_buttonsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18overlay_scrollbarsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21windows_default_themeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18mac_graphite_themeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14mac_lion_themeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18mac_yosemite_themeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18windows_compositorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13windows_glassE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13touch_enabledE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12menubar_dragE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23swipe_animation_enabledE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20physical_home_buttonE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15windows_classicE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18windows_theme_aeroE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23windows_theme_aero_liteE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23windows_theme_luna_blueE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms24windows_theme_luna_oliveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms25windows_theme_luna_silverE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20windows_theme_royaleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18windows_theme_zuneE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21windows_theme_genericE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms27_moz_color_picker_availableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms29_moz_scrollbar_start_backwardE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms28_moz_scrollbar_start_forwardE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms27_moz_scrollbar_end_backwardE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms26_moz_scrollbar_end_forwardE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms33_moz_scrollbar_thumb_proportionalE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20_moz_images_in_menusE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms22_moz_images_in_buttonsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23_moz_overlay_scrollbarsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms26_moz_windows_default_themeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23_moz_mac_graphite_themeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms19_moz_mac_lion_themeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23_moz_mac_yosemite_themeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23_moz_windows_compositorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20_moz_windows_classicE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18_moz_windows_glassE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18_moz_windows_themeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15_moz_os_versionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18_moz_touch_enabledE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17_moz_menubar_dragE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23_moz_device_pixel_ratioE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms23_moz_device_orientationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms25_moz_is_resource_documentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms28_moz_swipe_animation_enabledE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms25_moz_physical_home_buttonE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4BackE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7ForwardE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6ReloadE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4StopE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6SearchE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9BookmarksE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4HomeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5ClearE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8VolumeUpE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10VolumeDownE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9NextTrackE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13PreviousTrackE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9MediaStopE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9PlayPauseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4MenuE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3NewE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4OpenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5CloseE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4SaveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4FindE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4HelpE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5PrintE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8SendMailE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11ForwardMailE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11ReplyToMailE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10mouseWheelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6pixelsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5linesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5pagesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10scrollbarsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5otherE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms3apzE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7restoreE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5alertE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11alertdialogE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11applicationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms21aria_activedescendantE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11aria_atomicE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17aria_autocompleteE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9aria_busyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12aria_checkedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_colcountE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_colindexE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_controlsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16aria_describedbyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_disabledE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15aria_dropeffectE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_expandedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11aria_flowtoE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12aria_grabbedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_haspopupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11aria_hiddenE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12aria_invalidE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10aria_labelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15aria_labelledbyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10aria_levelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9aria_liveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10aria_modalE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14aria_multilineE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20aria_multiselectableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16aria_orientationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9aria_ownsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_posinsetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12aria_pressedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_readonlyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_relevantE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_requiredE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_rowcountE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_rowindexE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_selectedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12aria_setsizeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9aria_sortE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_valuenowE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_valueminE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13aria_valuemaxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14aria_valuetextE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9AreaFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14auto_generatedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6bannerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9checkableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7choicesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12columnheaderE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13complementaryE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms15containerAtomicE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13containerBusyE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13containerLiveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17containerLiveRoleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms17containerRelevantE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11contentinfoE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6cyclesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9datatableE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14eventFromInputE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7grammarE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8gridcellE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7headingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9hitregionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16InlineBlockFrameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11inlinevalueE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7invalidE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4itemE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7itemsetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10lineNumberE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11linkedPanelE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms4liveE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16menuitemcheckboxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13menuitemradioE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5mixedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9multilineE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10navigationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6politeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8posinsetE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12presentationE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11progressbarE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6regionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8rowgroupE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9rowheaderE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6searchE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9searchboxE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7select1E: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7setsizeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8spellingE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10spinbuttonE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6statusE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7_switchE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14tableCellIndexE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms7tablistE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10textIndentE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13textInputTypeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20textLineThroughColorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms20textLineThroughStyleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12textPositionE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18textUnderlineColorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms18textUnderlineStyleE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms5timerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11toolbarnameE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms16toolbarseparatorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13toolbarspacerE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13toolbarspringE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8treegridE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10_undefinedE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8xmlrolesE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11close_fenceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11denominatorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9numeratorE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10open_fenceE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10overscriptE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12presubscriptE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms14presuperscriptE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10root_indexE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9subscriptE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11superscriptE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11underscriptE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onaudiostartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onaudioendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12onsoundstartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onsoundendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13onspeechstartE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11onspeechendE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8onresultE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9onnomatchE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8onresumeE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms6onmarkE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10onboundaryE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms8vr_stateE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms13usercontextidE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11nsuri_xmlnsE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9nsuri_xmlE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11nsuri_xhtmlE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms11nsuri_xlinkE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms10nsuri_xsltE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9nsuri_xblE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms12nsuri_mathmlE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9nsuri_rdfE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9nsuri_xulE: *mut nsIAtom; }
extern { pub static _ZN9nsGkAtoms9nsuri_svgE: *mut nsIAtom; }
#[macro_export]
macro_rules! atom {
("") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6_emptyE) };
("_moz") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3mozE) };
("mozframetype") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12mozframetypeE) };
("_moz_abspos") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11_moz_absposE) };
("_moz_activated") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14_moz_activatedE) };
("_moz_resizing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13_moz_resizingE) };
("mozallowfullscreen") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18mozallowfullscreenE) };
("_moz-type") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7moztypeE) };
("_moz_dirty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8mozdirtyE) };
("mozdisallowselectionprint") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms25mozdisallowselectionprintE) };
("moz-do-not-send") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12mozdonotsendE) };
("_moz_editor_bogus_node") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18mozeditorbogusnodeE) };
("_moz_generated_content_before") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms25mozgeneratedcontentbeforeE) };
("_moz_generated_content_after") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24mozgeneratedcontentafterE) };
("_moz_generated_content_image") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24mozgeneratedcontentimageE) };
("_moz_quote") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8mozquoteE) };
("moz-signature") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12mozsignatureE) };
("-moz-is-glyph") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13_moz_is_glyphE) };
("_moz_original_size") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18_moz_original_sizeE) };
("_moz_target") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11_moz_targetE) };
("_moz-menuactive") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10menuactiveE) };
("#default") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13_poundDefaultE) };
("*") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9_asteriskE) };
("a") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1aE) };
("abbr") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4abbrE) };
("abort") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5abortE) };
("above") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5aboveE) };
("acceltext") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9acceltextE) };
("accept") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6acceptE) };
("accept-charset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13acceptcharsetE) };
("accesskey") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9accesskeyE) };
("acronym") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7acronymE) };
("action") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6actionE) };
("active") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6activeE) };
("activetitlebarcolor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19activetitlebarcolorE) };
("activateontab") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13activateontabE) };
("actuate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7actuateE) };
("address") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7addressE) };
("after") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5afterE) };
("after_end") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9after_endE) };
("after_start") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11after_startE) };
("align") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5alignE) };
("alink") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5alinkE) };
("all") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3allE) };
("allowevents") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11alloweventsE) };
("allownegativeassertions") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23allownegativeassertionsE) };
("allow-forms") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10allowformsE) };
("allowfullscreen") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15allowfullscreenE) };
("allow-orientation-lock") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20alloworientationlockE) };
("allow-pointer-lock") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16allowpointerlockE) };
("allow-popups") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11allowpopupsE) };
("allow-same-origin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15allowsameoriginE) };
("allow-scripts") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12allowscriptsE) };
("allow-top-navigation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18allowtopnavigationE) };
("allowuntrusted") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14allowuntrustedE) };
("alt") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3altE) };
("alternate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9alternateE) };
("always") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6alwaysE) };
("ancestor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8ancestorE) };
("ancestor-or-self") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14ancestorOrSelfE) };
("anchor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6anchorE) };
("and") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4_andE) };
("animations") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10animationsE) };
("anonid") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6anonidE) };
("anonlocation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12anonlocationE) };
("any") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3anyE) };
("mozapp") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6mozappE) };
("mozwidget") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9mozwidgetE) };
("applet") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6appletE) };
("apply-imports") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12applyImportsE) };
("apply-templates") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14applyTemplatesE) };
("mozapptype") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10mozapptypeE) };
("archive") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7archiveE) };
("area") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4areaE) };
("arrow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5arrowE) };
("article") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7articleE) };
("ascending") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9ascendingE) };
("aside") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5asideE) };
("aspect-ratio") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11aspectRatioE) };
("assign") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6assignE) };
("async") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5asyncE) };
("attribute") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9attributeE) };
("attributes") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10attributesE) };
("attribute-set") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12attributeSetE) };
("aural") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5auralE) };
("auto") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5_autoE) };
("autocheck") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9autocheckE) };
("autocomplete") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12autocompleteE) };
("autofocus") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9autofocusE) };
("autoplay") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8autoplayE) };
("autorepeatbutton") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16autorepeatbuttonE) };
("axis") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4axisE) };
("b") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1bE) };
("BackdropFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13backdropFrameE) };
("background") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10backgroundE) };
("base") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4baseE) };
("basefont") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8basefontE) };
("baseline") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8baselineE) };
("bdi") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3bdiE) };
("bdo") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3bdoE) };
("before") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6beforeE) };
("before_end") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10before_endE) };
("before_start") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12before_startE) };
("below") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5belowE) };
("bgcolor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7bgcolorE) };
("bgsound") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7bgsoundE) };
("big") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3bigE) };
("binding") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7bindingE) };
("bindings") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8bindingsE) };
("bindToUntrustedContent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22bindToUntrustedContentE) };
("blankrow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8blankrowE) };
("block") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5blockE) };
("blockquote") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10blockquoteE) };
("blur") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4blurE) };
("body") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4bodyE) };
("boolean") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7booleanE) };
("border") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6borderE) };
("bordercolor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11bordercolorE) };
("both") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4bothE) };
("bottom") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6bottomE) };
("bottomend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9bottomendE) };
("bottomstart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11bottomstartE) };
("bottomleft") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10bottomleftE) };
("bottommargin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12bottommarginE) };
("bottompadding") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13bottompaddingE) };
("bottomright") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11bottomrightE) };
("box") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3boxE) };
("br") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2brE) };
("braille") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7brailleE) };
("broadcast") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9broadcastE) };
("broadcaster") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11broadcasterE) };
("broadcasterset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14broadcastersetE) };
("browser") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7browserE) };
("mozbrowser") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10mozbrowserE) };
("bulletinboard") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13bulletinboardE) };
("button") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6buttonE) };
("brighttitlebarforeground") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24brighttitlebarforegroundE) };
("call-template") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12callTemplateE) };
("cancel") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6cancelE) };
("canvas") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6canvasE) };
("caption") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7captionE) };
("capture") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7captureE) };
("case-order") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9caseOrderE) };
("cdata-section-elements") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20cdataSectionElementsE) };
("ceiling") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7ceilingE) };
("cell") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4cellE) };
("cellpadding") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11cellpaddingE) };
("cellspacing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11cellspacingE) };
("center") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6centerE) };
("ch") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2chE) };
("change") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6changeE) };
("char") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5_charE) };
("characterData") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13characterDataE) };
("charcode") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8charcodeE) };
("charoff") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7charoffE) };
("charset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7charsetE) };
("checkbox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8checkboxE) };
("checked") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7checkedE) };
("child") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5childE) };
("children") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8childrenE) };
("childList") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9childListE) };
("choose") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6chooseE) };
("chromemargin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12chromemarginE) };
("chromeOnlyContent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17chromeOnlyContentE) };
("exposeToUntrustedContent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24exposeToUntrustedContentE) };
("circ") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4circE) };
("circle") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6circleE) };
("cite") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4citeE) };
("class") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6_classE) };
("classid") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7classidE) };
("clear") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5clearE) };
("click") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5clickE) };
("clickcount") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10clickcountE) };
("clickthrough") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12clickthroughE) };
("movetoclick") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11movetoclickE) };
("clip") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4clipE) };
("close") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5closeE) };
("closed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6closedE) };
("closemenu") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9closemenuE) };
("coalesceduplicatearcs") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21coalesceduplicatearcsE) };
("code") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4codeE) };
("codebase") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8codebaseE) };
("codetype") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8codetypeE) };
("col") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3colE) };
("colgroup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8colgroupE) };
("collapse") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8collapseE) };
("collapsed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9collapsedE) };
("color") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5colorE) };
("color-index") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10colorIndexE) };
("cols") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4colsE) };
("colspan") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7colspanE) };
("column") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6columnE) };
("columns") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7columnsE) };
("combobox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8comboboxE) };
("command") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7commandE) };
("commands") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8commandsE) };
("commandset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10commandsetE) };
("commandupdate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13commandupdateE) };
("commandupdater") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14commandupdaterE) };
("comment") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7commentE) };
("compact") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7compactE) };
("concat") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6concatE) };
("conditions") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10conditionsE) };
("constructor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11constructorE) };
("consumeoutsideclicks") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20consumeoutsideclicksE) };
("container") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9containerE) };
("containment") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11containmentE) };
("contains") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8containsE) };
("content") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7contentE) };
("contenteditable") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15contenteditableE) };
("content-disposition") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24headerContentDispositionE) };
("content-language") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21headerContentLanguageE) };
("content-location") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15contentLocationE) };
("content-script-type") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23headerContentScriptTypeE) };
("content-style-type") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22headerContentStyleTypeE) };
("content-type") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17headerContentTypeE) };
("consumeanchor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13consumeanchorE) };
("context") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7contextE) };
("contextmenu") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11contextmenuE) };
("control") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7controlE) };
("controls") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8controlsE) };
("coords") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6coordsE) };
("copy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4copyE) };
("copy-of") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6copyOfE) };
("count") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5countE) };
("crop") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4cropE) };
("crossorigin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11crossoriginE) };
("curpos") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6curposE) };
("current") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7currentE) };
("cycler") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6cyclerE) };
("data") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4dataE) };
("datalist") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8datalistE) };
("data-type") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8dataTypeE) };
("date-time") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8dateTimeE) };
("datasources") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11datasourcesE) };
("datetime") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8datetimeE) };
("dblclick") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8dblclickE) };
("dd") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2ddE) };
("debug") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5debugE) };
("decimal-format") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13decimalFormatE) };
("decimal-separator") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16decimalSeparatorE) };
("deck") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4deckE) };
("declare") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7declareE) };
("decoder-doctor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13decoderDoctorE) };
("decrement") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9decrementE) };
("default") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8_defaultE) };
("default-style") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18headerDefaultStyleE) };
("defaultAction") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13defaultActionE) };
("defaultchecked") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14defaultcheckedE) };
("defaultLabel") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12defaultLabelE) };
("defaultselected") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15defaultselectedE) };
("defaultvalue") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12defaultvalueE) };
("defaultplaybackrate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19defaultplaybackrateE) };
("defer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5deferE) };
("del") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3delE) };
("descendant") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10descendantE) };
("descendant-or-self") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16descendantOrSelfE) };
("descending") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10descendingE) };
("description") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11descriptionE) };
("destructor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10destructorE) };
("details") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7detailsE) };
("device-aspect-ratio") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17deviceAspectRatioE) };
("device-height") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12deviceHeightE) };
("device-pixel-ratio") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16devicePixelRatioE) };
("device-width") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11deviceWidthE) };
("dfn") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3dfnE) };
("dialog") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6dialogE) };
("difference") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10differenceE) };
("digit") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5digitE) };
("dir") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3dirE) };
("dirAutoSetBy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12dirAutoSetByE) };
("directionality") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14directionalityE) };
("directory") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9directoryE) };
("disable-output-escaping") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21disableOutputEscapingE) };
("disabled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8disabledE) };
("disableglobalhistory") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20disableglobalhistoryE) };
("disablehistory") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14disablehistoryE) };
("display") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7displayE) };
("display-mode") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11displayModeE) };
("distinct") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8distinctE) };
("div") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3divE) };
("dl") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2dlE) };
("doctype-public") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13doctypePublicE) };
("doctype-system") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13doctypeSystemE) };
("document") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8documentE) };
("download") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8downloadE) };
("DOMAttrModified") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15DOMAttrModifiedE) };
("DOMCharacterDataModified") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24DOMCharacterDataModifiedE) };
("DOMNodeInserted") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15DOMNodeInsertedE) };
("DOMNodeInsertedIntoDocument") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms27DOMNodeInsertedIntoDocumentE) };
("DOMNodeRemoved") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14DOMNodeRemovedE) };
("DOMNodeRemovedFromDocument") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms26DOMNodeRemovedFromDocumentE) };
("DOMSubtreeModified") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18DOMSubtreeModifiedE) };
("double") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7double_E) };
("drag") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4dragE) };
("dragdrop") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8dragdropE) };
("dragend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7dragendE) };
("dragenter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9dragenterE) };
("dragevent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9drageventE) };
("dragexit") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8dragexitE) };
("draggable") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9draggableE) };
("draggesture") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11draggestureE) };
("dragging") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8draggingE) };
("dragleave") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9dragleaveE) };
("dragover") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8dragoverE) };
("dragSession") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11dragSessionE) };
("dragstart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9dragstartE) };
("drawintitlebar") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14drawintitlebarE) };
("drawtitle") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9drawtitleE) };
("drop") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4dropE) };
("dropAfter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9dropAfterE) };
("dropBefore") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10dropBeforeE) };
("dropOn") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6dropOnE) };
("dropmarker") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10dropMarkerE) };
("dt") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2dtE) };
("editable") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8editableE) };
("editing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7editingE) };
("editor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6editorE) };
("EditorDisplay-List") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17editorDisplayListE) };
("element") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7elementE) };
("element-available") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16elementAvailableE) };
("elements") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8elementsE) };
("em") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2emE) };
("embed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5embedE) };
("embossed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8embossedE) };
("empty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5emptyE) };
("encoding") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8encodingE) };
("enctype") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7enctypeE) };
("end") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3endE) };
("endEvent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8endEventE) };
("end_after") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9end_afterE) };
("end_before") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10end_beforeE) };
("equalsize") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9equalsizeE) };
("error") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5errorE) };
("even") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4evenE) };
("event") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5eventE) };
("events") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6eventsE) };
("exclude-result-prefixes") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21excludeResultPrefixesE) };
("excludes") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8excludesE) };
("expr") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4exprE) };
("expecting-system-message") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22expectingSystemMessageE) };
("extends") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7extendsE) };
("extension-element-prefixes") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24extensionElementPrefixesE) };
("face") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4faceE) };
("fallback") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8fallbackE) };
("false") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6_falseE) };
("farthest") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8farthestE) };
("field") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5fieldE) };
("fieldset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8fieldsetE) };
("figcaption") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10figcaptionE) };
("figure") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6figureE) };
("fixed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5fixedE) };
("flags") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5flagsE) };
("flex") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4flexE) };
("flexgroup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9flexgroupE) };
("flip") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4flipE) };
("floating") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8floatingE) };
("floor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5floorE) };
("flowlength") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10flowlengthE) };
("focus") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5focusE) };
("focused") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7focusedE) };
("following") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9followingE) };
("following-sibling") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16followingSiblingE) };
("font") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4fontE) };
("font-weight") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10fontWeightE) };
("fontpicker") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10fontpickerE) };
("footer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6footerE) };
("for") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4_forE) };
("for-each") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7forEachE) };
("forceOwnRefreshDriver") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21forceOwnRefreshDriverE) };
("form") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4formE) };
("formaction") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10formactionE) };
("format") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6formatE) };
("format-number") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12formatNumberE) };
("formenctype") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11formenctypeE) };
("formmethod") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10formmethodE) };
("formnovalidate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14formnovalidateE) };
("formtarget") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10formtargetE) };
("frame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5frameE) };
("frameborder") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11frameborderE) };
("frameset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8framesetE) };
("from") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4fromE) };
("fullscreenchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16fullscreenchangeE) };
("fullscreenerror") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15fullscreenerrorE) };
("function-available") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17functionAvailableE) };
("generate-id") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10generateIdE) };
("getter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6getterE) };
("glyphchar") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9glyphcharE) };
("glyphid") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7glyphidE) };
("grid") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4gridE) };
("grippy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6grippyE) };
("group") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5groupE) };
("grouping-separator") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17groupingSeparatorE) };
("grouping-size") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12groupingSizeE) };
("grow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4growE) };
("gutter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6gutterE) };
("h1") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2h1E) };
("h2") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2h2E) };
("h3") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2h3E) };
("h4") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2h4E) };
("h5") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2h5E) };
("h6") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2h6E) };
("handheld") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8handheldE) };
("HandheldFriendly") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16handheldFriendlyE) };
("handler") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7handlerE) };
("handlers") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8handlersE) };
("HARD") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4HARDE) };
("has-same-node") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11hasSameNodeE) };
("hbox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4hboxE) };
("head") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4headE) };
("header") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6headerE) };
("headers") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7headersE) };
("height") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6heightE) };
("hgroup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6hgroupE) };
("hidden") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6hiddenE) };
("hidechrome") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10hidechromeE) };
("hidecolumnpicker") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16hidecolumnpickerE) };
("high") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4highE) };
("highest") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7highestE) };
("horizontal") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10horizontalE) };
("hover") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5hoverE) };
("hr") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2hrE) };
("href") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4hrefE) };
("hreflang") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8hreflangE) };
("hspace") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6hspaceE) };
("html") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4htmlE) };
("http-equiv") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9httpEquivE) };
("i") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1iE) };
("icon") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4iconE) };
("id") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2idE) };
("if") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3_ifE) };
("iframe") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6iframeE) };
("ignorecase") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10ignorecaseE) };
("ignorekeys") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10ignorekeysE) };
("ignoreuserfocus") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15ignoreuserfocusE) };
("ilayer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6ilayerE) };
("image") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5imageE) };
("image-clicked-point") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17imageClickedPointE) };
("img") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3imgE) };
("implementation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14implementationE) };
("implements") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10implementsE) };
("import") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6importE) };
("inactivetitlebarcolor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21inactivetitlebarcolorE) };
("include") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7includeE) };
("includes") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8includesE) };
("increment") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9incrementE) };
("indent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6indentE) };
("indeterminate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13indeterminateE) };
("index") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5indexE) };
("infer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5inferE) };
("infinity") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8infinityE) };
("inherit") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7inheritE) };
("inherits") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8inheritsE) };
("inheritstyle") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12inheritstyleE) };
("initial-scale") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13initial_scaleE) };
("input") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5inputE) };
("inputmode") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9inputmodeE) };
("ins") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3insE) };
("insertafter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11insertafterE) };
("insertbefore") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12insertbeforeE) };
("instanceOf") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10instanceOfE) };
("int32") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5int32E) };
("int64") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5int64E) };
("integer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7integerE) };
("integrity") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9integrityE) };
("intersection") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12intersectionE) };
("is") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2isE) };
("iscontainer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11iscontainerE) };
("isempty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7isemptyE) };
("ismap") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5ismapE) };
("itemid") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6itemidE) };
("itemprop") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8itempropE) };
("itemref") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7itemrefE) };
("itemscope") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9itemscopeE) };
("itemtype") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8itemtypeE) };
("kbd") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3kbdE) };
("keepcurrentinview") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17keepcurrentinviewE) };
("keepobjectsalive") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16keepobjectsaliveE) };
("key") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3keyE) };
("keycode") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7keycodeE) };
("keystatuseschange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17keystatuseschangeE) };
("keydown") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7keydownE) };
("keygen") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6keygenE) };
("keypress") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8keypressE) };
("keyset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6keysetE) };
("keysystem") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9keysystemE) };
("keytext") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7keytextE) };
("keyup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5keyupE) };
("kind") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4kindE) };
("label") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5labelE) };
("lang") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4langE) };
("language") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8languageE) };
("last") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4lastE) };
("layer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5layerE) };
("LayerActivity") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13LayerActivityE) };
("layout") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6layoutE) };
("leading") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7leadingE) };
("leaf") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4leafE) };
("left") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4leftE) };
("leftmargin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10leftmarginE) };
("leftpadding") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11leftpaddingE) };
("legend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6legendE) };
("length") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6lengthE) };
("letter-value") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11letterValueE) };
("level") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5levelE) };
("li") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2liE) };
("line") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4lineE) };
("link") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4linkE) };
("list") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4listE) };
("listbox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7listboxE) };
("listboxbody") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11listboxbodyE) };
("listcell") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8listcellE) };
("listcol") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7listcolE) };
("listcols") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8listcolsE) };
("listener") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8listenerE) };
("listhead") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8listheadE) };
("listheader") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10listheaderE) };
("listing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7listingE) };
("listitem") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8listitemE) };
("listrows") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8listrowsE) };
("load") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4loadE) };
("localedir") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9localedirE) };
("local-name") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9localNameE) };
("longdesc") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8longdescE) };
("loop") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4loopE) };
("low") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3lowE) };
("lower-first") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10lowerFirstE) };
("lowest") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6lowestE) };
("lowsrc") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6lowsrcE) };
("ltr") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3ltrE) };
("lwtheme") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7lwthemeE) };
("lwthemetextcolor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16lwthemetextcolorE) };
("main") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4mainE) };
("map") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3mapE) };
("manifest") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8manifestE) };
("margin-bottom") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12marginBottomE) };
("margin-left") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10marginLeftE) };
("margin-right") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11marginRightE) };
("margin-top") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9marginTopE) };
("marginheight") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12marginheightE) };
("marginwidth") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11marginwidthE) };
("mark") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4markE) };
("marquee") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7marqueeE) };
("match") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5matchE) };
("max") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3maxE) };
("maxheight") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9maxheightE) };
("maximum-scale") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13maximum_scaleE) };
("maxlength") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9maxlengthE) };
("maxpos") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6maxposE) };
("maxwidth") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8maxwidthE) };
("mayscript") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9mayscriptE) };
("media") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5mediaE) };
("media-type") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9mediaTypeE) };
("member") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6memberE) };
("menu") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4menuE) };
("menubar") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7menubarE) };
("menubutton") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10menubuttonE) };
("menu-button") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10menuButtonE) };
("menugroup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9menugroupE) };
("menuitem") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8menuitemE) };
("menulist") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8menulistE) };
("menupopup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9menupopupE) };
("menuseparator") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13menuseparatorE) };
("message") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7messageE) };
("meta") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4metaE) };
("referrer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8referrerE) };
("referrerpolicy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14referrerpolicyE) };
("meter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5meterE) };
("method") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6methodE) };
("microdataProperties") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19microdataPropertiesE) };
("middle") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6middleE) };
("min") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3minE) };
("minheight") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9minheightE) };
("minimum-scale") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13minimum_scaleE) };
("minpos") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6minposE) };
("minus-sign") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9minusSignE) };
("minwidth") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8minwidthE) };
("mixed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6_mixedE) };
("messagemanagergroup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19messagemanagergroupE) };
("mod") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3modE) };
("mode") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4modeE) };
("modifiers") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9modifiersE) };
("monochrome") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10monochromeE) };
("mousedown") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9mousedownE) };
("mousemove") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9mousemoveE) };
("mouseout") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8mouseoutE) };
("mouseover") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9mouseoverE) };
("mousethrough") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12mousethroughE) };
("mouseup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7mouseupE) };
("mozaudiochannel") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15mozaudiochannelE) };
("mozfullscreenchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19mozfullscreenchangeE) };
("mozfullscreenerror") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18mozfullscreenerrorE) };
("mozpasspointerevents") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20mozpasspointereventsE) };
("mozpointerlockchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20mozpointerlockchangeE) };
("mozpointerlockerror") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19mozpointerlockerrorE) };
("mozprivatebrowsing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18mozprivatebrowsingE) };
("moz-opaque") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10moz_opaqueE) };
("mozactionhint") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15moz_action_hintE) };
("x-moz-errormessage") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18x_moz_errormessageE) };
("msthemecompatible") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17msthemecompatibleE) };
("multicol") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8multicolE) };
("multiple") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8multipleE) };
("muted") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5mutedE) };
("name") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4nameE) };
("namespace") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10_namespaceE) };
("namespace-alias") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14namespaceAliasE) };
("namespace-uri") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12namespaceUriE) };
("NaN") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3NaNE) };
("nativeAnonymousChildList") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24nativeAnonymousChildListE) };
("nav") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3navE) };
("negate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6negateE) };
("never") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5neverE) };
("new") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4_newE) };
("newline") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7newlineE) };
("NextBidi") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8nextBidiE) };
("no") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2noE) };
("noautofocus") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11noautofocusE) };
("noautohide") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10noautohideE) };
("norolluponanchor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16norolluponanchorE) };
("nobr") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4nobrE) };
("node") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4nodeE) };
("nodefaultsrc") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12nodefaultsrcE) };
("node-set") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7nodeSetE) };
("noembed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7noembedE) };
("noframes") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8noframesE) };
("nohref") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6nohrefE) };
("noisolation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11noisolationE) };
("nonce") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5nonceE) };
("none") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4noneE) };
("noresize") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8noresizeE) };
("normal") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6normalE) };
("normalize-space") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14normalizeSpaceE) };
("noscript") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8noscriptE) };
("noshade") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7noshadeE) };
("novalidate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10novalidateE) };
("not") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4_notE) };
("nowrap") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6nowrapE) };
("number") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6numberE) };
("null") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4nullE) };
("object") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6objectE) };
("object-type") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10objectTypeE) };
("observer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8observerE) };
("observes") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8observesE) };
("odd") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3oddE) };
("OFF") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3OFFE) };
("ol") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2olE) };
("omit-xml-declaration") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18omitXmlDeclarationE) };
("ona2dpstatuschanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19ona2dpstatuschangedE) };
("onabort") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onabortE) };
("onactivate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onactivateE) };
("onadapteradded") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14onadapteraddedE) };
("onadapterremoved") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16onadapterremovedE) };
("onafterprint") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onafterprintE) };
("onafterscriptexecute") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onafterscriptexecuteE) };
("onalerting") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onalertingE) };
("onanimationend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14onanimationendE) };
("onanimationiteration") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onanimationiterationE) };
("onanimationstart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16onanimationstartE) };
("onantennaavailablechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24onantennaavailablechangeE) };
("onAppCommand") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onAppCommandE) };
("onattributechanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onattributechangedE) };
("onattributereadreq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onattributereadreqE) };
("onattributewritereq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19onattributewritereqE) };
("onaudioprocess") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14onaudioprocessE) };
("onbeforecopy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onbeforecopyE) };
("onbeforecut") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onbeforecutE) };
("onbeforepaste") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onbeforepasteE) };
("onbeforeevicted") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15onbeforeevictedE) };
("onbeforeprint") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onbeforeprintE) };
("onbeforescriptexecute") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21onbeforescriptexecuteE) };
("onbeforeunload") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14onbeforeunloadE) };
("onblocked") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onblockedE) };
("onblur") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6onblurE) };
("onbroadcast") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onbroadcastE) };
("onbusy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6onbusyE) };
("onbufferedamountlow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19onbufferedamountlowE) };
("oncached") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8oncachedE) };
("oncallschanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14oncallschangedE) };
("oncancel") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8oncancelE) };
("oncardstatechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17oncardstatechangeE) };
("oncfstatechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15oncfstatechangeE) };
("onchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8onchangeE) };
("oncharacteristicchanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23oncharacteristicchangedE) };
("onchargingchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16onchargingchangeE) };
("onchargingtimechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onchargingtimechangeE) };
("onchecking") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10oncheckingE) };
("onclick") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onclickE) };
("onclirmodechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16onclirmodechangeE) };
("onclose") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7oncloseE) };
("oncommand") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9oncommandE) };
("oncommandupdate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15oncommandupdateE) };
("oncomplete") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10oncompleteE) };
("oncompositionend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16oncompositionendE) };
("oncompositionstart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18oncompositionstartE) };
("oncompositionupdate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19oncompositionupdateE) };
("onconfigurationchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21onconfigurationchangeE) };
("onconnect") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onconnectE) };
("onconnected") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onconnectedE) };
("onconnecting") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onconnectingE) };
("onconnectionavailable") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21onconnectionavailableE) };
("onconnectionstatechanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24onconnectionstatechangedE) };
("oncontextmenu") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13oncontextmenuE) };
("oncopy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6oncopyE) };
("oncurrentchannelchanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23oncurrentchannelchangedE) };
("oncurrentsourcechanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22oncurrentsourcechangedE) };
("oncut") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5oncutE) };
("ondatachange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12ondatachangeE) };
("ondataerror") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11ondataerrorE) };
("ondblclick") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10ondblclickE) };
("ondeleted") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9ondeletedE) };
("ondeliverysuccess") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17ondeliverysuccessE) };
("ondeliveryerror") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15ondeliveryerrorE) };
("ondevicefound") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13ondevicefoundE) };
("ondevicepaired") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14ondevicepairedE) };
("ondeviceunpaired") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16ondeviceunpairedE) };
("ondialing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9ondialingE) };
("ondisabled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10ondisabledE) };
("ondischargingtimechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23ondischargingtimechangeE) };
("ondisconnect") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12ondisconnectE) };
("ondisconnected") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14ondisconnectedE) };
("ondisconnecting") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15ondisconnectingE) };
("ondisplaypasskeyreq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19ondisplaypasskeyreqE) };
("ondownloading") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13ondownloadingE) };
("onDOMActivate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onDOMActivateE) };
("onDOMAttrModified") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17onDOMAttrModifiedE) };
("onDOMCharacterDataModified") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms26onDOMCharacterDataModifiedE) };
("onDOMFocusIn") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onDOMFocusInE) };
("onDOMFocusOut") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onDOMFocusOutE) };
("onDOMMouseScroll") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16onDOMMouseScrollE) };
("onDOMNodeInserted") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17onDOMNodeInsertedE) };
("onDOMNodeInsertedIntoDocument") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms29onDOMNodeInsertedIntoDocumentE) };
("onDOMNodeRemoved") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16onDOMNodeRemovedE) };
("onDOMNodeRemovedFromDocument") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms28onDOMNodeRemovedFromDocumentE) };
("onDOMSubtreeModified") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onDOMSubtreeModifiedE) };
("ondata") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6ondataE) };
("ondrag") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6ondragE) };
("ondragdrop") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10ondragdropE) };
("ondragend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9ondragendE) };
("ondragenter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11ondragenterE) };
("ondragexit") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10ondragexitE) };
("ondraggesture") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13ondraggestureE) };
("ondragleave") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11ondragleaveE) };
("ondragover") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10ondragoverE) };
("ondragstart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11ondragstartE) };
("ondrain") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7ondrainE) };
("ondrop") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6ondropE) };
("oneitbroadcasted") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16oneitbroadcastedE) };
("onenabled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onenabledE) };
("onenterpincodereq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17onenterpincodereqE) };
("onemergencycbmodechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23onemergencycbmodechangeE) };
("onerror") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onerrorE) };
("onevicted") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onevictedE) };
("onfacesdetected") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15onfacesdetectedE) };
("onfailed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8onfailedE) };
("onfetch") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onfetchE) };
("onfinish") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8onfinishE) };
("onfocus") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onfocusE) };
("onfrequencychange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17onfrequencychangeE) };
("onfullscreenchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onfullscreenchangeE) };
("onfullscreenerror") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17onfullscreenerrorE) };
("onspeakerforcedchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21onspeakerforcedchangeE) };
("onget") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5ongetE) };
("ongroupchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13ongroupchangeE) };
("onhashchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onhashchangeE) };
("onheadphoneschange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onheadphoneschangeE) };
("onheld") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6onheldE) };
("onhfpstatuschanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onhfpstatuschangedE) };
("onhidstatuschanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onhidstatuschangedE) };
("onholding") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onholdingE) };
("oniccchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11oniccchangeE) };
("oniccdetected") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13oniccdetectedE) };
("oniccinfochange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15oniccinfochangeE) };
("oniccundetected") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15oniccundetectedE) };
("onincoming") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onincomingE) };
("oninput") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7oninputE) };
("oninstall") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9oninstallE) };
("oninvalid") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9oninvalidE) };
("onkeydown") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onkeydownE) };
("onkeypress") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onkeypressE) };
("onkeyup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onkeyupE) };
("onlanguagechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16onlanguagechangeE) };
("onlevelchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onlevelchangeE) };
("onLoad") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6onLoadE) };
("onload") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6onloadE) };
("onloading") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onloadingE) };
("onloadingdone") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onloadingdoneE) };
("onloadingerror") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14onloadingerrorE) };
("onpopstate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onpopstateE) };
("only") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4onlyE) };
("onmessage") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onmessageE) };
("onmousedown") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onmousedownE) };
("onmouseenter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onmouseenterE) };
("onmouseleave") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onmouseleaveE) };
("onmousemove") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onmousemoveE) };
("onmouseout") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onmouseoutE) };
("onmouseover") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onmouseoverE) };
("onMozMouseHittest") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17onMozMouseHittestE) };
("onmouseup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onmouseupE) };
("onMozAfterPaint") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15onMozAfterPaintE) };
("onmozbrowserafterkeydown") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24onmozbrowserafterkeydownE) };
("onmozbrowserafterkeyup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22onmozbrowserafterkeyupE) };
("onmozbrowserbeforekeydown") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms25onmozbrowserbeforekeydownE) };
("onmozbrowserbeforekeyup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23onmozbrowserbeforekeyupE) };
("onmozfullscreenchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21onmozfullscreenchangeE) };
("onmozfullscreenerror") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onmozfullscreenerrorE) };
("onmozkeydownonplugin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onmozkeydownonpluginE) };
("onmozkeyuponplugin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onmozkeyuponpluginE) };
("onmozpointerlockchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22onmozpointerlockchangeE) };
("onmozpointerlockerror") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21onmozpointerlockerrorE) };
("onmoztimechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15onmoztimechangeE) };
("onMozMousePixelScroll") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21onMozMousePixelScrollE) };
("onMozScrolledAreaChanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24onMozScrolledAreaChangedE) };
("onmoznetworkupload") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onmoznetworkuploadE) };
("onmoznetworkdownload") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onmoznetworkdownloadE) };
("onmapfolderlistingreq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21onmapfolderlistingreqE) };
("onmapmessageslistingreq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23onmapmessageslistingreqE) };
("onmapgetmessagereq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onmapgetmessagereqE) };
("onmapsetmessagestatusreq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24onmapsetmessagestatusreqE) };
("onmapsendmessagereq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19onmapsendmessagereqE) };
("onmapmessageupdatereq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21onmapmessageupdatereqE) };
("onnewrdsgroup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onnewrdsgroupE) };
("onnotificationclick") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19onnotificationclickE) };
("onnoupdate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onnoupdateE) };
("onobexpasswordreq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17onobexpasswordreqE) };
("onobsolete") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onobsoleteE) };
("ononline") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8ononlineE) };
("onoffline") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onofflineE) };
("onopen") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6onopenE) };
("onorientationchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19onorientationchangeE) };
("onotastatuschange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17onotastatuschangeE) };
("onoverflow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onoverflowE) };
("onoverflowchanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17onoverflowchangedE) };
("onpagehide") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onpagehideE) };
("onpageshow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onpageshowE) };
("onpaint") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onpaintE) };
("onpairingaborted") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16onpairingabortedE) };
("onpairingconfirmationreq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24onpairingconfirmationreqE) };
("onpairingconsentreq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19onpairingconsentreqE) };
("onpaste") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onpasteE) };
("onpendingchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15onpendingchangeE) };
("onpichange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onpichangeE) };
("onpicture") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onpictureE) };
("onpopuphidden") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onpopuphiddenE) };
("onpopuphiding") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onpopuphidingE) };
("onpopupshowing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14onpopupshowingE) };
("onpopupshown") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onpopupshownE) };
("onposter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8onposterE) };
("onpreviewstatechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onpreviewstatechangeE) };
("onpullphonebookreq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onpullphonebookreqE) };
("onpullvcardentryreq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19onpullvcardentryreqE) };
("onpullvcardlistingreq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21onpullvcardlistingreqE) };
("onpush") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6onpushE) };
("onpushsubscriptionchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24onpushsubscriptionchangeE) };
("onpschange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onpschangeE) };
("onptychange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onptychangeE) };
("onradiostatechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onradiostatechangeE) };
("onrdsdisabled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onrdsdisabledE) };
("onrdsenabled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onrdsenabledE) };
("onreaderror") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onreaderrorE) };
("onreadsuccess") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onreadsuccessE) };
("onready") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onreadyE) };
("onreadystatechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onreadystatechangeE) };
("onreceived") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onreceivedE) };
("onrecorderstatechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21onrecorderstatechangeE) };
("onremoteheld") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onremoteheldE) };
("onremoteresumed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15onremoteresumedE) };
("onresourcetimingbufferfull") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms26onresourcetimingbufferfullE) };
("onretrieving") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onretrievingE) };
("onRequest") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onRequestE) };
("onrequestmediaplaystatus") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24onrequestmediaplaystatusE) };
("onreset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onresetE) };
("onresuming") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onresumingE) };
("onresize") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8onresizeE) };
("onrtchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onrtchangeE) };
("onscanningstatechanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22onscanningstatechangedE) };
("onscostatuschanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onscostatuschangedE) };
("onscroll") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8onscrollE) };
("onselect") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8onselectE) };
("onselectionchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17onselectionchangeE) };
("onselectstart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onselectstartE) };
("onsending") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onsendingE) };
("onsent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6onsentE) };
("onset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5onsetE) };
("onshow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6onshowE) };
("onshutter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onshutterE) };
("onstatechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onstatechangeE) };
("onstatuschanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15onstatuschangedE) };
("onstkcommand") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onstkcommandE) };
("onstksessionend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15onstksessionendE) };
("onstorage") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onstorageE) };
("onstorageareachanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onstorageareachangedE) };
("onsubmit") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8onsubmitE) };
("onsuccess") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onsuccessE) };
("ontypechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12ontypechangeE) };
("ontext") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6ontextE) };
("ontoggle") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8ontoggleE) };
("ontouchstart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12ontouchstartE) };
("ontouchend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10ontouchendE) };
("ontouchmove") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11ontouchmoveE) };
("ontouchcancel") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13ontouchcancelE) };
("ontransitionend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15ontransitionendE) };
("onunderflow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onunderflowE) };
("onunload") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8onunloadE) };
("onupdatefound") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onupdatefoundE) };
("onupdateready") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onupdatereadyE) };
("onupgradeneeded") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15onupgradeneededE) };
("onussdreceived") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14onussdreceivedE) };
("onversionchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15onversionchangeE) };
("onvoicechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onvoicechangeE) };
("onvoiceschanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15onvoiceschangedE) };
("onwebkitAnimationEnd") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onwebkitAnimationEndE) };
("onwebkitAnimationIteration") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms26onwebkitAnimationIterationE) };
("onwebkitAnimationStart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22onwebkitAnimationStartE) };
("onwebkitTransitionEnd") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21onwebkitTransitionEndE) };
("onwheel") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onwheelE) };
("open") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4openE) };
("optgroup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8optgroupE) };
("optimum") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7optimumE) };
("option") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6optionE) };
("or") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3_orE) };
("order") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5orderE) };
("ordinal") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7ordinalE) };
("orient") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6orientE) };
("orientation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11orientationE) };
("otherwise") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9otherwiseE) };
("output") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6outputE) };
("overflow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8overflowE) };
("overflowchanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15overflowchangedE) };
("overlay") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7overlayE) };
("overlap") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7overlapE) };
("p") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1pE) };
("pack") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4packE) };
("page") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4pageE) };
("pageincrement") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13pageincrementE) };
("pagex") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5pagexE) };
("pagey") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5pageyE) };
("paint-order") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11paint_orderE) };
("palettename") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11palettenameE) };
("panel") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5panelE) };
("param") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5paramE) };
("parameter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9parameterE) };
("parent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6parentE) };
("parentapp") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9parentappE) };
("parentfocused") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13parentfocusedE) };
("parsetype") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9parsetypeE) };
("password") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8passwordE) };
("pattern") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7patternE) };
("pattern-separator") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16patternSeparatorE) };
("per-mille") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8perMilleE) };
("percent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7percentE) };
("persist") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7persistE) };
("phase") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5phaseE) };
("picture") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7pictureE) };
("ping") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4pingE) };
("pinned") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6pinnedE) };
("placeholder") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11placeholderE) };
("plaintext") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9plaintextE) };
("playbackrate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12playbackrateE) };
("point-size") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9pointSizeE) };
("poly") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4polyE) };
("polygon") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7polygonE) };
("popup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5popupE) };
("popupalign") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10popupalignE) };
("popupanchor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11popupanchorE) };
("popupgroup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10popupgroupE) };
("popuphidden") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11popuphiddenE) };
("popuphiding") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11popuphidingE) };
("popupset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8popupsetE) };
("popupshowing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12popupshowingE) };
("popupshown") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10popupshownE) };
("popupsinherittooltip") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20popupsinherittooltipE) };
("position") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8positionE) };
("poster") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6posterE) };
("pre") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3preE) };
("preceding") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9precedingE) };
("preceding-sibling") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16precedingSiblingE) };
("predicate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9predicateE) };
("prefix") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6prefixE) };
("preload") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7preloadE) };
("prerendered") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11prerenderedE) };
("preserve") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8preserveE) };
("preserve-space") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13preserveSpaceE) };
("preventdefault") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14preventdefaultE) };
("primary") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7primaryE) };
("print") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5printE) };
("priority") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8priorityE) };
("processing-instruction") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21processingInstructionE) };
("profile") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7profileE) };
("progress") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8progressE) };
("progressmeter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13progressmeterE) };
("progressNormal") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14progressNormalE) };
("progressUndetermined") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20progressUndeterminedE) };
("projection") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10projectionE) };
("prompt") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6promptE) };
("propagate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9propagateE) };
("properties") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10propertiesE) };
("property") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8propertyE) };
("pubdate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7pubdateE) };
("q") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1qE) };
("query") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5queryE) };
("queryset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8querysetE) };
("querytype") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9querytypeE) };
("radio") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5radioE) };
("radiogroup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10radiogroupE) };
("range") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5rangeE) };
("readonly") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8readonlyE) };
("rect") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4rectE) };
("rectangle") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9rectangleE) };
("ref") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3refE) };
("refresh") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7refreshE) };
("rel") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3relE) };
("onreloadpage") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onreloadpageE) };
("rem") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3remE) };
("removeelement") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13removeelementE) };
("renderingobserverlist") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21renderingobserverlistE) };
("repeat") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6repeatE) };
("replace") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7replaceE) };
("required") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8requiredE) };
("reserved") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8reservedE) };
("reset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5resetE) };
("resizeafter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11resizeafterE) };
("resizebefore") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12resizebeforeE) };
("resizer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7resizerE) };
("resolution") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10resolutionE) };
("resource") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8resourceE) };
("resources") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9resourcesE) };
("result") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6resultE) };
("result-prefix") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12resultPrefixE) };
("retargetdocumentfocus") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21retargetdocumentfocusE) };
("rev") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3revE) };
("reverse") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7reverseE) };
("reversed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8reversedE) };
("richlistbox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11richlistboxE) };
("richlistitem") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12richlistitemE) };
("right") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5rightE) };
("rightmargin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11rightmarginE) };
("rightpadding") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12rightpaddingE) };
("role") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4roleE) };
("rolluponmousewheel") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18rolluponmousewheelE) };
("round") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5roundE) };
("row") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3rowE) };
("rows") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4rowsE) };
("rowspan") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7rowspanE) };
("rb") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2rbE) };
("rp") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2rpE) };
("rt") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2rtE) };
("rtc") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3rtcE) };
("rtl") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3rtlE) };
("ruby") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4rubyE) };
("ruby-base") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8rubyBaseE) };
("ruby-base-container") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17rubyBaseContainerE) };
("ruby-text") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8rubyTextE) };
("ruby-text-container") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17rubyTextContainerE) };
("rule") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4ruleE) };
("rules") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5rulesE) };
("s") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1sE) };
("samp") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4sampE) };
("sandbox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7sandboxE) };
("sbattr") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6sbattrE) };
("scale") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5scaleE) };
("scan") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4scanE) };
("scheme") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6schemeE) };
("scope") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5scopeE) };
("scoped") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6scopedE) };
("screen") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6screenE) };
("screenX") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7screenXE) };
("screenY") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7screenYE) };
("script") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6scriptE) };
("scriptEnabledBeforePrintOrPreview") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms33scriptEnabledBeforePrintOrPreviewE) };
("scrollbar") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9scrollbarE) };
("scrollbarbutton") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15scrollbarbuttonE) };
("scrollbar-down-bottom") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19scrollbarDownBottomE) };
("scrollbar-down-top") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16scrollbarDownTopE) };
("scrollbar-up-bottom") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17scrollbarUpBottomE) };
("scrollbar-up-top") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14scrollbarUpTopE) };
("scrollbox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9scrollboxE) };
("scrollcorner") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12scrollcornerE) };
("scrolling") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9scrollingE) };
("section") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7sectionE) };
("select") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6selectE) };
("selectable") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10selectableE) };
("selected") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8selectedE) };
("selectedIndex") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13selectedIndexE) };
("selectedindex") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13selectedindexE) };
("self") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4selfE) };
("seltype") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7seltypeE) };
("set-cookie") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9setcookieE) };
("setter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6setterE) };
("shape") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5shapeE) };
("show") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4showE) };
("showcaret") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9showcaretE) };
("showresizer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11showresizerE) };
("simple") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6simpleE) };
("single") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6singleE) };
("size") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4sizeE) };
("sizes") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5sizesE) };
("sizemode") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8sizemodeE) };
("sizetopopup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11sizetopopupE) };
("slider") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6sliderE) };
("small") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5smallE) };
("smooth") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6smoothE) };
("snap") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4snapE) };
("sort") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4sortE) };
("sortActive") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10sortActiveE) };
("sortDirection") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13sortDirectionE) };
("sorted") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6sortedE) };
("sorthints") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9sorthintsE) };
("sortLocked") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10sortLockedE) };
("sortResource") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12sortResourceE) };
("sortResource2") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13sortResource2E) };
("sortSeparators") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14sortSeparatorsE) };
("sortStaticsLast") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15sortStaticsLastE) };
("source") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6sourceE) };
("space") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5spaceE) };
("spacer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6spacerE) };
("span") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4spanE) };
("spellcheck") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10spellcheckE) };
("spinner") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7spinnerE) };
("split") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5splitE) };
("splitmenu") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9splitmenuE) };
("splitter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8splitterE) };
("spring") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6springE) };
("src") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3srcE) };
("srcdoc") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6srcdocE) };
("srclang") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7srclangE) };
("srcset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6srcsetE) };
("stack") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5stackE) };
("standalone") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10standaloneE) };
("standby") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7standbyE) };
("start") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5startE) };
("start_after") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11start_afterE) };
("start_before") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12start_beforeE) };
("starts-with") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10startsWithE) };
("state") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5stateE) };
("statedatasource") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15statedatasourceE) };
("staticHint") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10staticHintE) };
("statusbar") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9statusbarE) };
("statustext") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10statustextE) };
("step") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4stepE) };
("stop") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4stopE) };
("stretch") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7stretchE) };
("strike") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6strikeE) };
("string") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6stringE) };
("string-length") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12stringLengthE) };
("strip-space") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10stripSpaceE) };
("strong") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6strongE) };
("style") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5styleE) };
("stylesheet") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10stylesheetE) };
("stylesheet-prefix") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16stylesheetPrefixE) };
("subject") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7subjectE) };
("submit") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6submitE) };
("substate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8substateE) };
("substring") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9substringE) };
("substring-after") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14substringAfterE) };
("substring-before") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15substringBeforeE) };
("sub") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3subE) };
("sum") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3sumE) };
("sup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3supE) };
("summary") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7summaryE) };
("system-property") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14systemPropertyE) };
("tab") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3tabE) };
("tabbox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6tabboxE) };
("tabindex") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8tabindexE) };
("table") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5tableE) };
("tabpanel") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8tabpanelE) };
("tabpanels") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9tabpanelsE) };
("tag") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3tagE) };
("target") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6targetE) };
("targets") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7targetsE) };
("tbody") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5tbodyE) };
("td") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2tdE) };
("template") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9_templateE) };
("text-decoration") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15text_decorationE) };
("terminate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9terminateE) };
("test") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4testE) };
("text") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4textE) };
("text-align") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9textAlignE) };
("textarea") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8textareaE) };
("textbox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7textboxE) };
("textnode") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8textnodeE) };
("textNodeDirectionalityMap") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms25textNodeDirectionalityMapE) };
("tfoot") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5tfootE) };
("th") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2thE) };
("thead") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5theadE) };
("thumb") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5thumbE) };
("time") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4timeE) };
("title") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5titleE) };
("titlebar") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8titlebarE) };
("titletip") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8titletipE) };
("toggled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7toggledE) };
("token") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5tokenE) };
("tokenize") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8tokenizeE) };
("toolbar") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7toolbarE) };
("toolbarbutton") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13toolbarbuttonE) };
("toolbaritem") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11toolbaritemE) };
("toolbox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7toolboxE) };
("tooltip") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7tooltipE) };
("tooltiptext") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11tooltiptextE) };
("top") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3topE) };
("topleft") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7topleftE) };
("topmargin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9topmarginE) };
("toppadding") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10toppaddingE) };
("topright") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8toprightE) };
("tr") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2trE) };
("track") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5trackE) };
("trailing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8trailingE) };
("transform") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9transformE) };
("transform-3d") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12transform_3dE) };
("transformiix") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12transformiixE) };
("translate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9translateE) };
("transparent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11transparentE) };
("tree") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4treeE) };
("treecell") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8treecellE) };
("treechildren") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12treechildrenE) };
("treecol") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7treecolE) };
("treecolpicker") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13treecolpickerE) };
("treecols") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8treecolsE) };
("treeitem") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8treeitemE) };
("treerow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7treerowE) };
("treeseparator") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13treeseparatorE) };
("triple") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6tripleE) };
("true") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5_trueE) };
("tt") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2ttE) };
("tty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3ttyE) };
("tv") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2tvE) };
("type") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4typeE) };
("typemustmatch") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13typemustmatchE) };
("u") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1uE) };
("ul") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2ulE) };
("underflow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9underflowE) };
("undetermined") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12undeterminedE) };
("undoscope") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9undoscopeE) };
("unload") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6unloadE) };
("unparsed-entity-uri") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17unparsedEntityUriE) };
("upper-first") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10upperFirstE) };
("uri") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3uriE) };
("use") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3useE) };
("use-attribute-sets") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16useAttributeSetsE) };
("usemap") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6usemapE) };
("user-scalable") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13user_scalableE) };
("userInput") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9userInputE) };
("validate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8validateE) };
("valign") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6valignE) };
("value") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5valueE) };
("values") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6valuesE) };
("value-of") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7valueOfE) };
("valuetype") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9valuetypeE) };
("var") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3varE) };
("variable") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8variableE) };
("vbox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4vboxE) };
("vcard_name") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10vcard_nameE) };
("vendor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6vendorE) };
("vendor-url") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9vendorUrlE) };
("version") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7versionE) };
("vert") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4vertE) };
("vertical") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8verticalE) };
("audio") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5audioE) };
("video") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5videoE) };
("videocontrols") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13videocontrolsE) };
("viewport") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8viewportE) };
("viewport-height") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15viewport_heightE) };
("viewport-initial-scale") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22viewport_initial_scaleE) };
("viewport-maximum-scale") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22viewport_maximum_scaleE) };
("viewport-minimum-scale") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22viewport_minimum_scaleE) };
("viewport-user-scalable") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22viewport_user_scalableE) };
("viewport-width") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14viewport_widthE) };
("visibility") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10visibilityE) };
("visuallyselected") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16visuallyselectedE) };
("vlink") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5vlinkE) };
("vspace") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6vspaceE) };
("wbr") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3wbrE) };
("when") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4whenE) };
("where") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5whereE) };
("widget") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6widgetE) };
("width") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5widthE) };
("window") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6windowE) };
("window-target") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18headerWindowTargetE) };
("windowtype") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10windowtypeE) };
("with-param") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9withParamE) };
("wizard") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6wizardE) };
("wrap") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4wrapE) };
("x-dns-prefetch-control") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24headerDNSPrefetchControlE) };
("content-security-policy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9headerCSPE) };
("content-security-policy-report-only") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19headerCSPReportOnlyE) };
("x-frame-options") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9headerXFOE) };
("x-western") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9x_westernE) };
("xml") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3xmlE) };
("xml-stylesheet") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14xml_stylesheetE) };
("xmlns") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5xmlnsE) };
("xmp") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3xmpE) };
("xulcontentsgenerated") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20xulcontentsgeneratedE) };
("yes") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3yesE) };
("z-index") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7z_indexE) };
("zero-digit") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9zeroDigitE) };
("%") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10percentageE) };
("A") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1AE) };
("alignment-baseline") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18alignment_baselineE) };
("allowReorder") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12allowReorderE) };
("altGlyph") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8altGlyphE) };
("altGlyphDef") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11altGlyphDefE) };
("altGlyphItem") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12altGlyphItemE) };
("amplitude") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9amplitudeE) };
("animate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7animateE) };
("animateColor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12animateColorE) };
("animateMotion") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13animateMotionE) };
("animateTransform") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16animateTransformE) };
("arithmetic") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10arithmeticE) };
("atop") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4atopE) };
("azimuth") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7azimuthE) };
("B") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1BE) };
("background-color") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15backgroundColorE) };
("background-image") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16background_imageE) };
("baseFrequency") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13baseFrequencyE) };
("baseline-shift") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14baseline_shiftE) };
("bias") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4biasE) };
("caption-side") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12caption_sideE) };
("clip-path") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9clip_pathE) };
("clip-rule") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9clip_ruleE) };
("clipPath") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8clipPathE) };
("clipPathUnits") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13clipPathUnitsE) };
("cm") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2cmE) };
("color-burn") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9colorBurnE) };
("color-dodge") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10colorDodgeE) };
("color-interpolation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18colorInterpolationE) };
("color-interpolation-filters") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms25colorInterpolationFiltersE) };
("color-profile") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12colorProfileE) };
("cursor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6cursorE) };
("cx") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2cxE) };
("cy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2cyE) };
("d") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1dE) };
("darken") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6darkenE) };
("defs") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4defsE) };
("deg") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3degE) };
("desc") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4descE) };
("diffuseConstant") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15diffuseConstantE) };
("dilate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6dilateE) };
("direction") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9directionE) };
("disable") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7disableE) };
("discrete") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8discreteE) };
("divisor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7divisorE) };
("dominant-baseline") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17dominant_baselineE) };
("duplicate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9duplicateE) };
("dx") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2dxE) };
("dy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2dyE) };
("edgeMode") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8edgeModeE) };
("ellipse") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7ellipseE) };
("elevation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9elevationE) };
("erode") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5erodeE) };
("ex") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2exE) };
("exact") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5exactE) };
("exclusion") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9exclusionE) };
("exponent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8exponentE) };
("feBlend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7feBlendE) };
("feColorMatrix") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13feColorMatrixE) };
("feComponentTransfer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19feComponentTransferE) };
("feComposite") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11feCompositeE) };
("feConvolveMatrix") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16feConvolveMatrixE) };
("feDiffuseLighting") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17feDiffuseLightingE) };
("feDisplacementMap") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17feDisplacementMapE) };
("feDistantLight") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14feDistantLightE) };
("feDropShadow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12feDropShadowE) };
("feFlood") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7feFloodE) };
("feFuncA") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7feFuncAE) };
("feFuncB") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7feFuncBE) };
("feFuncG") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7feFuncGE) };
("feFuncR") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7feFuncRE) };
("feGaussianBlur") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14feGaussianBlurE) };
("feImage") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7feImageE) };
("feMerge") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7feMergeE) };
("feMergeNode") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11feMergeNodeE) };
("feMorphology") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12feMorphologyE) };
("feOffset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8feOffsetE) };
("fePointLight") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12fePointLightE) };
("feSpecularLighting") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18feSpecularLightingE) };
("feSpotLight") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11feSpotLightE) };
("feTile") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6feTileE) };
("feTurbulence") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12feTurbulenceE) };
("fill") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4fillE) };
("fill-opacity") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12fill_opacityE) };
("fill-rule") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9fill_ruleE) };
("filter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6filterE) };
("filterUnits") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11filterUnitsE) };
("float") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6_floatE) };
("flood-color") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11flood_colorE) };
("flood-opacity") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13flood_opacityE) };
("font-face") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9font_faceE) };
("font-face-format") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16font_face_formatE) };
("font-face-name") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14font_face_nameE) };
("font-face-src") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13font_face_srcE) };
("font-face-uri") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13font_face_uriE) };
("font-family") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11font_familyE) };
("font-size") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9font_sizeE) };
("font-size-adjust") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16font_size_adjustE) };
("font-stretch") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12font_stretchE) };
("font-style") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10font_styleE) };
("font-variant") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12font_variantE) };
("foreignObject") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13foreignObjectE) };
("fractalNoise") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12fractalNoiseE) };
("fx") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2fxE) };
("fy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2fyE) };
("G") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1GE) };
("g") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1gE) };
("gamma") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5gammaE) };
("generic") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8generic_E) };
("glyphRef") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8glyphRefE) };
("grad") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4gradE) };
("gradientTransform") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17gradientTransformE) };
("gradientUnits") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13gradientUnitsE) };
("hard-light") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9hardLightE) };
("hue") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3hueE) };
("hueRotate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9hueRotateE) };
("identity") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8identityE) };
("image-rendering") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15image_renderingE) };
("in") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2inE) };
("in2") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3in2E) };
("intercept") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9interceptE) };
("k1") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2k1E) };
("k2") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2k2E) };
("k3") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2k3E) };
("k4") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2k4E) };
("kernelMatrix") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12kernelMatrixE) };
("kernelUnitLength") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16kernelUnitLengthE) };
("lengthAdjust") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12lengthAdjustE) };
("letter-spacing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14letter_spacingE) };
("lighten") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7lightenE) };
("lighting-color") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14lighting_colorE) };
("limitingConeAngle") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17limitingConeAngleE) };
("linear") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6linearE) };
("linearGradient") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14linearGradientE) };
("linearRGB") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9linearRGBE) };
("list-style-type") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15list_style_typeE) };
("luminanceToAlpha") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16luminanceToAlphaE) };
("luminosity") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10luminosityE) };
("magnify") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7magnifyE) };
("marker") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6markerE) };
("marker-end") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10marker_endE) };
("marker-mid") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10marker_midE) };
("marker-start") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12marker_startE) };
("markerHeight") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12markerHeightE) };
("markerUnits") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11markerUnitsE) };
("markerWidth") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11markerWidthE) };
("mask") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4maskE) };
("maskContentUnits") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16maskContentUnitsE) };
("mask-type") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9mask_typeE) };
("maskUnits") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9maskUnitsE) };
("matrix") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6matrixE) };
("metadata") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8metadataE) };
("missing-glyph") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12missingGlyphE) };
("mm") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2mmE) };
("mpath") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5mpathE) };
("noStitch") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8noStitchE) };
("numOctaves") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10numOctavesE) };
("multiply") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8multiplyE) };
("objectBoundingBox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17objectBoundingBoxE) };
("offset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6offsetE) };
("onSVGLoad") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onSVGLoadE) };
("onSVGResize") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onSVGResizeE) };
("onSVGScroll") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onSVGScrollE) };
("onSVGUnload") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onSVGUnloadE) };
("onSVGZoom") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onSVGZoomE) };
("onzoom") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6onzoomE) };
("opacity") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7opacityE) };
("operator") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9_operatorE) };
("out") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3outE) };
("over") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4overE) };
("overridePreserveAspectRatio") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms27overridePreserveAspectRatioE) };
("pad") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3padE) };
("path") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4pathE) };
("pathLength") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10pathLengthE) };
("patternContentUnits") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19patternContentUnitsE) };
("patternTransform") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16patternTransformE) };
("patternUnits") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12patternUnitsE) };
("pc") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2pcE) };
("pointer-events") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14pointer_eventsE) };
("points") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6pointsE) };
("pointsAtX") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9pointsAtXE) };
("pointsAtY") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9pointsAtYE) };
("pointsAtZ") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9pointsAtZE) };
("polyline") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8polylineE) };
("preserveAlpha") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13preserveAlphaE) };
("preserveAspectRatio") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19preserveAspectRatioE) };
("primitiveUnits") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14primitiveUnitsE) };
("pt") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2ptE) };
("px") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2pxE) };
("R") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1RE) };
("r") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1rE) };
("rad") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3radE) };
("radialGradient") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14radialGradientE) };
("radius") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6radiusE) };
("reflect") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7reflectE) };
("refX") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4refXE) };
("refY") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4refYE) };
("requiredExtensions") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18requiredExtensionsE) };
("requiredFeatures") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16requiredFeaturesE) };
("rotate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6rotateE) };
("rx") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2rxE) };
("ry") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2ryE) };
("saturate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8saturateE) };
("saturation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10saturationE) };
("set") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3setE) };
("seed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4seedE) };
("shadow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6shadowE) };
("shape-rendering") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15shape_renderingE) };
("skewX") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5skewXE) };
("skewY") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5skewYE) };
("slope") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5slopeE) };
("soft-light") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9softLightE) };
("spacing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7spacingE) };
("spacingAndGlyphs") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16spacingAndGlyphsE) };
("specularConstant") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16specularConstantE) };
("specularExponent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16specularExponentE) };
("spreadMethod") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12spreadMethodE) };
("sRGB") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4sRGBE) };
("startOffset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11startOffsetE) };
("stdDeviation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12stdDeviationE) };
("stitch") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6stitchE) };
("stitchTiles") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11stitchTilesE) };
("stop-color") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10stop_colorE) };
("stop-opacity") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12stop_opacityE) };
("stroke") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6strokeE) };
("stroke-dasharray") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16stroke_dasharrayE) };
("stroke-dashoffset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17stroke_dashoffsetE) };
("stroke-linecap") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14stroke_linecapE) };
("stroke-linejoin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15stroke_linejoinE) };
("stroke-miterlimit") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17stroke_miterlimitE) };
("stroke-opacity") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14stroke_opacityE) };
("stroke-width") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12stroke_widthE) };
("strokeWidth") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11strokeWidthE) };
("surfaceScale") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12surfaceScaleE) };
("svg") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3svgE) };
("switch") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9svgSwitchE) };
("symbol") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6symbolE) };
("systemLanguage") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14systemLanguageE) };
("tableValues") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11tableValuesE) };
("targetX") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7targetXE) };
("targetY") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7targetYE) };
("text-anchor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11text_anchorE) };
("text-rendering") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14text_renderingE) };
("textLength") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10textLengthE) };
("textPath") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8textPathE) };
("tref") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4trefE) };
("tspan") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5tspanE) };
("turbulence") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10turbulenceE) };
("unicode-bidi") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12unicode_bidiE) };
("userSpaceOnUse") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14userSpaceOnUseE) };
("view") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4viewE) };
("viewBox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7viewBoxE) };
("viewTarget") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10viewTargetE) };
("white-space") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11white_spaceE) };
("word-spacing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12word_spacingE) };
("writing-mode") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12writing_modeE) };
("x") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1xE) };
("x1") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2x1E) };
("x2") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2x2E) };
("xChannelSelector") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16xChannelSelectorE) };
("xor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4xor_E) };
("y") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1yE) };
("y1") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2y1E) };
("y2") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2y2E) };
("yChannelSelector") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16yChannelSelectorE) };
("z") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms1zE) };
("zoomAndPan") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10zoomAndPanE) };
("vector-effect") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13vector_effectE) };
("vertical-align") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14vertical_alignE) };
("accumulate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10accumulateE) };
("additive") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8additiveE) };
("attributeName") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13attributeNameE) };
("attributeType") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13attributeTypeE) };
("auto-reverse") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12auto_reverseE) };
("begin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5beginE) };
("beginEvent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10beginEventE) };
("by") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2byE) };
("calcMode") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8calcModeE) };
("CSS") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3cssE) };
("dur") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3durE) };
("keyPoints") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9keyPointsE) };
("keySplines") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10keySplinesE) };
("keyTimes") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8keyTimesE) };
("_mozAnimateMotionDummyAttr") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms25mozAnimateMotionDummyAttrE) };
("onbegin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onbeginE) };
("onbeginEvent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onbeginEventE) };
("onend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5onendE) };
("onendEvent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onendEventE) };
("onrepeat") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8onrepeatE) };
("onrepeatEvent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onrepeatEventE) };
("repeatCount") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11repeatCountE) };
("repeatDur") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9repeatDurE) };
("repeatEvent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11repeatEventE) };
("restart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7restartE) };
("to") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2toE) };
("XML") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3XMLE) };
("abs") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4abs_E) };
("accent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7accent_E) };
("accentunder") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12accentunder_E) };
("actiontype") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11actiontype_E) };
("alignmentscope") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15alignmentscope_E) };
("altimg") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7altimg_E) };
("altimg-height") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14altimg_height_E) };
("altimg-valign") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14altimg_valign_E) };
("altimg-width") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13altimg_width_E) };
("annotation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11annotation_E) };
("annotation-xml") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15annotation_xml_E) };
("apply") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6apply_E) };
("approx") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7approx_E) };
("arccos") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7arccos_E) };
("arccosh") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8arccosh_E) };
("arccot") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7arccot_E) };
("arccoth") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8arccoth_E) };
("arccsc") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7arccsc_E) };
("arccsch") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8arccsch_E) };
("arcsec") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7arcsec_E) };
("arcsech") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8arcsech_E) };
("arcsin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7arcsin_E) };
("arcsinh") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8arcsinh_E) };
("arctan") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7arctan_E) };
("arctanh") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8arctanh_E) };
("arg") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4arg_E) };
("bevelled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9bevelled_E) };
("bind") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5bind_E) };
("bvar") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5bvar_E) };
("card") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5card_E) };
("cartesianproduct") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17cartesianproduct_E) };
("cbytes") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7cbytes_E) };
("cd") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3cd_E) };
("cdgroup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8cdgroup_E) };
("cerror") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7cerror_E) };
("charalign") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10charalign_E) };
("ci") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3ci_E) };
("closure") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8closure_E) };
("cn") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3cn_E) };
("codomain") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9codomain_E) };
("columnalign") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12columnalign_E) };
("columnalignment") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16columnalignment_E) };
("columnlines") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12columnlines_E) };
("columnspacing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14columnspacing_E) };
("columnspan") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11columnspan_E) };
("columnwidth") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12columnwidth_E) };
("complexes") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10complexes_E) };
("compose") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8compose_E) };
("condition") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10condition_E) };
("conjugate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10conjugate_E) };
("cos") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4cos_E) };
("cosh") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5cosh_E) };
("cot") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4cot_E) };
("coth") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5coth_E) };
("crossout") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9crossout_E) };
("csc") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4csc_E) };
("csch") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5csch_E) };
("cs") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3cs_E) };
("csymbol") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8csymbol_E) };
("curl") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5curl_E) };
("decimalpoint") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13decimalpoint_E) };
("definitionURL") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14definitionURL_E) };
("degree") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7degree_E) };
("denomalign") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11denomalign_E) };
("depth") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6depth_E) };
("determinant") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12determinant_E) };
("diff") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5diff_E) };
("displaystyle") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13displaystyle_E) };
("divergence") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11divergence_E) };
("divide") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7divide_E) };
("domain") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7domain_E) };
("domainofapplication") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20domainofapplication_E) };
("edge") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5edge_E) };
("el") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3el_E) };
("emptyset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9emptyset_E) };
("eq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3eq_E) };
("equalcolumns") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13equalcolumns_E) };
("equalrows") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10equalrows_E) };
("equivalent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11equivalent_E) };
("eulergamma") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11eulergamma_E) };
("exists") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7exists_E) };
("exp") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4exp_E) };
("exponentiale") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13exponentiale_E) };
("factorial") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10factorial_E) };
("factorof") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9factorof_E) };
("fence") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6fence_E) };
("fn") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3fn_E) };
("fontfamily") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11fontfamily_E) };
("fontsize") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9fontsize_E) };
("fontstyle") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10fontstyle_E) };
("fontweight") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11fontweight_E) };
("forall") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7forall_E) };
("framespacing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13framespacing_E) };
("gcd") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4gcd_E) };
("geq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4geq_E) };
("groupalign") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11groupalign_E) };
("gt") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3gt_E) };
("ident") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6ident_E) };
("imaginaryi") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11imaginaryi_E) };
("imaginary") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10imaginary_E) };
("implies") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8implies_E) };
("indentalignfirst") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17indentalignfirst_E) };
("indentalign") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12indentalign_E) };
("indentalignlast") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16indentalignlast_E) };
("indentshiftfirst") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17indentshiftfirst_E) };
("indentshift") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12indentshift_E) };
("indenttarget") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13indenttarget_E) };
("integers") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9integers_E) };
("intersect") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10intersect_E) };
("interval") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9interval_E) };
("int") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4int_E) };
("inverse") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8inverse_E) };
("lambda") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7lambda_E) };
("laplacian") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10laplacian_E) };
("largeop") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8largeop_E) };
("lcm") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4lcm_E) };
("leq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4leq_E) };
("limit") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6limit_E) };
("linebreak") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10linebreak_E) };
("linebreakmultchar") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18linebreakmultchar_E) };
("linebreakstyle") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15linebreakstyle_E) };
("linethickness") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14linethickness_E) };
("list") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5list_E) };
("ln") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3ln_E) };
("location") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9location_E) };
("logbase") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8logbase_E) };
("log") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4log_E) };
("longdivstyle") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13longdivstyle_E) };
("lowlimit") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9lowlimit_E) };
("lquote") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7lquote_E) };
("lspace") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7lspace_E) };
("lt") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3lt_E) };
("maction") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8maction_E) };
("maligngroup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12maligngroup_E) };
("malignmark") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11malignmark_E) };
("mathbackground") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15mathbackground_E) };
("mathcolor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10mathcolor_E) };
("mathsize") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9mathsize_E) };
("mathvariant") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12mathvariant_E) };
("matrixrow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10matrixrow_E) };
("maxsize") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8maxsize_E) };
("mean") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5mean_E) };
("median") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7median_E) };
("menclose") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9menclose_E) };
("merror") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7merror_E) };
("mfenced") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8mfenced_E) };
("mfrac") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6mfrac_E) };
("mglyph") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7mglyph_E) };
("mi") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3mi_E) };
("minlabelspacing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16minlabelspacing_E) };
("minsize") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8minsize_E) };
("minus") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6minus_E) };
("mlabeledtr") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11mlabeledtr_E) };
("mlongdiv") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9mlongdiv_E) };
("mmultiscripts") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14mmultiscripts_E) };
("mn") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3mn_E) };
("momentabout") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12momentabout_E) };
("moment") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7moment_E) };
("mo") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3mo_E) };
("movablelimits") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14movablelimits_E) };
("mover") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6mover_E) };
("mpadded") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8mpadded_E) };
("mphantom") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9mphantom_E) };
("mprescripts") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12mprescripts_E) };
("mroot") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6mroot_E) };
("mrow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5mrow_E) };
("mscarries") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10mscarries_E) };
("mscarry") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8mscarry_E) };
("msgroup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8msgroup_E) };
("msline") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7msline_E) };
("ms") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3ms_E) };
("mspace") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7mspace_E) };
("msqrt") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6msqrt_E) };
("msrow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6msrow_E) };
("mstack") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7mstack_E) };
("mstyle") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7mstyle_E) };
("msub") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5msub_E) };
("msubsup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8msubsup_E) };
("msup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5msup_E) };
("mtable") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7mtable_E) };
("mtd") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4mtd_E) };
("mtext") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6mtext_E) };
("mtr") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4mtr_E) };
("munder") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7munder_E) };
("munderover") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11munderover_E) };
("naturalnumbers") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15naturalnumbers_E) };
("neq") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4neq_E) };
("notanumber") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11notanumber_E) };
("notation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9notation_E) };
("note") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5note_E) };
("notin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6notin_E) };
("notprsubset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12notprsubset_E) };
("notsubset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10notsubset_E) };
("numalign") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9numalign_E) };
("other") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6other_E) };
("outerproduct") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13outerproduct_E) };
("partialdiff") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12partialdiff_E) };
("piece") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6piece_E) };
("piecewise") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10piecewise_E) };
("pi") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3pi_E) };
("plus") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5plus_E) };
("power") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6power_E) };
("primes") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7primes_E) };
("product") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8product_E) };
("prsubset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9prsubset_E) };
("quotient") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9quotient_E) };
("rationals") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10rationals_E) };
("real") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5real_E) };
("reals") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6reals_E) };
("reln") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5reln_E) };
("root") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5root_E) };
("rowalign") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9rowalign_E) };
("rowlines") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9rowlines_E) };
("rowspacing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11rowspacing_E) };
("rquote") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7rquote_E) };
("rspace") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7rspace_E) };
("scalarproduct") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14scalarproduct_E) };
("schemaLocation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15schemaLocation_E) };
("scriptlevel") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12scriptlevel_E) };
("scriptminsize") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14scriptminsize_E) };
("scriptsizemultiplier") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21scriptsizemultiplier_E) };
("scriptsize") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11scriptsize_E) };
("sdev") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5sdev_E) };
("sech") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5sech_E) };
("sec") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4sec_E) };
("selection") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10selection_E) };
("selector") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9selector_E) };
("semantics") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10semantics_E) };
("separator") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10separator_E) };
("separators") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11separators_E) };
("sep") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4sep_E) };
("setdiff") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8setdiff_E) };
("set") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4set_E) };
("share") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6share_E) };
("shift") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6shift_E) };
("side") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5side_E) };
("sinh") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5sinh_E) };
("sin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4sin_E) };
("stackalign") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11stackalign_E) };
("stretchy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9stretchy_E) };
("subscriptshift") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15subscriptshift_E) };
("subset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7subset_E) };
("superscriptshift") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17superscriptshift_E) };
("symmetric") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10symmetric_E) };
("tanh") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5tanh_E) };
("tan") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4tan_E) };
("tendsto") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8tendsto_E) };
("times") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6times_E) };
("transpose") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10transpose_E) };
("union") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6union_E) };
("uplimit") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8uplimit_E) };
("variance") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9variance_E) };
("vectorproduct") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14vectorproduct_E) };
("vector") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7vector_E) };
("voffset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8voffset_E) };
("xref") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5xref_E) };
("math") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4mathE) };
("avg") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3avgE) };
("boolean-from-string") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17booleanFromStringE) };
("count-non-empty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13countNonEmptyE) };
("days-from-date") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12daysFromDateE) };
("init") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4initE) };
("instance") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8instanceE) };
("months") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6monthsE) };
("now") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3nowE) };
("seconds") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7secondsE) };
("seconds-from-dateTime") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19secondsFromDateTimeE) };
("onMozSwipeGestureMayStart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms25onMozSwipeGestureMayStartE) };
("onMozSwipeGestureStart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22onMozSwipeGestureStartE) };
("onMozSwipeGestureUpdate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23onMozSwipeGestureUpdateE) };
("onMozSwipeGestureEnd") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onMozSwipeGestureEndE) };
("onMozSwipeGesture") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17onMozSwipeGestureE) };
("onMozMagnifyGestureStart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24onMozMagnifyGestureStartE) };
("onMozMagnifyGestureUpdate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms25onMozMagnifyGestureUpdateE) };
("onMozMagnifyGesture") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19onMozMagnifyGestureE) };
("onMozRotateGestureStart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23onMozRotateGestureStartE) };
("onMozRotateGestureUpdate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24onMozRotateGestureUpdateE) };
("onMozRotateGesture") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onMozRotateGestureE) };
("onMozTapGesture") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15onMozTapGestureE) };
("onMozPressTapGesture") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onMozPressTapGestureE) };
("onMozEdgeUIStarted") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18onMozEdgeUIStartedE) };
("onMozEdgeUICanceled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19onMozEdgeUICanceledE) };
("onMozEdgeUICompleted") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onMozEdgeUICompletedE) };
("onpointerdown") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onpointerdownE) };
("onpointermove") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onpointermoveE) };
("onpointerup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onpointerupE) };
("onpointercancel") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15onpointercancelE) };
("onpointerover") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onpointeroverE) };
("onpointerout") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onpointeroutE) };
("onpointerenter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14onpointerenterE) };
("onpointerleave") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14onpointerleaveE) };
("ongotpointercapture") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19ongotpointercaptureE) };
("onlostpointercapture") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onlostpointercaptureE) };
("ondevicemotion") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14ondevicemotionE) };
("ondeviceorientation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19ondeviceorientationE) };
("onabsolutedeviceorientation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms27onabsolutedeviceorientationE) };
("ondeviceproximity") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17ondeviceproximityE) };
("onmozorientationchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22onmozorientationchangeE) };
("onuserproximity") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15onuserproximityE) };
("ondevicelight") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13ondevicelightE) };
("onmozinterruptbegin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19onmozinterruptbeginE) };
("onmozinterruptend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17onmozinterruptendE) };
("#cdata-section") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12cdataTagNameE) };
("#comment") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14commentTagNameE) };
("#document") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16documentNodeNameE) };
("#document-fragment") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24documentFragmentNodeNameE) };
("#document-type") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20documentTypeNodeNameE) };
("#processing-instruction") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms28processingInstructionTagNameE) };
("#text") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11textTagNameE) };
("BCTableCellFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16bcTableCellFrameE) };
("BlockFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10blockFrameE) };
("BoxFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8boxFrameE) };
("BRFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7brFrameE) };
("BulletFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11bulletFrameE) };
("colorControlFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17colorControlFrameE) };
("ColumnSetFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14columnSetFrameE) };
("ComboboxControlFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20comboboxControlFrameE) };
("ComboboxDisplayFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20comboboxDisplayFrameE) };
("DeckFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9deckFrameE) };
("DetailsFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12detailsFrameE) };
("FieldSetFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13fieldSetFrameE) };
("FlexContainerFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18flexContainerFrameE) };
("FormControlFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16formControlFrameE) };
("FrameSetFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13frameSetFrameE) };
("gfxButtonControlFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21gfxButtonControlFrameE) };
("GridContainerFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18gridContainerFrameE) };
("HTMLButtonControlFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22HTMLButtonControlFrameE) };
("HTMLCanvasFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15HTMLCanvasFrameE) };
("subDocumentFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16subDocumentFrameE) };
("ImageBoxFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13imageBoxFrameE) };
("ImageFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10imageFrameE) };
("ImageControlFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17imageControlFrameE) };
("InlineFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11inlineFrameE) };
("LeafBoxFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12leafBoxFrameE) };
("LegendFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11legendFrameE) };
("LetterFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11letterFrameE) };
("LineFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9lineFrameE) };
("ListControlFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16listControlFrameE) };
("MenuFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9menuFrameE) };
("MeterFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10meterFrameE) };
("MenuPopupFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14menuPopupFrameE) };
("NumberControlFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18numberControlFrameE) };
("ObjectFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11objectFrameE) };
("PageFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9pageFrameE) };
("PageBreakFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14pageBreakFrameE) };
("PageContentFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16pageContentFrameE) };
("PlaceholderFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16placeholderFrameE) };
("PopupSetFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13popupSetFrameE) };
("ProgressFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13progressFrameE) };
("CanvasFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11canvasFrameE) };
("RangeFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10rangeFrameE) };
("RootFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9rootFrameE) };
("RubyBaseContainerFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22rubyBaseContainerFrameE) };
("RubyBaseFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13rubyBaseFrameE) };
("RubyFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9rubyFrameE) };
("RubyTextContainerFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22rubyTextContainerFrameE) };
("RubyTextFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13rubyTextFrameE) };
("ScrollFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11scrollFrameE) };
("ScrollbarFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14scrollbarFrameE) };
("SequenceFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13sequenceFrameE) };
("sliderFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11sliderFrameE) };
("TableCellFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14tableCellFrameE) };
("TableColFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13tableColFrameE) };
("TableColGroupFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18tableColGroupFrameE) };
("TableFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10tableFrameE) };
("TableOuterFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15tableOuterFrameE) };
("TableRowGroupFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18tableRowGroupFrameE) };
("TableRowFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13tableRowFrameE) };
("TextInputFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14textInputFrameE) };
("TextFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9textFrameE) };
("ViewportFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13viewportFrameE) };
("XULLabelFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13XULLabelFrameE) };
("SVGAFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9svgAFrameE) };
("SVGClipPathFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16svgClipPathFrameE) };
("SVGDefsFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12svgDefsFrameE) };
("SVGFEContainerFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19svgFEContainerFrameE) };
("SVGFEImageFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15svgFEImageFrameE) };
("SVGFELeafFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14svgFELeafFrameE) };
("SVGFEUnstyledLeafFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22svgFEUnstyledLeafFrameE) };
("SVGFilterFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14svgFilterFrameE) };
("SVGForeignObjectFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21svgForeignObjectFrameE) };
("SVGGenericContainerFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24svgGenericContainerFrameE) };
("SVGGFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9svgGFrameE) };
("SVGGradientFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16svgGradientFrameE) };
("SVGImageFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13svgImageFrameE) };
("SVGInnerSVGFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16svgInnerSVGFrameE) };
("SVGLinearGradientFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22svgLinearGradientFrameE) };
("SVGMarkerFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14svgMarkerFrameE) };
("SVGMarkerAnonChildFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23svgMarkerAnonChildFrameE) };
("SVGMaskFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12svgMaskFrameE) };
("SVGOuterSVGFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16svgOuterSVGFrameE) };
("SVGOuterSVGAnonChildFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms25svgOuterSVGAnonChildFrameE) };
("SVGPathGeometryFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20svgPathGeometryFrameE) };
("SVGPatternFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15svgPatternFrameE) };
("SVGRadialGradientFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22svgRadialGradientFrameE) };
("SVGStopFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12svgStopFrameE) };
("SVGSwitchFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14svgSwitchFrameE) };
("SVGTextFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12svgTextFrameE) };
("SVGUseFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11svgUseFrameE) };
("SVGViewFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12svgViewFrameE) };
("VideoFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14HTMLVideoFrameE) };
("onloadend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onloadendE) };
("onloadstart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onloadstartE) };
("onprogress") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onprogressE) };
("onsuspend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onsuspendE) };
("onemptied") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onemptiedE) };
("onstalled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onstalledE) };
("onplay") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6onplayE) };
("onpause") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onpauseE) };
("onloadedmetadata") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16onloadedmetadataE) };
("onloadeddata") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onloadeddataE) };
("onwaiting") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onwaitingE) };
("onplaying") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onplayingE) };
("oncanplay") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9oncanplayE) };
("oncanplaythrough") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16oncanplaythroughE) };
("onseeking") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onseekingE) };
("onseeked") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8onseekedE) };
("ontimeout") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9ontimeoutE) };
("ontimeupdate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12ontimeupdateE) };
("onended") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onendedE) };
("onratechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onratechangeE) };
("ondurationchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16ondurationchangeE) };
("onvolumechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14onvolumechangeE) };
("onaddtrack") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onaddtrackE) };
("oncontrollerchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18oncontrollerchangeE) };
("oncuechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11oncuechangeE) };
("onenter") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onenterE) };
("onexit") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6onexitE) };
("onencrypted") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onencryptedE) };
("encrypted") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9encryptedE) };
("onremovetrack") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onremovetrackE) };
("loadstart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9loadstartE) };
("suspend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7suspendE) };
("emptied") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7emptiedE) };
("stalled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7stalledE) };
("play") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4playE) };
("pause") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5pauseE) };
("loadedmetadata") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14loadedmetadataE) };
("loadeddata") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10loadeddataE) };
("waiting") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7waitingE) };
("playing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7playingE) };
("seeking") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7seekingE) };
("seeked") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6seekedE) };
("timeupdate") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10timeupdateE) };
("ended") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5endedE) };
("canplay") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7canplayE) };
("canplaythrough") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14canplaythroughE) };
("ratechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10ratechangeE) };
("durationchange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14durationchangeE) };
("volumechange") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12volumechangeE) };
("ondataavailable") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15ondataavailableE) };
("onwarning") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onwarningE) };
("onstart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onstartE) };
("onstop") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6onstopE) };
("onphoto") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7onphotoE) };
("onactivestatechanged") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20onactivestatechangedE) };
("ongamepadbuttondown") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19ongamepadbuttondownE) };
("ongamepadbuttonup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17ongamepadbuttonupE) };
("ongamepadaxismove") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17ongamepadaxismoveE) };
("ongamepadconnected") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18ongamepadconnectedE) };
("ongamepaddisconnected") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21ongamepaddisconnectedE) };
("AnimationsProperty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18animationsPropertyE) };
("AnimationsOfBeforeProperty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms26animationsOfBeforePropertyE) };
("AnimationsOfAfterProperty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms25animationsOfAfterPropertyE) };
("AnimationEffectsProperty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24animationEffectsPropertyE) };
("AnimationsEffectsForBeforeProperty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms33animationEffectsForBeforePropertyE) };
("AnimationsEffectsForAfterProperty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms32animationEffectsForAfterPropertyE) };
("CSSPseudoElementBeforeProperty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms30cssPseudoElementBeforePropertyE) };
("CSSPseudoElementAfterProperty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms29cssPseudoElementAfterPropertyE) };
("TransitionsProperty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19transitionsPropertyE) };
("TransitionsOfBeforeProperty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms27transitionsOfBeforePropertyE) };
("TransitionsOfAfterProperty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms26transitionsOfAfterPropertyE) };
("QuoteNodeProperty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms25genConInitializerPropertyE) };
("LabelMouseDownPtProperty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24labelMouseDownPtPropertyE) };
("baseURIProperty") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15baseURIPropertyE) };
("lockedStyleStates") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17lockedStyleStatesE) };
("apzCallbackTransform") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20apzCallbackTransformE) };
("restylableAnonymousNode") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23restylableAnonymousNodeE) };
("PaintRequestTime") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16paintRequestTimeE) };
("ja") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8JapaneseE) };
("zh-CN") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7ChineseE) };
("zh-TW") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9TaiwaneseE) };
("zh-HK") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15HongKongChineseE) };
("x-unicode") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7UnicodeE) };
("ko") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2koE) };
("zh-cn") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5zh_cnE) };
("zh-hk") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5zh_hkE) };
("zh-tw") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5zh_twE) };
("x-cyrillic") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10x_cyrillicE) };
("he") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2heE) };
("ar") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2arE) };
("x-devanagari") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12x_devanagariE) };
("x-tamil") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7x_tamilE) };
("x-armn") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_armnE) };
("x-beng") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_bengE) };
("x-cans") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_cansE) };
("x-ethi") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_ethiE) };
("x-geor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_georE) };
("x-gujr") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_gujrE) };
("x-guru") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_guruE) };
("x-khmr") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_khmrE) };
("x-knda") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_kndaE) };
("x-mlym") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_mlymE) };
("x-orya") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_oryaE) };
("x-sinh") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_sinhE) };
("x-telu") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_teluE) };
("x-tibt") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_tibtE) };
("az") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2azE) };
("ba") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2baE) };
("crh") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3crhE) };
("el") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2elE) };
("ga") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2gaE) };
("nl") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms2nlE) };
("x-math") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6x_mathE) };
("Typing") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13TypingTxnNameE) };
("IME") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10IMETxnNameE) };
("Deleting") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13DeleteTxnNameE) };
("serif") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5serifE) };
("sans-serif") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10sans_serifE) };
("cursive") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7cursiveE) };
("fantasy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7fantasyE) };
("monospace") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9monospaceE) };
("remote") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6RemoteE) };
("_remote_id") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8RemoteIdE) };
("_displayport") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11DisplayPortE) };
("_displayportmargins") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18DisplayPortMarginsE) };
("_displayportbase") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15DisplayPortBaseE) };
("_asyncscrolllayercreationfailed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms30AsyncScrollLayerCreationFailedE) };
("forcemessagemanager") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19forcemessagemanagerE) };
("color-picker-available") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22color_picker_availableE) };
("scrollbar-start-backward") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24scrollbar_start_backwardE) };
("scrollbar-start-forward") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23scrollbar_start_forwardE) };
("scrollbar-end-backward") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22scrollbar_end_backwardE) };
("scrollbar-end-forward") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21scrollbar_end_forwardE) };
("scrollbar-thumb-proportional") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms28scrollbar_thumb_proportionalE) };
("images-in-menus") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15images_in_menusE) };
("images-in-buttons") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17images_in_buttonsE) };
("overlay-scrollbars") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18overlay_scrollbarsE) };
("windows-default-theme") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21windows_default_themeE) };
("mac-graphite-theme") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18mac_graphite_themeE) };
("mac-lion-theme") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14mac_lion_themeE) };
("mac-yosemite-theme") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18mac_yosemite_themeE) };
("windows-compositor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18windows_compositorE) };
("windows-glass") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13windows_glassE) };
("touch-enabled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13touch_enabledE) };
("menubar-drag") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12menubar_dragE) };
("swipe-animation-enabled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23swipe_animation_enabledE) };
("physical-home-button") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20physical_home_buttonE) };
("windows-classic") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15windows_classicE) };
("windows-theme-aero") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18windows_theme_aeroE) };
("windows-theme-aero-lite") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23windows_theme_aero_liteE) };
("windows-theme-luna-blue") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23windows_theme_luna_blueE) };
("windows-theme-luna-olive") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms24windows_theme_luna_oliveE) };
("windows-theme-luna-silver") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms25windows_theme_luna_silverE) };
("windows-theme-royale") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20windows_theme_royaleE) };
("windows-theme-zune") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18windows_theme_zuneE) };
("windows-theme-generic") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21windows_theme_genericE) };
("-moz-color-picker-available") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms27_moz_color_picker_availableE) };
("-moz-scrollbar-start-backward") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms29_moz_scrollbar_start_backwardE) };
("-moz-scrollbar-start-forward") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms28_moz_scrollbar_start_forwardE) };
("-moz-scrollbar-end-backward") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms27_moz_scrollbar_end_backwardE) };
("-moz-scrollbar-end-forward") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms26_moz_scrollbar_end_forwardE) };
("-moz-scrollbar-thumb-proportional") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms33_moz_scrollbar_thumb_proportionalE) };
("-moz-images-in-menus") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20_moz_images_in_menusE) };
("-moz-images-in-buttons") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms22_moz_images_in_buttonsE) };
("-moz-overlay-scrollbars") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23_moz_overlay_scrollbarsE) };
("-moz-windows-default-theme") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms26_moz_windows_default_themeE) };
("-moz-mac-graphite-theme") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23_moz_mac_graphite_themeE) };
("-moz-mac-lion-theme") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms19_moz_mac_lion_themeE) };
("-moz-mac-yosemite-theme") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23_moz_mac_yosemite_themeE) };
("-moz-windows-compositor") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23_moz_windows_compositorE) };
("-moz-windows-classic") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20_moz_windows_classicE) };
("-moz-windows-glass") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18_moz_windows_glassE) };
("-moz-windows-theme") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18_moz_windows_themeE) };
("-moz-os-version") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15_moz_os_versionE) };
("-moz-touch-enabled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18_moz_touch_enabledE) };
("-moz-menubar-drag") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17_moz_menubar_dragE) };
("-moz-device-pixel-ratio") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23_moz_device_pixel_ratioE) };
("-moz-device-orientation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms23_moz_device_orientationE) };
("-moz-is-resource-document") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms25_moz_is_resource_documentE) };
("-moz-swipe-animation-enabled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms28_moz_swipe_animation_enabledE) };
("-moz-physical-home-button") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms25_moz_physical_home_buttonE) };
("Back") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4BackE) };
("Forward") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7ForwardE) };
("Reload") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6ReloadE) };
("Stop") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4StopE) };
("Search") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6SearchE) };
("Bookmarks") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9BookmarksE) };
("Home") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4HomeE) };
("Clear") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5ClearE) };
("VolumeUp") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8VolumeUpE) };
("VolumeDown") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10VolumeDownE) };
("NextTrack") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9NextTrackE) };
("PreviousTrack") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13PreviousTrackE) };
("MediaStop") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9MediaStopE) };
("PlayPause") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9PlayPauseE) };
("Menu") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4MenuE) };
("New") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3NewE) };
("Open") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4OpenE) };
("Close") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5CloseE) };
("Save") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4SaveE) };
("Find") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4FindE) };
("Help") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4HelpE) };
("Print") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5PrintE) };
("SendMail") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8SendMailE) };
("ForwardMail") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11ForwardMailE) };
("ReplyToMail") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11ReplyToMailE) };
("mouseWheel") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10mouseWheelE) };
("pixels") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6pixelsE) };
("lines") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5linesE) };
("pages") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5pagesE) };
("scrollbars") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10scrollbarsE) };
("other") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5otherE) };
("apz") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms3apzE) };
("restore") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7restoreE) };
("alert") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5alertE) };
("alertdialog") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11alertdialogE) };
("application") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11applicationE) };
("aria-activedescendant") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms21aria_activedescendantE) };
("aria-atomic") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11aria_atomicE) };
("aria-autocomplete") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17aria_autocompleteE) };
("aria-busy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9aria_busyE) };
("aria-checked") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12aria_checkedE) };
("aria-colcount") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_colcountE) };
("aria-colindex") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_colindexE) };
("aria-controls") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_controlsE) };
("aria-describedby") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16aria_describedbyE) };
("aria-disabled") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_disabledE) };
("aria-dropeffect") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15aria_dropeffectE) };
("aria-expanded") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_expandedE) };
("aria-flowto") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11aria_flowtoE) };
("aria-grabbed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12aria_grabbedE) };
("aria-haspopup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_haspopupE) };
("aria-hidden") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11aria_hiddenE) };
("aria-invalid") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12aria_invalidE) };
("aria-label") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10aria_labelE) };
("aria-labelledby") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15aria_labelledbyE) };
("aria-level") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10aria_levelE) };
("aria-live") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9aria_liveE) };
("aria-modal") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10aria_modalE) };
("aria-multiline") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14aria_multilineE) };
("aria-multiselectable") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20aria_multiselectableE) };
("aria-orientation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16aria_orientationE) };
("aria-owns") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9aria_ownsE) };
("aria-posinset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_posinsetE) };
("aria-pressed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12aria_pressedE) };
("aria-readonly") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_readonlyE) };
("aria-relevant") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_relevantE) };
("aria-required") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_requiredE) };
("aria-rowcount") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_rowcountE) };
("aria-rowindex") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_rowindexE) };
("aria-selected") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_selectedE) };
("aria-setsize") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12aria_setsizeE) };
("aria-sort") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9aria_sortE) };
("aria-valuenow") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_valuenowE) };
("aria-valuemin") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_valueminE) };
("aria-valuemax") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13aria_valuemaxE) };
("aria-valuetext") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14aria_valuetextE) };
("AreaFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9AreaFrameE) };
("auto-generated") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14auto_generatedE) };
("banner") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6bannerE) };
("checkable") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9checkableE) };
("choices") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7choicesE) };
("columnheader") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12columnheaderE) };
("complementary") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13complementaryE) };
("container-atomic") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms15containerAtomicE) };
("container-busy") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13containerBusyE) };
("container-live") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13containerLiveE) };
("container-live-role") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17containerLiveRoleE) };
("container-relevant") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms17containerRelevantE) };
("contentinfo") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11contentinfoE) };
("cycles") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6cyclesE) };
("datatable") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9datatableE) };
("event-from-input") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14eventFromInputE) };
("grammar") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7grammarE) };
("gridcell") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8gridcellE) };
("heading") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7headingE) };
("hitregion") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9hitregionE) };
("InlineBlockFrame") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16InlineBlockFrameE) };
("inline") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11inlinevalueE) };
("invalid") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7invalidE) };
("item") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4itemE) };
("itemset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7itemsetE) };
("line-number") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10lineNumberE) };
("linkedpanel") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11linkedPanelE) };
("live") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms4liveE) };
("menuitemcheckbox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16menuitemcheckboxE) };
("menuitemradio") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13menuitemradioE) };
("mixed") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5mixedE) };
("multiline") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9multilineE) };
("navigation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10navigationE) };
("polite") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6politeE) };
("posinset") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8posinsetE) };
("presentation") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12presentationE) };
("progressbar") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11progressbarE) };
("region") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6regionE) };
("rowgroup") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8rowgroupE) };
("rowheader") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9rowheaderE) };
("search") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6searchE) };
("searchbox") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9searchboxE) };
("select1") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7select1E) };
("setsize") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7setsizeE) };
("spelling") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8spellingE) };
("spinbutton") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10spinbuttonE) };
("status") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6statusE) };
("switch") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7_switchE) };
("table-cell-index") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14tableCellIndexE) };
("tablist") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms7tablistE) };
("text-indent") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10textIndentE) };
("text-input-type") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13textInputTypeE) };
("text-line-through-color") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20textLineThroughColorE) };
("text-line-through-style") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms20textLineThroughStyleE) };
("text-position") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12textPositionE) };
("text-underline-color") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18textUnderlineColorE) };
("text-underline-style") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms18textUnderlineStyleE) };
("timer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms5timerE) };
("toolbarname") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11toolbarnameE) };
("toolbarseparator") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms16toolbarseparatorE) };
("toolbarspacer") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13toolbarspacerE) };
("toolbarspring") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13toolbarspringE) };
("treegrid") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8treegridE) };
("undefined") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10_undefinedE) };
("xml-roles") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8xmlrolesE) };
("close-fence") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11close_fenceE) };
("denominator") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11denominatorE) };
("numerator") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9numeratorE) };
("open-fence") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10open_fenceE) };
("overscript") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10overscriptE) };
("presubscript") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12presubscriptE) };
("presuperscript") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms14presuperscriptE) };
("root-index") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10root_indexE) };
("subscript") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9subscriptE) };
("superscript") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11superscriptE) };
("underscript") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11underscriptE) };
("onaudiostart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onaudiostartE) };
("onaudioend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onaudioendE) };
("onsoundstart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12onsoundstartE) };
("onsoundend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onsoundendE) };
("onspeechstart") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13onspeechstartE) };
("onspeechend") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11onspeechendE) };
("onresult") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8onresultE) };
("onnomatch") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9onnomatchE) };
("onresume") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8onresumeE) };
("onmark") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms6onmarkE) };
("onboundary") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10onboundaryE) };
("vr-state") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms8vr_stateE) };
("usercontextid") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms13usercontextidE) };
("http://www.w3.org/2000/xmlns/") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11nsuri_xmlnsE) };
("http://www.w3.org/XML/1998/namespace") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9nsuri_xmlE) };
("http://www.w3.org/1999/xhtml") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11nsuri_xhtmlE) };
("http://www.w3.org/1999/xlink") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms11nsuri_xlinkE) };
("http://www.w3.org/1999/XSL/Transform") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms10nsuri_xsltE) };
("http://www.mozilla.org/xbl") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9nsuri_xblE) };
("http://www.w3.org/1998/Math/MathML") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms12nsuri_mathmlE) };
("http://www.w3.org/1999/02/22-rdf-syntax-ns#") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9nsuri_rdfE) };
("http://www.mozilla.org/keymaster/gatekeeper/there.is.only.xul") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9nsuri_xulE) };
("http://www.w3.org/2000/svg") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::_ZN9nsGkAtoms9nsuri_svgE) };
}
