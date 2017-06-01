/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Gecko's pseudo-element definition.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PseudoElement {
        /// :after
        After,
        /// :before
        Before,
        /// :backdrop
        Backdrop,
        /// :cue
        Cue,
        /// :first-letter
        FirstLetter,
        /// :first-line
        FirstLine,
        /// :-moz-selection
        MozSelection,
        /// :-moz-focus-inner
        MozFocusInner,
        /// :-moz-focus-outer
        MozFocusOuter,
        /// :-moz-list-bullet
        MozListBullet,
        /// :-moz-list-number
        MozListNumber,
        /// :-moz-math-anonymous
        MozMathAnonymous,
        /// :-moz-number-wrapper
        MozNumberWrapper,
        /// :-moz-number-text
        MozNumberText,
        /// :-moz-number-spin-box
        MozNumberSpinBox,
        /// :-moz-number-spin-up
        MozNumberSpinUp,
        /// :-moz-number-spin-down
        MozNumberSpinDown,
        /// :-moz-progress-bar
        MozProgressBar,
        /// :-moz-range-track
        MozRangeTrack,
        /// :-moz-range-progress
        MozRangeProgress,
        /// :-moz-range-thumb
        MozRangeThumb,
        /// :-moz-meter-bar
        MozMeterBar,
        /// :-moz-placeholder
        MozPlaceholder,
        /// :placeholder
        Placeholder,
        /// :-moz-color-swatch
        MozColorSwatch,
        /// :-moz-text
        MozText,
        /// :-moz-oof-placeholder
        OofPlaceholder,
        /// :-moz-first-letter-continuation
        FirstLetterContinuation,
        /// :-moz-block-inside-inline-wrapper
        MozBlockInsideInlineWrapper,
        /// :-moz-mathml-anonymous-block
        MozMathMLAnonymousBlock,
        /// :-moz-xul-anonymous-block
        MozXULAnonymousBlock,
        /// :-moz-hframeset-border
        HorizontalFramesetBorder,
        /// :-moz-vframeset-border
        VerticalFramesetBorder,
        /// :-moz-line-frame
        MozLineFrame,
        /// :-moz-button-content
        ButtonContent,
        /// :-moz-cell-content
        CellContent,
        /// :-moz-dropdown-list
        DropDownList,
        /// :-moz-fieldset-content
        FieldsetContent,
        /// :-moz-frameset-blank
        FramesetBlank,
        /// :-moz-display-comboboxcontrol-frame
        MozDisplayComboboxControlFrame,
        /// :-moz-html-canvas-content
        HtmlCanvasContent,
        /// :-moz-inline-table
        InlineTable,
        /// :-moz-table
        Table,
        /// :-moz-table-cell
        TableCell,
        /// :-moz-table-column-group
        TableColGroup,
        /// :-moz-table-column
        TableCol,
        /// :-moz-table-wrapper
        TableWrapper,
        /// :-moz-table-row-group
        TableRowGroup,
        /// :-moz-table-row
        TableRow,
        /// :-moz-canvas
        Canvas,
        /// :-moz-pagebreak
        PageBreak,
        /// :-moz-page
        Page,
        /// :-moz-pagecontent
        PageContent,
        /// :-moz-page-sequence
        PageSequence,
        /// :-moz-scrolled-content
        ScrolledContent,
        /// :-moz-scrolled-canvas
        ScrolledCanvas,
        /// :-moz-scrolled-page-sequence
        ScrolledPageSequence,
        /// :-moz-column-content
        ColumnContent,
        /// :-moz-viewport
        Viewport,
        /// :-moz-viewport-scroll
        ViewportScroll,
        /// :-moz-anonymous-flex-item
        AnonymousFlexItem,
        /// :-moz-anonymous-grid-item
        AnonymousGridItem,
        /// :-moz-ruby
        Ruby,
        /// :-moz-ruby-base
        RubyBase,
        /// :-moz-ruby-base-container
        RubyBaseContainer,
        /// :-moz-ruby-text
        RubyText,
        /// :-moz-ruby-text-container
        RubyTextContainer,
        /// :-moz-tree-column
        Moztreecolumn,
        /// :-moz-tree-row
        Moztreerow,
        /// :-moz-tree-separator
        Moztreeseparator,
        /// :-moz-tree-cell
        Moztreecell,
        /// :-moz-tree-indentation
        Moztreeindentation,
        /// :-moz-tree-line
        Moztreeline,
        /// :-moz-tree-twisty
        Moztreetwisty,
        /// :-moz-tree-image
        Moztreeimage,
        /// :-moz-tree-cell-text
        Moztreecelltext,
        /// :-moz-tree-checkbox
        Moztreecheckbox,
        /// :-moz-tree-progressmeter
        Moztreeprogressmeter,
        /// :-moz-tree-drop-feedback
        Moztreedropfeedback,
        /// :-moz-svg-marker-anon-child
        MozSVGMarkerAnonChild,
        /// :-moz-svg-outer-svg-anon-child
        MozSVGOuterSVGAnonChild,
        /// :-moz-svg-foreign-content
        MozSVGForeignContent,
        /// :-moz-svg-text
        MozSVGText,
}



/// The number of eager pseudo-elements.
pub const EAGER_PSEUDO_COUNT: usize = 4;

/// The list of eager pseudos.
pub const EAGER_PSEUDOS: [PseudoElement; EAGER_PSEUDO_COUNT] = [
    PseudoElement::Before,
    PseudoElement::After,
    PseudoElement::FirstLine,
    PseudoElement::FirstLetter,
];

impl PseudoElement {
    /// Executes a closure with each pseudo-element as an argument.
    pub fn each<F>(mut fun: F)
        where F: FnMut(Self),
    {
            fun(PseudoElement::After);
            fun(PseudoElement::Before);
            fun(PseudoElement::Backdrop);
            fun(PseudoElement::Cue);
            fun(PseudoElement::FirstLetter);
            fun(PseudoElement::FirstLine);
            fun(PseudoElement::MozSelection);
            fun(PseudoElement::MozFocusInner);
            fun(PseudoElement::MozFocusOuter);
            fun(PseudoElement::MozListBullet);
            fun(PseudoElement::MozListNumber);
            fun(PseudoElement::MozMathAnonymous);
            fun(PseudoElement::MozNumberWrapper);
            fun(PseudoElement::MozNumberText);
            fun(PseudoElement::MozNumberSpinBox);
            fun(PseudoElement::MozNumberSpinUp);
            fun(PseudoElement::MozNumberSpinDown);
            fun(PseudoElement::MozProgressBar);
            fun(PseudoElement::MozRangeTrack);
            fun(PseudoElement::MozRangeProgress);
            fun(PseudoElement::MozRangeThumb);
            fun(PseudoElement::MozMeterBar);
            fun(PseudoElement::MozPlaceholder);
            fun(PseudoElement::Placeholder);
            fun(PseudoElement::MozColorSwatch);
            fun(PseudoElement::MozText);
            fun(PseudoElement::OofPlaceholder);
            fun(PseudoElement::FirstLetterContinuation);
            fun(PseudoElement::MozBlockInsideInlineWrapper);
            fun(PseudoElement::MozMathMLAnonymousBlock);
            fun(PseudoElement::MozXULAnonymousBlock);
            fun(PseudoElement::HorizontalFramesetBorder);
            fun(PseudoElement::VerticalFramesetBorder);
            fun(PseudoElement::MozLineFrame);
            fun(PseudoElement::ButtonContent);
            fun(PseudoElement::CellContent);
            fun(PseudoElement::DropDownList);
            fun(PseudoElement::FieldsetContent);
            fun(PseudoElement::FramesetBlank);
            fun(PseudoElement::MozDisplayComboboxControlFrame);
            fun(PseudoElement::HtmlCanvasContent);
            fun(PseudoElement::InlineTable);
            fun(PseudoElement::Table);
            fun(PseudoElement::TableCell);
            fun(PseudoElement::TableColGroup);
            fun(PseudoElement::TableCol);
            fun(PseudoElement::TableWrapper);
            fun(PseudoElement::TableRowGroup);
            fun(PseudoElement::TableRow);
            fun(PseudoElement::Canvas);
            fun(PseudoElement::PageBreak);
            fun(PseudoElement::Page);
            fun(PseudoElement::PageContent);
            fun(PseudoElement::PageSequence);
            fun(PseudoElement::ScrolledContent);
            fun(PseudoElement::ScrolledCanvas);
            fun(PseudoElement::ScrolledPageSequence);
            fun(PseudoElement::ColumnContent);
            fun(PseudoElement::Viewport);
            fun(PseudoElement::ViewportScroll);
            fun(PseudoElement::AnonymousFlexItem);
            fun(PseudoElement::AnonymousGridItem);
            fun(PseudoElement::Ruby);
            fun(PseudoElement::RubyBase);
            fun(PseudoElement::RubyBaseContainer);
            fun(PseudoElement::RubyText);
            fun(PseudoElement::RubyTextContainer);
            fun(PseudoElement::Moztreecolumn);
            fun(PseudoElement::Moztreerow);
            fun(PseudoElement::Moztreeseparator);
            fun(PseudoElement::Moztreecell);
            fun(PseudoElement::Moztreeindentation);
            fun(PseudoElement::Moztreeline);
            fun(PseudoElement::Moztreetwisty);
            fun(PseudoElement::Moztreeimage);
            fun(PseudoElement::Moztreecelltext);
            fun(PseudoElement::Moztreecheckbox);
            fun(PseudoElement::Moztreeprogressmeter);
            fun(PseudoElement::Moztreedropfeedback);
            fun(PseudoElement::MozSVGMarkerAnonChild);
            fun(PseudoElement::MozSVGOuterSVGAnonChild);
            fun(PseudoElement::MozSVGForeignContent);
            fun(PseudoElement::MozSVGText);
    }

    /// Get the pseudo-element as an atom.
    #[inline]
    pub fn atom(&self) -> Atom {
        match *self {
                PseudoElement::After => atom!(":after"),
                PseudoElement::Before => atom!(":before"),
                PseudoElement::Backdrop => atom!(":backdrop"),
                PseudoElement::Cue => atom!(":cue"),
                PseudoElement::FirstLetter => atom!(":first-letter"),
                PseudoElement::FirstLine => atom!(":first-line"),
                PseudoElement::MozSelection => atom!(":-moz-selection"),
                PseudoElement::MozFocusInner => atom!(":-moz-focus-inner"),
                PseudoElement::MozFocusOuter => atom!(":-moz-focus-outer"),
                PseudoElement::MozListBullet => atom!(":-moz-list-bullet"),
                PseudoElement::MozListNumber => atom!(":-moz-list-number"),
                PseudoElement::MozMathAnonymous => atom!(":-moz-math-anonymous"),
                PseudoElement::MozNumberWrapper => atom!(":-moz-number-wrapper"),
                PseudoElement::MozNumberText => atom!(":-moz-number-text"),
                PseudoElement::MozNumberSpinBox => atom!(":-moz-number-spin-box"),
                PseudoElement::MozNumberSpinUp => atom!(":-moz-number-spin-up"),
                PseudoElement::MozNumberSpinDown => atom!(":-moz-number-spin-down"),
                PseudoElement::MozProgressBar => atom!(":-moz-progress-bar"),
                PseudoElement::MozRangeTrack => atom!(":-moz-range-track"),
                PseudoElement::MozRangeProgress => atom!(":-moz-range-progress"),
                PseudoElement::MozRangeThumb => atom!(":-moz-range-thumb"),
                PseudoElement::MozMeterBar => atom!(":-moz-meter-bar"),
                PseudoElement::MozPlaceholder => atom!(":-moz-placeholder"),
                PseudoElement::Placeholder => atom!(":placeholder"),
                PseudoElement::MozColorSwatch => atom!(":-moz-color-swatch"),
                PseudoElement::MozText => atom!(":-moz-text"),
                PseudoElement::OofPlaceholder => atom!(":-moz-oof-placeholder"),
                PseudoElement::FirstLetterContinuation => atom!(":-moz-first-letter-continuation"),
                PseudoElement::MozBlockInsideInlineWrapper => atom!(":-moz-block-inside-inline-wrapper"),
                PseudoElement::MozMathMLAnonymousBlock => atom!(":-moz-mathml-anonymous-block"),
                PseudoElement::MozXULAnonymousBlock => atom!(":-moz-xul-anonymous-block"),
                PseudoElement::HorizontalFramesetBorder => atom!(":-moz-hframeset-border"),
                PseudoElement::VerticalFramesetBorder => atom!(":-moz-vframeset-border"),
                PseudoElement::MozLineFrame => atom!(":-moz-line-frame"),
                PseudoElement::ButtonContent => atom!(":-moz-button-content"),
                PseudoElement::CellContent => atom!(":-moz-cell-content"),
                PseudoElement::DropDownList => atom!(":-moz-dropdown-list"),
                PseudoElement::FieldsetContent => atom!(":-moz-fieldset-content"),
                PseudoElement::FramesetBlank => atom!(":-moz-frameset-blank"),
                PseudoElement::MozDisplayComboboxControlFrame => atom!(":-moz-display-comboboxcontrol-frame"),
                PseudoElement::HtmlCanvasContent => atom!(":-moz-html-canvas-content"),
                PseudoElement::InlineTable => atom!(":-moz-inline-table"),
                PseudoElement::Table => atom!(":-moz-table"),
                PseudoElement::TableCell => atom!(":-moz-table-cell"),
                PseudoElement::TableColGroup => atom!(":-moz-table-column-group"),
                PseudoElement::TableCol => atom!(":-moz-table-column"),
                PseudoElement::TableWrapper => atom!(":-moz-table-wrapper"),
                PseudoElement::TableRowGroup => atom!(":-moz-table-row-group"),
                PseudoElement::TableRow => atom!(":-moz-table-row"),
                PseudoElement::Canvas => atom!(":-moz-canvas"),
                PseudoElement::PageBreak => atom!(":-moz-pagebreak"),
                PseudoElement::Page => atom!(":-moz-page"),
                PseudoElement::PageContent => atom!(":-moz-pagecontent"),
                PseudoElement::PageSequence => atom!(":-moz-page-sequence"),
                PseudoElement::ScrolledContent => atom!(":-moz-scrolled-content"),
                PseudoElement::ScrolledCanvas => atom!(":-moz-scrolled-canvas"),
                PseudoElement::ScrolledPageSequence => atom!(":-moz-scrolled-page-sequence"),
                PseudoElement::ColumnContent => atom!(":-moz-column-content"),
                PseudoElement::Viewport => atom!(":-moz-viewport"),
                PseudoElement::ViewportScroll => atom!(":-moz-viewport-scroll"),
                PseudoElement::AnonymousFlexItem => atom!(":-moz-anonymous-flex-item"),
                PseudoElement::AnonymousGridItem => atom!(":-moz-anonymous-grid-item"),
                PseudoElement::Ruby => atom!(":-moz-ruby"),
                PseudoElement::RubyBase => atom!(":-moz-ruby-base"),
                PseudoElement::RubyBaseContainer => atom!(":-moz-ruby-base-container"),
                PseudoElement::RubyText => atom!(":-moz-ruby-text"),
                PseudoElement::RubyTextContainer => atom!(":-moz-ruby-text-container"),
                PseudoElement::Moztreecolumn => atom!(":-moz-tree-column"),
                PseudoElement::Moztreerow => atom!(":-moz-tree-row"),
                PseudoElement::Moztreeseparator => atom!(":-moz-tree-separator"),
                PseudoElement::Moztreecell => atom!(":-moz-tree-cell"),
                PseudoElement::Moztreeindentation => atom!(":-moz-tree-indentation"),
                PseudoElement::Moztreeline => atom!(":-moz-tree-line"),
                PseudoElement::Moztreetwisty => atom!(":-moz-tree-twisty"),
                PseudoElement::Moztreeimage => atom!(":-moz-tree-image"),
                PseudoElement::Moztreecelltext => atom!(":-moz-tree-cell-text"),
                PseudoElement::Moztreecheckbox => atom!(":-moz-tree-checkbox"),
                PseudoElement::Moztreeprogressmeter => atom!(":-moz-tree-progressmeter"),
                PseudoElement::Moztreedropfeedback => atom!(":-moz-tree-drop-feedback"),
                PseudoElement::MozSVGMarkerAnonChild => atom!(":-moz-svg-marker-anon-child"),
                PseudoElement::MozSVGOuterSVGAnonChild => atom!(":-moz-svg-outer-svg-anon-child"),
                PseudoElement::MozSVGForeignContent => atom!(":-moz-svg-foreign-content"),
                PseudoElement::MozSVGText => atom!(":-moz-svg-text"),
        }
    }

    /// Whether this pseudo-element is an anonymous box.
    #[inline]
    fn is_anon_box(&self) -> bool {
        match *self {
                PseudoElement::After => false,
                PseudoElement::Before => false,
                PseudoElement::Backdrop => false,
                PseudoElement::Cue => false,
                PseudoElement::FirstLetter => false,
                PseudoElement::FirstLine => false,
                PseudoElement::MozSelection => false,
                PseudoElement::MozFocusInner => false,
                PseudoElement::MozFocusOuter => false,
                PseudoElement::MozListBullet => false,
                PseudoElement::MozListNumber => false,
                PseudoElement::MozMathAnonymous => false,
                PseudoElement::MozNumberWrapper => false,
                PseudoElement::MozNumberText => false,
                PseudoElement::MozNumberSpinBox => false,
                PseudoElement::MozNumberSpinUp => false,
                PseudoElement::MozNumberSpinDown => false,
                PseudoElement::MozProgressBar => false,
                PseudoElement::MozRangeTrack => false,
                PseudoElement::MozRangeProgress => false,
                PseudoElement::MozRangeThumb => false,
                PseudoElement::MozMeterBar => false,
                PseudoElement::MozPlaceholder => false,
                PseudoElement::Placeholder => false,
                PseudoElement::MozColorSwatch => false,
                PseudoElement::MozText => true,
                PseudoElement::OofPlaceholder => true,
                PseudoElement::FirstLetterContinuation => true,
                PseudoElement::MozBlockInsideInlineWrapper => true,
                PseudoElement::MozMathMLAnonymousBlock => true,
                PseudoElement::MozXULAnonymousBlock => true,
                PseudoElement::HorizontalFramesetBorder => true,
                PseudoElement::VerticalFramesetBorder => true,
                PseudoElement::MozLineFrame => true,
                PseudoElement::ButtonContent => true,
                PseudoElement::CellContent => true,
                PseudoElement::DropDownList => true,
                PseudoElement::FieldsetContent => true,
                PseudoElement::FramesetBlank => true,
                PseudoElement::MozDisplayComboboxControlFrame => true,
                PseudoElement::HtmlCanvasContent => true,
                PseudoElement::InlineTable => true,
                PseudoElement::Table => true,
                PseudoElement::TableCell => true,
                PseudoElement::TableColGroup => true,
                PseudoElement::TableCol => true,
                PseudoElement::TableWrapper => true,
                PseudoElement::TableRowGroup => true,
                PseudoElement::TableRow => true,
                PseudoElement::Canvas => true,
                PseudoElement::PageBreak => true,
                PseudoElement::Page => true,
                PseudoElement::PageContent => true,
                PseudoElement::PageSequence => true,
                PseudoElement::ScrolledContent => true,
                PseudoElement::ScrolledCanvas => true,
                PseudoElement::ScrolledPageSequence => true,
                PseudoElement::ColumnContent => true,
                PseudoElement::Viewport => true,
                PseudoElement::ViewportScroll => true,
                PseudoElement::AnonymousFlexItem => true,
                PseudoElement::AnonymousGridItem => true,
                PseudoElement::Ruby => true,
                PseudoElement::RubyBase => true,
                PseudoElement::RubyBaseContainer => true,
                PseudoElement::RubyText => true,
                PseudoElement::RubyTextContainer => true,
                PseudoElement::Moztreecolumn => true,
                PseudoElement::Moztreerow => true,
                PseudoElement::Moztreeseparator => true,
                PseudoElement::Moztreecell => true,
                PseudoElement::Moztreeindentation => true,
                PseudoElement::Moztreeline => true,
                PseudoElement::Moztreetwisty => true,
                PseudoElement::Moztreeimage => true,
                PseudoElement::Moztreecelltext => true,
                PseudoElement::Moztreecheckbox => true,
                PseudoElement::Moztreeprogressmeter => true,
                PseudoElement::Moztreedropfeedback => true,
                PseudoElement::MozSVGMarkerAnonChild => true,
                PseudoElement::MozSVGOuterSVGAnonChild => true,
                PseudoElement::MozSVGForeignContent => true,
                PseudoElement::MozSVGText => true,
        }
    }

    /// Whether this pseudo-element is eagerly-cascaded.
    #[inline]
    pub fn is_eager(&self) -> bool {
        matches!(*self,
                 PseudoElement::Before | PseudoElement::After | PseudoElement::FirstLine | PseudoElement::FirstLetter)
    }

    /// Gets the flags associated to this pseudo-element, or 0 if it's an
    /// anonymous box.
    pub fn flags(&self) -> u32 {
        match *self {
                PseudoElement::After => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_after
                }
                PseudoElement::Before => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_before
                }
                PseudoElement::Backdrop => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_backdrop
                }
                PseudoElement::Cue => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_cue
                }
                PseudoElement::FirstLetter => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_firstLetter
                }
                PseudoElement::FirstLine => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_firstLine
                }
                PseudoElement::MozSelection => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozSelection
                }
                PseudoElement::MozFocusInner => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozFocusInner
                }
                PseudoElement::MozFocusOuter => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozFocusOuter
                }
                PseudoElement::MozListBullet => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozListBullet
                }
                PseudoElement::MozListNumber => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozListNumber
                }
                PseudoElement::MozMathAnonymous => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozMathAnonymous
                }
                PseudoElement::MozNumberWrapper => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozNumberWrapper
                }
                PseudoElement::MozNumberText => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozNumberText
                }
                PseudoElement::MozNumberSpinBox => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozNumberSpinBox
                }
                PseudoElement::MozNumberSpinUp => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozNumberSpinUp
                }
                PseudoElement::MozNumberSpinDown => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozNumberSpinDown
                }
                PseudoElement::MozProgressBar => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozProgressBar
                }
                PseudoElement::MozRangeTrack => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozRangeTrack
                }
                PseudoElement::MozRangeProgress => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozRangeProgress
                }
                PseudoElement::MozRangeThumb => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozRangeThumb
                }
                PseudoElement::MozMeterBar => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozMeterBar
                }
                PseudoElement::MozPlaceholder => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozPlaceholder
                }
                PseudoElement::Placeholder => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_placeholder
                }
                PseudoElement::MozColorSwatch => {
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozColorSwatch
                }
                PseudoElement::MozText => {
                        0
                }
                PseudoElement::OofPlaceholder => {
                        0
                }
                PseudoElement::FirstLetterContinuation => {
                        0
                }
                PseudoElement::MozBlockInsideInlineWrapper => {
                        0
                }
                PseudoElement::MozMathMLAnonymousBlock => {
                        0
                }
                PseudoElement::MozXULAnonymousBlock => {
                        0
                }
                PseudoElement::HorizontalFramesetBorder => {
                        0
                }
                PseudoElement::VerticalFramesetBorder => {
                        0
                }
                PseudoElement::MozLineFrame => {
                        0
                }
                PseudoElement::ButtonContent => {
                        0
                }
                PseudoElement::CellContent => {
                        0
                }
                PseudoElement::DropDownList => {
                        0
                }
                PseudoElement::FieldsetContent => {
                        0
                }
                PseudoElement::FramesetBlank => {
                        0
                }
                PseudoElement::MozDisplayComboboxControlFrame => {
                        0
                }
                PseudoElement::HtmlCanvasContent => {
                        0
                }
                PseudoElement::InlineTable => {
                        0
                }
                PseudoElement::Table => {
                        0
                }
                PseudoElement::TableCell => {
                        0
                }
                PseudoElement::TableColGroup => {
                        0
                }
                PseudoElement::TableCol => {
                        0
                }
                PseudoElement::TableWrapper => {
                        0
                }
                PseudoElement::TableRowGroup => {
                        0
                }
                PseudoElement::TableRow => {
                        0
                }
                PseudoElement::Canvas => {
                        0
                }
                PseudoElement::PageBreak => {
                        0
                }
                PseudoElement::Page => {
                        0
                }
                PseudoElement::PageContent => {
                        0
                }
                PseudoElement::PageSequence => {
                        0
                }
                PseudoElement::ScrolledContent => {
                        0
                }
                PseudoElement::ScrolledCanvas => {
                        0
                }
                PseudoElement::ScrolledPageSequence => {
                        0
                }
                PseudoElement::ColumnContent => {
                        0
                }
                PseudoElement::Viewport => {
                        0
                }
                PseudoElement::ViewportScroll => {
                        0
                }
                PseudoElement::AnonymousFlexItem => {
                        0
                }
                PseudoElement::AnonymousGridItem => {
                        0
                }
                PseudoElement::Ruby => {
                        0
                }
                PseudoElement::RubyBase => {
                        0
                }
                PseudoElement::RubyBaseContainer => {
                        0
                }
                PseudoElement::RubyText => {
                        0
                }
                PseudoElement::RubyTextContainer => {
                        0
                }
                PseudoElement::Moztreecolumn => {
                        0
                }
                PseudoElement::Moztreerow => {
                        0
                }
                PseudoElement::Moztreeseparator => {
                        0
                }
                PseudoElement::Moztreecell => {
                        0
                }
                PseudoElement::Moztreeindentation => {
                        0
                }
                PseudoElement::Moztreeline => {
                        0
                }
                PseudoElement::Moztreetwisty => {
                        0
                }
                PseudoElement::Moztreeimage => {
                        0
                }
                PseudoElement::Moztreecelltext => {
                        0
                }
                PseudoElement::Moztreecheckbox => {
                        0
                }
                PseudoElement::Moztreeprogressmeter => {
                        0
                }
                PseudoElement::Moztreedropfeedback => {
                        0
                }
                PseudoElement::MozSVGMarkerAnonChild => {
                        0
                }
                PseudoElement::MozSVGOuterSVGAnonChild => {
                        0
                }
                PseudoElement::MozSVGForeignContent => {
                        0
                }
                PseudoElement::MozSVGText => {
                        0
                }
        }
    }

    /// Construct a pseudo-element from a `CSSPseudoElementType`.
    #[inline]
    pub fn from_pseudo_type(type_: CSSPseudoElementType) -> Option<Self> {
        match type_ {
                    CSSPseudoElementType::after => {
                        Some(PseudoElement::After)
                    },
                    CSSPseudoElementType::before => {
                        Some(PseudoElement::Before)
                    },
                    CSSPseudoElementType::backdrop => {
                        Some(PseudoElement::Backdrop)
                    },
                    CSSPseudoElementType::cue => {
                        Some(PseudoElement::Cue)
                    },
                    CSSPseudoElementType::firstLetter => {
                        Some(PseudoElement::FirstLetter)
                    },
                    CSSPseudoElementType::firstLine => {
                        Some(PseudoElement::FirstLine)
                    },
                    CSSPseudoElementType::mozSelection => {
                        Some(PseudoElement::MozSelection)
                    },
                    CSSPseudoElementType::mozFocusInner => {
                        Some(PseudoElement::MozFocusInner)
                    },
                    CSSPseudoElementType::mozFocusOuter => {
                        Some(PseudoElement::MozFocusOuter)
                    },
                    CSSPseudoElementType::mozListBullet => {
                        Some(PseudoElement::MozListBullet)
                    },
                    CSSPseudoElementType::mozListNumber => {
                        Some(PseudoElement::MozListNumber)
                    },
                    CSSPseudoElementType::mozMathAnonymous => {
                        Some(PseudoElement::MozMathAnonymous)
                    },
                    CSSPseudoElementType::mozNumberWrapper => {
                        Some(PseudoElement::MozNumberWrapper)
                    },
                    CSSPseudoElementType::mozNumberText => {
                        Some(PseudoElement::MozNumberText)
                    },
                    CSSPseudoElementType::mozNumberSpinBox => {
                        Some(PseudoElement::MozNumberSpinBox)
                    },
                    CSSPseudoElementType::mozNumberSpinUp => {
                        Some(PseudoElement::MozNumberSpinUp)
                    },
                    CSSPseudoElementType::mozNumberSpinDown => {
                        Some(PseudoElement::MozNumberSpinDown)
                    },
                    CSSPseudoElementType::mozProgressBar => {
                        Some(PseudoElement::MozProgressBar)
                    },
                    CSSPseudoElementType::mozRangeTrack => {
                        Some(PseudoElement::MozRangeTrack)
                    },
                    CSSPseudoElementType::mozRangeProgress => {
                        Some(PseudoElement::MozRangeProgress)
                    },
                    CSSPseudoElementType::mozRangeThumb => {
                        Some(PseudoElement::MozRangeThumb)
                    },
                    CSSPseudoElementType::mozMeterBar => {
                        Some(PseudoElement::MozMeterBar)
                    },
                    CSSPseudoElementType::mozPlaceholder => {
                        Some(PseudoElement::MozPlaceholder)
                    },
                    CSSPseudoElementType::placeholder => {
                        Some(PseudoElement::Placeholder)
                    },
                    CSSPseudoElementType::mozColorSwatch => {
                        Some(PseudoElement::MozColorSwatch)
                    },
            _ => None,
        }
    }

    /// Construct a pseudo-element from an anonymous box `Atom`.
    #[inline]
    pub fn from_anon_box_atom(atom: &Atom) -> Option<Self> {
                if atom == &atom!(":-moz-text") {
                    return Some(PseudoElement::MozText);
                }
                if atom == &atom!(":-moz-oof-placeholder") {
                    return Some(PseudoElement::OofPlaceholder);
                }
                if atom == &atom!(":-moz-first-letter-continuation") {
                    return Some(PseudoElement::FirstLetterContinuation);
                }
                if atom == &atom!(":-moz-block-inside-inline-wrapper") {
                    return Some(PseudoElement::MozBlockInsideInlineWrapper);
                }
                if atom == &atom!(":-moz-mathml-anonymous-block") {
                    return Some(PseudoElement::MozMathMLAnonymousBlock);
                }
                if atom == &atom!(":-moz-xul-anonymous-block") {
                    return Some(PseudoElement::MozXULAnonymousBlock);
                }
                if atom == &atom!(":-moz-hframeset-border") {
                    return Some(PseudoElement::HorizontalFramesetBorder);
                }
                if atom == &atom!(":-moz-vframeset-border") {
                    return Some(PseudoElement::VerticalFramesetBorder);
                }
                if atom == &atom!(":-moz-line-frame") {
                    return Some(PseudoElement::MozLineFrame);
                }
                if atom == &atom!(":-moz-button-content") {
                    return Some(PseudoElement::ButtonContent);
                }
                if atom == &atom!(":-moz-cell-content") {
                    return Some(PseudoElement::CellContent);
                }
                if atom == &atom!(":-moz-dropdown-list") {
                    return Some(PseudoElement::DropDownList);
                }
                if atom == &atom!(":-moz-fieldset-content") {
                    return Some(PseudoElement::FieldsetContent);
                }
                if atom == &atom!(":-moz-frameset-blank") {
                    return Some(PseudoElement::FramesetBlank);
                }
                if atom == &atom!(":-moz-display-comboboxcontrol-frame") {
                    return Some(PseudoElement::MozDisplayComboboxControlFrame);
                }
                if atom == &atom!(":-moz-html-canvas-content") {
                    return Some(PseudoElement::HtmlCanvasContent);
                }
                if atom == &atom!(":-moz-inline-table") {
                    return Some(PseudoElement::InlineTable);
                }
                if atom == &atom!(":-moz-table") {
                    return Some(PseudoElement::Table);
                }
                if atom == &atom!(":-moz-table-cell") {
                    return Some(PseudoElement::TableCell);
                }
                if atom == &atom!(":-moz-table-column-group") {
                    return Some(PseudoElement::TableColGroup);
                }
                if atom == &atom!(":-moz-table-column") {
                    return Some(PseudoElement::TableCol);
                }
                if atom == &atom!(":-moz-table-wrapper") {
                    return Some(PseudoElement::TableWrapper);
                }
                if atom == &atom!(":-moz-table-row-group") {
                    return Some(PseudoElement::TableRowGroup);
                }
                if atom == &atom!(":-moz-table-row") {
                    return Some(PseudoElement::TableRow);
                }
                if atom == &atom!(":-moz-canvas") {
                    return Some(PseudoElement::Canvas);
                }
                if atom == &atom!(":-moz-pagebreak") {
                    return Some(PseudoElement::PageBreak);
                }
                if atom == &atom!(":-moz-page") {
                    return Some(PseudoElement::Page);
                }
                if atom == &atom!(":-moz-pagecontent") {
                    return Some(PseudoElement::PageContent);
                }
                if atom == &atom!(":-moz-page-sequence") {
                    return Some(PseudoElement::PageSequence);
                }
                if atom == &atom!(":-moz-scrolled-content") {
                    return Some(PseudoElement::ScrolledContent);
                }
                if atom == &atom!(":-moz-scrolled-canvas") {
                    return Some(PseudoElement::ScrolledCanvas);
                }
                if atom == &atom!(":-moz-scrolled-page-sequence") {
                    return Some(PseudoElement::ScrolledPageSequence);
                }
                if atom == &atom!(":-moz-column-content") {
                    return Some(PseudoElement::ColumnContent);
                }
                if atom == &atom!(":-moz-viewport") {
                    return Some(PseudoElement::Viewport);
                }
                if atom == &atom!(":-moz-viewport-scroll") {
                    return Some(PseudoElement::ViewportScroll);
                }
                if atom == &atom!(":-moz-anonymous-flex-item") {
                    return Some(PseudoElement::AnonymousFlexItem);
                }
                if atom == &atom!(":-moz-anonymous-grid-item") {
                    return Some(PseudoElement::AnonymousGridItem);
                }
                if atom == &atom!(":-moz-ruby") {
                    return Some(PseudoElement::Ruby);
                }
                if atom == &atom!(":-moz-ruby-base") {
                    return Some(PseudoElement::RubyBase);
                }
                if atom == &atom!(":-moz-ruby-base-container") {
                    return Some(PseudoElement::RubyBaseContainer);
                }
                if atom == &atom!(":-moz-ruby-text") {
                    return Some(PseudoElement::RubyText);
                }
                if atom == &atom!(":-moz-ruby-text-container") {
                    return Some(PseudoElement::RubyTextContainer);
                }
                if atom == &atom!(":-moz-tree-column") {
                    return Some(PseudoElement::Moztreecolumn);
                }
                if atom == &atom!(":-moz-tree-row") {
                    return Some(PseudoElement::Moztreerow);
                }
                if atom == &atom!(":-moz-tree-separator") {
                    return Some(PseudoElement::Moztreeseparator);
                }
                if atom == &atom!(":-moz-tree-cell") {
                    return Some(PseudoElement::Moztreecell);
                }
                if atom == &atom!(":-moz-tree-indentation") {
                    return Some(PseudoElement::Moztreeindentation);
                }
                if atom == &atom!(":-moz-tree-line") {
                    return Some(PseudoElement::Moztreeline);
                }
                if atom == &atom!(":-moz-tree-twisty") {
                    return Some(PseudoElement::Moztreetwisty);
                }
                if atom == &atom!(":-moz-tree-image") {
                    return Some(PseudoElement::Moztreeimage);
                }
                if atom == &atom!(":-moz-tree-cell-text") {
                    return Some(PseudoElement::Moztreecelltext);
                }
                if atom == &atom!(":-moz-tree-checkbox") {
                    return Some(PseudoElement::Moztreecheckbox);
                }
                if atom == &atom!(":-moz-tree-progressmeter") {
                    return Some(PseudoElement::Moztreeprogressmeter);
                }
                if atom == &atom!(":-moz-tree-drop-feedback") {
                    return Some(PseudoElement::Moztreedropfeedback);
                }
                if atom == &atom!(":-moz-svg-marker-anon-child") {
                    return Some(PseudoElement::MozSVGMarkerAnonChild);
                }
                if atom == &atom!(":-moz-svg-outer-svg-anon-child") {
                    return Some(PseudoElement::MozSVGOuterSVGAnonChild);
                }
                if atom == &atom!(":-moz-svg-foreign-content") {
                    return Some(PseudoElement::MozSVGForeignContent);
                }
                if atom == &atom!(":-moz-svg-text") {
                    return Some(PseudoElement::MozSVGText);
                }
        None
    }

    /// Constructs an atom from a string of text, and whether we're in a
    /// user-agent stylesheet.
    ///
    /// If we're not in a user-agent stylesheet, we will never parse anonymous
    /// box pseudo-elements.
    ///
    /// Returns `None` if the pseudo-element is not recognised.
    #[inline]
    pub fn from_slice(s: &str, in_ua_stylesheet: bool) -> Option<Self> {
        use std::ascii::AsciiExt;

            if in_ua_stylesheet || PseudoElement::After.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("after") {
                    return Some(PseudoElement::After)
                }
            }
            if in_ua_stylesheet || PseudoElement::Before.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("before") {
                    return Some(PseudoElement::Before)
                }
            }
            if in_ua_stylesheet || PseudoElement::Backdrop.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("backdrop") {
                    return Some(PseudoElement::Backdrop)
                }
            }
            if in_ua_stylesheet || PseudoElement::Cue.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("cue") {
                    return Some(PseudoElement::Cue)
                }
            }
            if in_ua_stylesheet || PseudoElement::FirstLetter.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("first-letter") {
                    return Some(PseudoElement::FirstLetter)
                }
            }
            if in_ua_stylesheet || PseudoElement::FirstLine.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("first-line") {
                    return Some(PseudoElement::FirstLine)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozSelection.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-selection") {
                    return Some(PseudoElement::MozSelection)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozFocusInner.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-focus-inner") {
                    return Some(PseudoElement::MozFocusInner)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozFocusOuter.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-focus-outer") {
                    return Some(PseudoElement::MozFocusOuter)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozListBullet.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-list-bullet") {
                    return Some(PseudoElement::MozListBullet)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozListNumber.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-list-number") {
                    return Some(PseudoElement::MozListNumber)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozMathAnonymous.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-math-anonymous") {
                    return Some(PseudoElement::MozMathAnonymous)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozNumberWrapper.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-number-wrapper") {
                    return Some(PseudoElement::MozNumberWrapper)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozNumberText.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-number-text") {
                    return Some(PseudoElement::MozNumberText)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozNumberSpinBox.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-number-spin-box") {
                    return Some(PseudoElement::MozNumberSpinBox)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozNumberSpinUp.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-number-spin-up") {
                    return Some(PseudoElement::MozNumberSpinUp)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozNumberSpinDown.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-number-spin-down") {
                    return Some(PseudoElement::MozNumberSpinDown)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozProgressBar.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-progress-bar") {
                    return Some(PseudoElement::MozProgressBar)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozRangeTrack.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-range-track") {
                    return Some(PseudoElement::MozRangeTrack)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozRangeProgress.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-range-progress") {
                    return Some(PseudoElement::MozRangeProgress)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozRangeThumb.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-range-thumb") {
                    return Some(PseudoElement::MozRangeThumb)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozMeterBar.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-meter-bar") {
                    return Some(PseudoElement::MozMeterBar)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozPlaceholder.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-placeholder") {
                    return Some(PseudoElement::MozPlaceholder)
                }
            }
            if in_ua_stylesheet || PseudoElement::Placeholder.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("placeholder") {
                    return Some(PseudoElement::Placeholder)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozColorSwatch.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-color-swatch") {
                    return Some(PseudoElement::MozColorSwatch)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozText.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-text") {
                    return Some(PseudoElement::MozText)
                }
            }
            if in_ua_stylesheet || PseudoElement::OofPlaceholder.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-oof-placeholder") {
                    return Some(PseudoElement::OofPlaceholder)
                }
            }
            if in_ua_stylesheet || PseudoElement::FirstLetterContinuation.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-first-letter-continuation") {
                    return Some(PseudoElement::FirstLetterContinuation)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozBlockInsideInlineWrapper.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-block-inside-inline-wrapper") {
                    return Some(PseudoElement::MozBlockInsideInlineWrapper)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozMathMLAnonymousBlock.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-mathml-anonymous-block") {
                    return Some(PseudoElement::MozMathMLAnonymousBlock)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozXULAnonymousBlock.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-xul-anonymous-block") {
                    return Some(PseudoElement::MozXULAnonymousBlock)
                }
            }
            if in_ua_stylesheet || PseudoElement::HorizontalFramesetBorder.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-hframeset-border") {
                    return Some(PseudoElement::HorizontalFramesetBorder)
                }
            }
            if in_ua_stylesheet || PseudoElement::VerticalFramesetBorder.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-vframeset-border") {
                    return Some(PseudoElement::VerticalFramesetBorder)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozLineFrame.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-line-frame") {
                    return Some(PseudoElement::MozLineFrame)
                }
            }
            if in_ua_stylesheet || PseudoElement::ButtonContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-button-content") {
                    return Some(PseudoElement::ButtonContent)
                }
            }
            if in_ua_stylesheet || PseudoElement::CellContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-cell-content") {
                    return Some(PseudoElement::CellContent)
                }
            }
            if in_ua_stylesheet || PseudoElement::DropDownList.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-dropdown-list") {
                    return Some(PseudoElement::DropDownList)
                }
            }
            if in_ua_stylesheet || PseudoElement::FieldsetContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-fieldset-content") {
                    return Some(PseudoElement::FieldsetContent)
                }
            }
            if in_ua_stylesheet || PseudoElement::FramesetBlank.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-frameset-blank") {
                    return Some(PseudoElement::FramesetBlank)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozDisplayComboboxControlFrame.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-display-comboboxcontrol-frame") {
                    return Some(PseudoElement::MozDisplayComboboxControlFrame)
                }
            }
            if in_ua_stylesheet || PseudoElement::HtmlCanvasContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-html-canvas-content") {
                    return Some(PseudoElement::HtmlCanvasContent)
                }
            }
            if in_ua_stylesheet || PseudoElement::InlineTable.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-inline-table") {
                    return Some(PseudoElement::InlineTable)
                }
            }
            if in_ua_stylesheet || PseudoElement::Table.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-table") {
                    return Some(PseudoElement::Table)
                }
            }
            if in_ua_stylesheet || PseudoElement::TableCell.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-table-cell") {
                    return Some(PseudoElement::TableCell)
                }
            }
            if in_ua_stylesheet || PseudoElement::TableColGroup.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-table-column-group") {
                    return Some(PseudoElement::TableColGroup)
                }
            }
            if in_ua_stylesheet || PseudoElement::TableCol.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-table-column") {
                    return Some(PseudoElement::TableCol)
                }
            }
            if in_ua_stylesheet || PseudoElement::TableWrapper.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-table-wrapper") {
                    return Some(PseudoElement::TableWrapper)
                }
            }
            if in_ua_stylesheet || PseudoElement::TableRowGroup.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-table-row-group") {
                    return Some(PseudoElement::TableRowGroup)
                }
            }
            if in_ua_stylesheet || PseudoElement::TableRow.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-table-row") {
                    return Some(PseudoElement::TableRow)
                }
            }
            if in_ua_stylesheet || PseudoElement::Canvas.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-canvas") {
                    return Some(PseudoElement::Canvas)
                }
            }
            if in_ua_stylesheet || PseudoElement::PageBreak.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-pagebreak") {
                    return Some(PseudoElement::PageBreak)
                }
            }
            if in_ua_stylesheet || PseudoElement::Page.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-page") {
                    return Some(PseudoElement::Page)
                }
            }
            if in_ua_stylesheet || PseudoElement::PageContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-pagecontent") {
                    return Some(PseudoElement::PageContent)
                }
            }
            if in_ua_stylesheet || PseudoElement::PageSequence.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-page-sequence") {
                    return Some(PseudoElement::PageSequence)
                }
            }
            if in_ua_stylesheet || PseudoElement::ScrolledContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-scrolled-content") {
                    return Some(PseudoElement::ScrolledContent)
                }
            }
            if in_ua_stylesheet || PseudoElement::ScrolledCanvas.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-scrolled-canvas") {
                    return Some(PseudoElement::ScrolledCanvas)
                }
            }
            if in_ua_stylesheet || PseudoElement::ScrolledPageSequence.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-scrolled-page-sequence") {
                    return Some(PseudoElement::ScrolledPageSequence)
                }
            }
            if in_ua_stylesheet || PseudoElement::ColumnContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-column-content") {
                    return Some(PseudoElement::ColumnContent)
                }
            }
            if in_ua_stylesheet || PseudoElement::Viewport.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-viewport") {
                    return Some(PseudoElement::Viewport)
                }
            }
            if in_ua_stylesheet || PseudoElement::ViewportScroll.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-viewport-scroll") {
                    return Some(PseudoElement::ViewportScroll)
                }
            }
            if in_ua_stylesheet || PseudoElement::AnonymousFlexItem.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-anonymous-flex-item") {
                    return Some(PseudoElement::AnonymousFlexItem)
                }
            }
            if in_ua_stylesheet || PseudoElement::AnonymousGridItem.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-anonymous-grid-item") {
                    return Some(PseudoElement::AnonymousGridItem)
                }
            }
            if in_ua_stylesheet || PseudoElement::Ruby.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-ruby") {
                    return Some(PseudoElement::Ruby)
                }
            }
            if in_ua_stylesheet || PseudoElement::RubyBase.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-ruby-base") {
                    return Some(PseudoElement::RubyBase)
                }
            }
            if in_ua_stylesheet || PseudoElement::RubyBaseContainer.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-ruby-base-container") {
                    return Some(PseudoElement::RubyBaseContainer)
                }
            }
            if in_ua_stylesheet || PseudoElement::RubyText.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-ruby-text") {
                    return Some(PseudoElement::RubyText)
                }
            }
            if in_ua_stylesheet || PseudoElement::RubyTextContainer.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-ruby-text-container") {
                    return Some(PseudoElement::RubyTextContainer)
                }
            }
            if in_ua_stylesheet || PseudoElement::Moztreecolumn.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-tree-column") {
                    return Some(PseudoElement::Moztreecolumn)
                }
            }
            if in_ua_stylesheet || PseudoElement::Moztreerow.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-tree-row") {
                    return Some(PseudoElement::Moztreerow)
                }
            }
            if in_ua_stylesheet || PseudoElement::Moztreeseparator.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-tree-separator") {
                    return Some(PseudoElement::Moztreeseparator)
                }
            }
            if in_ua_stylesheet || PseudoElement::Moztreecell.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-tree-cell") {
                    return Some(PseudoElement::Moztreecell)
                }
            }
            if in_ua_stylesheet || PseudoElement::Moztreeindentation.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-tree-indentation") {
                    return Some(PseudoElement::Moztreeindentation)
                }
            }
            if in_ua_stylesheet || PseudoElement::Moztreeline.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-tree-line") {
                    return Some(PseudoElement::Moztreeline)
                }
            }
            if in_ua_stylesheet || PseudoElement::Moztreetwisty.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-tree-twisty") {
                    return Some(PseudoElement::Moztreetwisty)
                }
            }
            if in_ua_stylesheet || PseudoElement::Moztreeimage.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-tree-image") {
                    return Some(PseudoElement::Moztreeimage)
                }
            }
            if in_ua_stylesheet || PseudoElement::Moztreecelltext.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-tree-cell-text") {
                    return Some(PseudoElement::Moztreecelltext)
                }
            }
            if in_ua_stylesheet || PseudoElement::Moztreecheckbox.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-tree-checkbox") {
                    return Some(PseudoElement::Moztreecheckbox)
                }
            }
            if in_ua_stylesheet || PseudoElement::Moztreeprogressmeter.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-tree-progressmeter") {
                    return Some(PseudoElement::Moztreeprogressmeter)
                }
            }
            if in_ua_stylesheet || PseudoElement::Moztreedropfeedback.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-tree-drop-feedback") {
                    return Some(PseudoElement::Moztreedropfeedback)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozSVGMarkerAnonChild.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-svg-marker-anon-child") {
                    return Some(PseudoElement::MozSVGMarkerAnonChild)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozSVGOuterSVGAnonChild.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-svg-outer-svg-anon-child") {
                    return Some(PseudoElement::MozSVGOuterSVGAnonChild)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozSVGForeignContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-svg-foreign-content") {
                    return Some(PseudoElement::MozSVGForeignContent)
                }
            }
            if in_ua_stylesheet || PseudoElement::MozSVGText.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-svg-text") {
                    return Some(PseudoElement::MozSVGText)
                }
            }

        None
    }

    /// Returns the pseudo-element's definition as a string, with only one colon
    /// before it.
    pub fn as_str(&self) -> &'static str {
        match *self {
            PseudoElement::After => ":after",
            PseudoElement::Before => ":before",
            PseudoElement::Backdrop => ":backdrop",
            PseudoElement::Cue => ":cue",
            PseudoElement::FirstLetter => ":first-letter",
            PseudoElement::FirstLine => ":first-line",
            PseudoElement::MozSelection => ":-moz-selection",
            PseudoElement::MozFocusInner => ":-moz-focus-inner",
            PseudoElement::MozFocusOuter => ":-moz-focus-outer",
            PseudoElement::MozListBullet => ":-moz-list-bullet",
            PseudoElement::MozListNumber => ":-moz-list-number",
            PseudoElement::MozMathAnonymous => ":-moz-math-anonymous",
            PseudoElement::MozNumberWrapper => ":-moz-number-wrapper",
            PseudoElement::MozNumberText => ":-moz-number-text",
            PseudoElement::MozNumberSpinBox => ":-moz-number-spin-box",
            PseudoElement::MozNumberSpinUp => ":-moz-number-spin-up",
            PseudoElement::MozNumberSpinDown => ":-moz-number-spin-down",
            PseudoElement::MozProgressBar => ":-moz-progress-bar",
            PseudoElement::MozRangeTrack => ":-moz-range-track",
            PseudoElement::MozRangeProgress => ":-moz-range-progress",
            PseudoElement::MozRangeThumb => ":-moz-range-thumb",
            PseudoElement::MozMeterBar => ":-moz-meter-bar",
            PseudoElement::MozPlaceholder => ":-moz-placeholder",
            PseudoElement::Placeholder => ":placeholder",
            PseudoElement::MozColorSwatch => ":-moz-color-swatch",
            PseudoElement::MozText => ":-moz-text",
            PseudoElement::OofPlaceholder => ":-moz-oof-placeholder",
            PseudoElement::FirstLetterContinuation => ":-moz-first-letter-continuation",
            PseudoElement::MozBlockInsideInlineWrapper => ":-moz-block-inside-inline-wrapper",
            PseudoElement::MozMathMLAnonymousBlock => ":-moz-mathml-anonymous-block",
            PseudoElement::MozXULAnonymousBlock => ":-moz-xul-anonymous-block",
            PseudoElement::HorizontalFramesetBorder => ":-moz-hframeset-border",
            PseudoElement::VerticalFramesetBorder => ":-moz-vframeset-border",
            PseudoElement::MozLineFrame => ":-moz-line-frame",
            PseudoElement::ButtonContent => ":-moz-button-content",
            PseudoElement::CellContent => ":-moz-cell-content",
            PseudoElement::DropDownList => ":-moz-dropdown-list",
            PseudoElement::FieldsetContent => ":-moz-fieldset-content",
            PseudoElement::FramesetBlank => ":-moz-frameset-blank",
            PseudoElement::MozDisplayComboboxControlFrame => ":-moz-display-comboboxcontrol-frame",
            PseudoElement::HtmlCanvasContent => ":-moz-html-canvas-content",
            PseudoElement::InlineTable => ":-moz-inline-table",
            PseudoElement::Table => ":-moz-table",
            PseudoElement::TableCell => ":-moz-table-cell",
            PseudoElement::TableColGroup => ":-moz-table-column-group",
            PseudoElement::TableCol => ":-moz-table-column",
            PseudoElement::TableWrapper => ":-moz-table-wrapper",
            PseudoElement::TableRowGroup => ":-moz-table-row-group",
            PseudoElement::TableRow => ":-moz-table-row",
            PseudoElement::Canvas => ":-moz-canvas",
            PseudoElement::PageBreak => ":-moz-pagebreak",
            PseudoElement::Page => ":-moz-page",
            PseudoElement::PageContent => ":-moz-pagecontent",
            PseudoElement::PageSequence => ":-moz-page-sequence",
            PseudoElement::ScrolledContent => ":-moz-scrolled-content",
            PseudoElement::ScrolledCanvas => ":-moz-scrolled-canvas",
            PseudoElement::ScrolledPageSequence => ":-moz-scrolled-page-sequence",
            PseudoElement::ColumnContent => ":-moz-column-content",
            PseudoElement::Viewport => ":-moz-viewport",
            PseudoElement::ViewportScroll => ":-moz-viewport-scroll",
            PseudoElement::AnonymousFlexItem => ":-moz-anonymous-flex-item",
            PseudoElement::AnonymousGridItem => ":-moz-anonymous-grid-item",
            PseudoElement::Ruby => ":-moz-ruby",
            PseudoElement::RubyBase => ":-moz-ruby-base",
            PseudoElement::RubyBaseContainer => ":-moz-ruby-base-container",
            PseudoElement::RubyText => ":-moz-ruby-text",
            PseudoElement::RubyTextContainer => ":-moz-ruby-text-container",
            PseudoElement::Moztreecolumn => ":-moz-tree-column",
            PseudoElement::Moztreerow => ":-moz-tree-row",
            PseudoElement::Moztreeseparator => ":-moz-tree-separator",
            PseudoElement::Moztreecell => ":-moz-tree-cell",
            PseudoElement::Moztreeindentation => ":-moz-tree-indentation",
            PseudoElement::Moztreeline => ":-moz-tree-line",
            PseudoElement::Moztreetwisty => ":-moz-tree-twisty",
            PseudoElement::Moztreeimage => ":-moz-tree-image",
            PseudoElement::Moztreecelltext => ":-moz-tree-cell-text",
            PseudoElement::Moztreecheckbox => ":-moz-tree-checkbox",
            PseudoElement::Moztreeprogressmeter => ":-moz-tree-progressmeter",
            PseudoElement::Moztreedropfeedback => ":-moz-tree-drop-feedback",
            PseudoElement::MozSVGMarkerAnonChild => ":-moz-svg-marker-anon-child",
            PseudoElement::MozSVGOuterSVGAnonChild => ":-moz-svg-outer-svg-anon-child",
            PseudoElement::MozSVGForeignContent => ":-moz-svg-foreign-content",
            PseudoElement::MozSVGText => ":-moz-svg-text",
        }
    }
}
