<!DOCTYPE html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    
  <link rel="shortcut icon" href="/static/icons/mimetypes/py.5ef6367a.png" />

    <title>test_prototype_ident.py - DXR</title>

    
  
      <link href="/static/css/dxr.75151d98.css" rel="stylesheet" type="text/css" media="screen" />
      <link href="/static/css/forms.583492b2.css" rel="stylesheet" type="text/css" media="screen" />
      <link href="/static/css/icons.6b0c33c8.css" rel="stylesheet" type="text/css" media="screen" />
      <link href="/static/css/code-style.a87614e9.css" rel="stylesheet" type="text/css" media="screen" />
      <link href="/static/css/selector-common.949000e3.css" rel="stylesheet" type="text/css" media="screen" />
      <link href="/static/css/filter.b696a09b.css" rel="stylesheet" type="text/css" media="screen" />
    
  <link href="/static/css/tree-selector.aeb25cea.css" rel="stylesheet" type="text/css" media="screen" />

  </head>
  <body>

    <div class="header">
        <div class="help-icon">
            <div class="help-msg">
                <p>DXR is a code search and navigation tool aimed at making sense of large projects. It supports full-text and regex searches as well as structural queries.</p>
                <ul>
                    <li><a href="https://github.com/mozilla/dxr">DXR on GitHub</a></li>
                    <li><a href="https://dxr.readthedocs.org/en/latest/">How to Get Involved</a></li>
                </ul>
            </div>
        </div>
        <form method="get" action="/mozilla-central/search" id="basic_search" class="search-box">
            <fieldset>
                <div id="search-box" class="flex-container" role="group">
                    <div class="elem_container find">
                        <label for="query" class="query_label visually-hidden">Find</label>
                        <input type="text" name="q" value="" maxlength="2048" id="query" class="query" accesskey="s" title="Search" placeholder="Search mozilla-central" autocomplete="off" />
                        <div class="zero-size-container">
                          <div class="bubble">
                          </div>
                        </div>
                        <section id="search-filter" class="search-filter">
                          <button type="button" class="sf-select-trigger" aria-label="Select Filter">
                              <!-- arrow icon using icon font -->
                              <span aria-hidden="true" data-filter-arrow="&#xe801;" class="sf-selector-arrow">
                                  Operators
                              </span>
                          </button>
                        </section>
                        <div class="sf-select-options sf-modal" aria-expanded="false">
                          <ul class="selector-options" tabindex="-1">
                            
                              <li>
                                <a href="javascript:void(0)" data-value="ext:">
                                  <span class="selector-option-label">
                                    ext
                                  </span>
                                  <span class="selector-option-description">
                                    Filename extension: <code>ext:cpp</code>. Always case-sensitive.
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="id:">
                                  <span class="selector-option-label">
                                    id
                                  </span>
                                  <span class="selector-option-description">
                                    Definition of an identifier: <code>id:someFunction</code> <code>id:SomeClass</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="path:">
                                  <span class="selector-option-label">
                                    path
                                  </span>
                                  <span class="selector-option-description">
                                    File or directory sub-path to search within. <code>*</code>, <code>?</code>, and <code>[...]</code> act as shell wildcards.
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="ref:">
                                  <span class="selector-option-label">
                                    ref
                                  </span>
                                  <span class="selector-option-description">
                                    Reference to an identifier: <code>ref:someVar</code> <code>ref:someType</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="regexp:">
                                  <span class="selector-option-label">
                                    regexp
                                  </span>
                                  <span class="selector-option-description">
                                    Regular expression. Examples: <code>regexp:(?i)\bs?printf</code> <code>regexp:"(three|3) mice"</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="bases:">
                                  <span class="selector-option-label">
                                    bases
                                  </span>
                                  <span class="selector-option-description">
                                    Superclasses of a class: <code>bases:SomeSubclass</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="callers:">
                                  <span class="selector-option-label">
                                    callers
                                  </span>
                                  <span class="selector-option-description">
                                    Calls to the given function: <code>callers:some_function</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="derived:">
                                  <span class="selector-option-label">
                                    derived
                                  </span>
                                  <span class="selector-option-description">
                                    Subclasses of a class: <code>derived:SomeSuperclass</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="extern-ref:">
                                  <span class="selector-option-label">
                                    extern-ref
                                  </span>
                                  <span class="selector-option-description">
                                    References to items in external crate
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="fn-impls:">
                                  <span class="selector-option-label">
                                    fn-impls
                                  </span>
                                  <span class="selector-option-description">
                                    Function implementations
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="function:">
                                  <span class="selector-option-label">
                                    function
                                  </span>
                                  <span class="selector-option-description">
                                    Function or method definition: <code>function:foo</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="function-decl:">
                                  <span class="selector-option-label">
                                    function-decl
                                  </span>
                                  <span class="selector-option-description">
                                    Function or method declaration
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="function-ref:">
                                  <span class="selector-option-label">
                                    function-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Function or method references
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="impl:">
                                  <span class="selector-option-label">
                                    impl
                                  </span>
                                  <span class="selector-option-description">
                                    Implementations
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="macro:">
                                  <span class="selector-option-label">
                                    macro
                                  </span>
                                  <span class="selector-option-description">
                                    Macro definition
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="macro-ref:">
                                  <span class="selector-option-label">
                                    macro-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Macro uses
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="member:">
                                  <span class="selector-option-label">
                                    member
                                  </span>
                                  <span class="selector-option-description">
                                    Member variables, types, or methods of a class: <code>member:SomeClass</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="module:">
                                  <span class="selector-option-label">
                                    module
                                  </span>
                                  <span class="selector-option-description">
                                    Module definition: <code>module:module.name</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="module-alias-ref:">
                                  <span class="selector-option-label">
                                    module-alias-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Module alias references
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="module-ref:">
                                  <span class="selector-option-label">
                                    module-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Module references
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="module-use:">
                                  <span class="selector-option-label">
                                    module-use
                                  </span>
                                  <span class="selector-option-description">
                                    Module imports
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="namespace:">
                                  <span class="selector-option-label">
                                    namespace
                                  </span>
                                  <span class="selector-option-description">
                                    Namespace definition
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="namespace-alias:">
                                  <span class="selector-option-label">
                                    namespace-alias
                                  </span>
                                  <span class="selector-option-description">
                                    Namespace alias
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="namespace-alias-ref:">
                                  <span class="selector-option-label">
                                    namespace-alias-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Namespace alias references
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="namespace-ref:">
                                  <span class="selector-option-label">
                                    namespace-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Namespace references
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="overridden:">
                                  <span class="selector-option-label">
                                    overridden
                                  </span>
                                  <span class="selector-option-description">
                                    Methods which are overridden by the given one. Useful mostly with fully qualified methods, like <code>+overridden:foo.bar.some_method</code>.
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="overrides:">
                                  <span class="selector-option-label">
                                    overrides
                                  </span>
                                  <span class="selector-option-description">
                                    Methods which override the given one: <code>overrides:some_method</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="type:">
                                  <span class="selector-option-label">
                                    type
                                  </span>
                                  <span class="selector-option-description">
                                    Class definition: <code>type:Stack</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="type-decl:">
                                  <span class="selector-option-label">
                                    type-decl
                                  </span>
                                  <span class="selector-option-description">
                                    Type or class declaration
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="type-ref:">
                                  <span class="selector-option-label">
                                    type-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Type or class references, uses, or instantiations
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="var:">
                                  <span class="selector-option-label">
                                    var
                                  </span>
                                  <span class="selector-option-description">
                                    Variable definition
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="var-decl:">
                                  <span class="selector-option-label">
                                    var-decl
                                  </span>
                                  <span class="selector-option-description">
                                    Variable declaration
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="var-ref:">
                                  <span class="selector-option-label">
                                    var-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Variable uses (lvalue, rvalue, dereference, etc.)
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="warning:">
                                  <span class="selector-option-label">
                                    warning
                                  </span>
                                  <span class="selector-option-description">
                                    Compiler warning messages
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="warning-opt:">
                                  <span class="selector-option-label">
                                    warning-opt
                                  </span>
                                  <span class="selector-option-description">
                                    Warning messages brought on by a given compiler command-line option
                                  </span>
                                </a>
                              </li>
                            
                          </ul>
                        </div>
                    </div>

                    <div class="elem_container case">
                        <label for="case">
                            <input type="checkbox" name="case" id="case" class="checkbox_case" value="true" accesskey="c" /><span class="access-key">C</span>ase-sensitive
                        </label>
                    </div>
                </div>
            </fieldset>

            <input type="hidden" value="mozilla-central" id="ts-value" />
            <input type="hidden" name="redirect" value="true" id="redirect" />
            <input type="submit" value="Search" class="visually-hidden" />
        </form>
    </div>

    <div id="content" class="content">
      
  
  <div class="breadcrumbs"><a href="/mozilla-central/source">mozilla-central</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom">dom</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings">bindings</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings/parser">parser</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings/parser/tests">tests</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings/parser/tests/test_prototype_ident.py">test_prototype_ident.py</a></div>

  
  
    <section id="tree-selector" class="tree-selector">
      <button type="button" class="ts-select-trigger" aria-label="Switch Tree">
        <!-- arrow icon using icon font -->
        <span aria-hidden="true" data-icon-arrow="&#xe801;" class="selector-arrow">
          <!-- tree icon using icon font -->
          <span aria-hidden="true" data-icon="&#xe800;"></span>
          <span class='current-tree'>Switch Tree</span>
        </span>
      </button>
      <div class="select-options ts-modal" aria-expanded="false">
        <form name="options-filter" class="options-filter" data-active="false">
          <label for="filter-txt" class="visually-hidden">Filter Trees</label>
          <input type="text" name="filter-txt" id="filter-txt" placeholder="Filter trees" />
          <input type="submit" value="Filter" class="visually-hidden" />
        </form>
        <ul class="selector-options" tabindex="-1">
          
            <li>
              <a href="/build-central/parallel/dom/bindings/parser/tests/test_prototype_ident.py" >
                <span class="selector-option-label">build-central</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/comm-central/parallel/dom/bindings/parser/tests/test_prototype_ident.py" >
                <span class="selector-option-label">comm-central</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/hgcustom_version-control-tools/parallel/dom/bindings/parser/tests/test_prototype_ident.py" >
                <span class="selector-option-label">hgcustom_version-control-tools</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/mozilla-central/parallel/dom/bindings/parser/tests/test_prototype_ident.py" class="selected" aria-checked="true">
                <span class="selector-option-label">mozilla-central</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/nss/parallel/dom/bindings/parser/tests/test_prototype_ident.py" >
                <span class="selector-option-label">nss</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/rust/parallel/dom/bindings/parser/tests/test_prototype_ident.py" >
                <span class="selector-option-label">rust</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/rustfmt/parallel/dom/bindings/parser/tests/test_prototype_ident.py" >
                <span class="selector-option-label">rustfmt</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
        </ul>
      </div>
    </section>
  



  

  
    <div class="panel">
      <button id="panel-toggle">
        <span class="navpanel-icon expanded" aria-hidden="false"></span>
        Navigation
      </button>
      <section id="panel-content" aria-expanded="true" aria-hidden="false">
        
          <h4>Mercurial (05c087337043)</h4>
          <ul>
            
              <li>
                <a href="/mozilla-central/rev/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_prototype_ident.py" title="Permalink" class="permalink icon">Permalink</a>
              </li>
          </ul>
        
          <h4>Untracked file</h4>
          <ul>
            
          </ul>
        
          <h4>VCS Links</h4>
          <ul>
            
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/filelog/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_prototype_ident.py" title="Log" class="log icon">Log</a>
              </li>
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/annotate/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_prototype_ident.py" title="Blame" class="blame icon">Blame</a>
              </li>
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/diff/e7ee10b0e30371c68a9c1550c71836d375c085ae/dom/bindings/parser/tests/test_prototype_ident.py" title="Diff" class="diff icon">Diff</a>
              </li>
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/raw-file/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_prototype_ident.py" title="Raw" class="raw icon">Raw</a>
              </li>
          </ul>
        
      </section>
    </div>
  

  <div id="annotations">
    
      <div class="annotation-set" id="aset-1"></div>
      <div class="annotation-set" id="aset-2"></div>
      <div class="annotation-set" id="aset-3"></div>
      <div class="annotation-set" id="aset-4"></div>
      <div class="annotation-set" id="aset-5"></div>
      <div class="annotation-set" id="aset-6"></div>
      <div class="annotation-set" id="aset-7"></div>
      <div class="annotation-set" id="aset-8"></div>
      <div class="annotation-set" id="aset-9"></div>
      <div class="annotation-set" id="aset-10"></div>
      <div class="annotation-set" id="aset-11"></div>
      <div class="annotation-set" id="aset-12"></div>
      <div class="annotation-set" id="aset-13"></div>
      <div class="annotation-set" id="aset-14"></div>
      <div class="annotation-set" id="aset-15"></div>
      <div class="annotation-set" id="aset-16"></div>
      <div class="annotation-set" id="aset-17"></div>
      <div class="annotation-set" id="aset-18"></div>
      <div class="annotation-set" id="aset-19"></div>
      <div class="annotation-set" id="aset-20"></div>
      <div class="annotation-set" id="aset-21"></div>
      <div class="annotation-set" id="aset-22"></div>
      <div class="annotation-set" id="aset-23"></div>
      <div class="annotation-set" id="aset-24"></div>
      <div class="annotation-set" id="aset-25"></div>
      <div class="annotation-set" id="aset-26"></div>
      <div class="annotation-set" id="aset-27"></div>
      <div class="annotation-set" id="aset-28"></div>
      <div class="annotation-set" id="aset-29"></div>
      <div class="annotation-set" id="aset-30"></div>
      <div class="annotation-set" id="aset-31"></div>
      <div class="annotation-set" id="aset-32"></div>
      <div class="annotation-set" id="aset-33"></div>
      <div class="annotation-set" id="aset-34"></div>
      <div class="annotation-set" id="aset-35"></div>
      <div class="annotation-set" id="aset-36"></div>
      <div class="annotation-set" id="aset-37"></div>
      <div class="annotation-set" id="aset-38"></div>
      <div class="annotation-set" id="aset-39"></div>
      <div class="annotation-set" id="aset-40"></div>
      <div class="annotation-set" id="aset-41"></div>
      <div class="annotation-set" id="aset-42"></div>
      <div class="annotation-set" id="aset-43"></div>
      <div class="annotation-set" id="aset-44"></div>
      <div class="annotation-set" id="aset-45"></div>
      <div class="annotation-set" id="aset-46"></div>
      <div class="annotation-set" id="aset-47"></div>
      <div class="annotation-set" id="aset-48"></div>
      <div class="annotation-set" id="aset-49"></div>
      <div class="annotation-set" id="aset-50"></div>
      <div class="annotation-set" id="aset-51"></div>
      <div class="annotation-set" id="aset-52"></div>
      <div class="annotation-set" id="aset-53"></div>
      <div class="annotation-set" id="aset-54"></div>
      <div class="annotation-set" id="aset-55"></div>
      <div class="annotation-set" id="aset-56"></div>
      <div class="annotation-set" id="aset-57"></div>
      <div class="annotation-set" id="aset-58"></div>
      <div class="annotation-set" id="aset-59"></div>
      <div class="annotation-set" id="aset-60"></div>
      <div class="annotation-set" id="aset-61"></div>
      <div class="annotation-set" id="aset-62"></div>
      <div class="annotation-set" id="aset-63"></div>
      <div class="annotation-set" id="aset-64"></div>
      <div class="annotation-set" id="aset-65"></div>
      <div class="annotation-set" id="aset-66"></div>
      <div class="annotation-set" id="aset-67"></div>
      <div class="annotation-set" id="aset-68"></div>
      <div class="annotation-set" id="aset-69"></div>
      <div class="annotation-set" id="aset-70"></div>
      <div class="annotation-set" id="aset-71"></div>
      <div class="annotation-set" id="aset-72"></div>
      <div class="annotation-set" id="aset-73"></div>
      <div class="annotation-set" id="aset-74"></div>
      <div class="annotation-set" id="aset-75"></div>
      <div class="annotation-set" id="aset-76"></div>
      <div class="annotation-set" id="aset-77"></div>
      <div class="annotation-set" id="aset-78"></div>
      <div class="annotation-set" id="aset-79"></div>
      <div class="annotation-set" id="aset-80"></div></div>

  <table id="file" class="file">
    <thead class="visually-hidden">
        <th scope="col">Line</th>
        <th scope="col">Code</th>
    </thead>
    <tbody>
      <tr>
        <td id="line-numbers">
          
            <span id="1" class="line-number" unselectable="on" rel="#1">1</span>
          
            <span id="2" class="line-number" unselectable="on" rel="#2">2</span>
          
            <span id="3" class="line-number" unselectable="on" rel="#3">3</span>
          
            <span id="4" class="line-number" unselectable="on" rel="#4">4</span>
          
            <span id="5" class="line-number" unselectable="on" rel="#5">5</span>
          
            <span id="6" class="line-number" unselectable="on" rel="#6">6</span>
          
            <span id="7" class="line-number" unselectable="on" rel="#7">7</span>
          
            <span id="8" class="line-number" unselectable="on" rel="#8">8</span>
          
            <span id="9" class="line-number" unselectable="on" rel="#9">9</span>
          
            <span id="10" class="line-number" unselectable="on" rel="#10">10</span>
          
            <span id="11" class="line-number" unselectable="on" rel="#11">11</span>
          
            <span id="12" class="line-number" unselectable="on" rel="#12">12</span>
          
            <span id="13" class="line-number" unselectable="on" rel="#13">13</span>
          
            <span id="14" class="line-number" unselectable="on" rel="#14">14</span>
          
            <span id="15" class="line-number" unselectable="on" rel="#15">15</span>
          
            <span id="16" class="line-number" unselectable="on" rel="#16">16</span>
          
            <span id="17" class="line-number" unselectable="on" rel="#17">17</span>
          
            <span id="18" class="line-number" unselectable="on" rel="#18">18</span>
          
            <span id="19" class="line-number" unselectable="on" rel="#19">19</span>
          
            <span id="20" class="line-number" unselectable="on" rel="#20">20</span>
          
            <span id="21" class="line-number" unselectable="on" rel="#21">21</span>
          
            <span id="22" class="line-number" unselectable="on" rel="#22">22</span>
          
            <span id="23" class="line-number" unselectable="on" rel="#23">23</span>
          
            <span id="24" class="line-number" unselectable="on" rel="#24">24</span>
          
            <span id="25" class="line-number" unselectable="on" rel="#25">25</span>
          
            <span id="26" class="line-number" unselectable="on" rel="#26">26</span>
          
            <span id="27" class="line-number" unselectable="on" rel="#27">27</span>
          
            <span id="28" class="line-number" unselectable="on" rel="#28">28</span>
          
            <span id="29" class="line-number" unselectable="on" rel="#29">29</span>
          
            <span id="30" class="line-number" unselectable="on" rel="#30">30</span>
          
            <span id="31" class="line-number" unselectable="on" rel="#31">31</span>
          
            <span id="32" class="line-number" unselectable="on" rel="#32">32</span>
          
            <span id="33" class="line-number" unselectable="on" rel="#33">33</span>
          
            <span id="34" class="line-number" unselectable="on" rel="#34">34</span>
          
            <span id="35" class="line-number" unselectable="on" rel="#35">35</span>
          
            <span id="36" class="line-number" unselectable="on" rel="#36">36</span>
          
            <span id="37" class="line-number" unselectable="on" rel="#37">37</span>
          
            <span id="38" class="line-number" unselectable="on" rel="#38">38</span>
          
            <span id="39" class="line-number" unselectable="on" rel="#39">39</span>
          
            <span id="40" class="line-number" unselectable="on" rel="#40">40</span>
          
            <span id="41" class="line-number" unselectable="on" rel="#41">41</span>
          
            <span id="42" class="line-number" unselectable="on" rel="#42">42</span>
          
            <span id="43" class="line-number" unselectable="on" rel="#43">43</span>
          
            <span id="44" class="line-number" unselectable="on" rel="#44">44</span>
          
            <span id="45" class="line-number" unselectable="on" rel="#45">45</span>
          
            <span id="46" class="line-number" unselectable="on" rel="#46">46</span>
          
            <span id="47" class="line-number" unselectable="on" rel="#47">47</span>
          
            <span id="48" class="line-number" unselectable="on" rel="#48">48</span>
          
            <span id="49" class="line-number" unselectable="on" rel="#49">49</span>
          
            <span id="50" class="line-number" unselectable="on" rel="#50">50</span>
          
            <span id="51" class="line-number" unselectable="on" rel="#51">51</span>
          
            <span id="52" class="line-number" unselectable="on" rel="#52">52</span>
          
            <span id="53" class="line-number" unselectable="on" rel="#53">53</span>
          
            <span id="54" class="line-number" unselectable="on" rel="#54">54</span>
          
            <span id="55" class="line-number" unselectable="on" rel="#55">55</span>
          
            <span id="56" class="line-number" unselectable="on" rel="#56">56</span>
          
            <span id="57" class="line-number" unselectable="on" rel="#57">57</span>
          
            <span id="58" class="line-number" unselectable="on" rel="#58">58</span>
          
            <span id="59" class="line-number" unselectable="on" rel="#59">59</span>
          
            <span id="60" class="line-number" unselectable="on" rel="#60">60</span>
          
            <span id="61" class="line-number" unselectable="on" rel="#61">61</span>
          
            <span id="62" class="line-number" unselectable="on" rel="#62">62</span>
          
            <span id="63" class="line-number" unselectable="on" rel="#63">63</span>
          
            <span id="64" class="line-number" unselectable="on" rel="#64">64</span>
          
            <span id="65" class="line-number" unselectable="on" rel="#65">65</span>
          
            <span id="66" class="line-number" unselectable="on" rel="#66">66</span>
          
            <span id="67" class="line-number" unselectable="on" rel="#67">67</span>
          
            <span id="68" class="line-number" unselectable="on" rel="#68">68</span>
          
            <span id="69" class="line-number" unselectable="on" rel="#69">69</span>
          
            <span id="70" class="line-number" unselectable="on" rel="#70">70</span>
          
            <span id="71" class="line-number" unselectable="on" rel="#71">71</span>
          
            <span id="72" class="line-number" unselectable="on" rel="#72">72</span>
          
            <span id="73" class="line-number" unselectable="on" rel="#73">73</span>
          
            <span id="74" class="line-number" unselectable="on" rel="#74">74</span>
          
            <span id="75" class="line-number" unselectable="on" rel="#75">75</span>
          
            <span id="76" class="line-number" unselectable="on" rel="#76">76</span>
          
            <span id="77" class="line-number" unselectable="on" rel="#77">77</span>
          
            <span id="78" class="line-number" unselectable="on" rel="#78">78</span>
          
            <span id="79" class="line-number" unselectable="on" rel="#79">79</span>
          
            <span id="80" class="line-number" unselectable="on" rel="#80">80</span>
          
        </td>
        <td class="code">
          
<pre>
<code id="line-1" aria-labelledby="1"><span class="k">def</span> WebIDLTest(parser, harness):
</code><code id="line-2" aria-labelledby="2">    threw = False
</code><code id="line-3" aria-labelledby="3">    <span class="k">try</span>:
</code><code id="line-4" aria-labelledby="4">        parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-5" aria-labelledby="5"><span class="str">            interface TestIface {</span><span class="str">
</span></code><code id="line-6" aria-labelledby="6"><span class="str">              static attribute boolean prototype;</span><span class="str">
</span></code><code id="line-7" aria-labelledby="7"><span class="str">            };</span><span class="str">
</span></code><code id="line-8" aria-labelledby="8"><span class="str">        </span><span class="str">"""</span>)
</code><code id="line-9" aria-labelledby="9">        results = parser.finish()
</code><code id="line-10" aria-labelledby="10">    <span class="k">except</span>:
</code><code id="line-11" aria-labelledby="11">        threw = True
</code><code id="line-12" aria-labelledby="12">
</code><code id="line-13" aria-labelledby="13">    harness.ok(threw, <span class="str">"</span><span class="str">The identifier of a static attribute must not be </span><span class="str">'</span><span class="str">prototype</span><span class="str">'</span><span class="str">"</span>)
</code><code id="line-14" aria-labelledby="14">
</code><code id="line-15" aria-labelledby="15">    parser = parser.reset()
</code><code id="line-16" aria-labelledby="16">    threw = False
</code><code id="line-17" aria-labelledby="17">    <span class="k">try</span>:
</code><code id="line-18" aria-labelledby="18">        parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-19" aria-labelledby="19"><span class="str">            interface TestIface {</span><span class="str">
</span></code><code id="line-20" aria-labelledby="20"><span class="str">              static boolean prototype();</span><span class="str">
</span></code><code id="line-21" aria-labelledby="21"><span class="str">            };</span><span class="str">
</span></code><code id="line-22" aria-labelledby="22"><span class="str">        </span><span class="str">"""</span>)
</code><code id="line-23" aria-labelledby="23">        results = parser.finish()
</code><code id="line-24" aria-labelledby="24">    <span class="k">except</span>:
</code><code id="line-25" aria-labelledby="25">        threw = True
</code><code id="line-26" aria-labelledby="26">
</code><code id="line-27" aria-labelledby="27">    harness.ok(threw, <span class="str">"</span><span class="str">The identifier of a static operation must not be </span><span class="str">'</span><span class="str">prototype</span><span class="str">'</span><span class="str">"</span>)
</code><code id="line-28" aria-labelledby="28">
</code><code id="line-29" aria-labelledby="29">    parser = parser.reset()
</code><code id="line-30" aria-labelledby="30">    threw = False
</code><code id="line-31" aria-labelledby="31">    <span class="k">try</span>:
</code><code id="line-32" aria-labelledby="32">        parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-33" aria-labelledby="33"><span class="str">            interface TestIface {</span><span class="str">
</span></code><code id="line-34" aria-labelledby="34"><span class="str">              const boolean prototype = true;</span><span class="str">
</span></code><code id="line-35" aria-labelledby="35"><span class="str">            };</span><span class="str">
</span></code><code id="line-36" aria-labelledby="36"><span class="str">        </span><span class="str">"""</span>)
</code><code id="line-37" aria-labelledby="37">        results = parser.finish()
</code><code id="line-38" aria-labelledby="38">    <span class="k">except</span>:
</code><code id="line-39" aria-labelledby="39">        threw = True
</code><code id="line-40" aria-labelledby="40">
</code><code id="line-41" aria-labelledby="41">    harness.ok(threw, <span class="str">"</span><span class="str">The identifier of a constant must not be </span><span class="str">'</span><span class="str">prototype</span><span class="str">'</span><span class="str">"</span>)
</code><code id="line-42" aria-labelledby="42">
</code><code id="line-43" aria-labelledby="43">    <span class="c"># Make sure that we can parse non-static attributes with 'prototype' as identifier.</span>
</code><code id="line-44" aria-labelledby="44">    parser = parser.reset()
</code><code id="line-45" aria-labelledby="45">    parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-46" aria-labelledby="46"><span class="str">        interface TestIface {</span><span class="str">
</span></code><code id="line-47" aria-labelledby="47"><span class="str">          attribute boolean prototype;</span><span class="str">
</span></code><code id="line-48" aria-labelledby="48"><span class="str">        };</span><span class="str">
</span></code><code id="line-49" aria-labelledby="49"><span class="str">    </span><span class="str">"""</span>)
</code><code id="line-50" aria-labelledby="50">    results = parser.finish()
</code><code id="line-51" aria-labelledby="51">
</code><code id="line-52" aria-labelledby="52">    testIface = results[0];
</code><code id="line-53" aria-labelledby="53">    harness.check(testIface.members[0].isStatic(), False, <span class="str">"</span><span class="str">Attribute should not be static</span><span class="str">"</span>)
</code><code id="line-54" aria-labelledby="54">    harness.check(testIface.members[0].identifier.name, <span class="str">"</span><span class="str">prototype</span><span class="str">"</span>, <span class="str">"</span><span class="str">Attribute identifier should be </span><span class="str">'</span><span class="str">prototype</span><span class="str">'</span><span class="str">"</span>)
</code><code id="line-55" aria-labelledby="55">
</code><code id="line-56" aria-labelledby="56">    <span class="c"># Make sure that we can parse non-static operations with 'prototype' as identifier.</span>
</code><code id="line-57" aria-labelledby="57">    parser = parser.reset()
</code><code id="line-58" aria-labelledby="58">    parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-59" aria-labelledby="59"><span class="str">        interface TestIface {</span><span class="str">
</span></code><code id="line-60" aria-labelledby="60"><span class="str">          boolean prototype();</span><span class="str">
</span></code><code id="line-61" aria-labelledby="61"><span class="str">        };</span><span class="str">
</span></code><code id="line-62" aria-labelledby="62"><span class="str">    </span><span class="str">"""</span>)
</code><code id="line-63" aria-labelledby="63">    results = parser.finish()
</code><code id="line-64" aria-labelledby="64">
</code><code id="line-65" aria-labelledby="65">    testIface = results[0];
</code><code id="line-66" aria-labelledby="66">    harness.check(testIface.members[0].isStatic(), False, <span class="str">"</span><span class="str">Operation should not be static</span><span class="str">"</span>)
</code><code id="line-67" aria-labelledby="67">    harness.check(testIface.members[0].identifier.name, <span class="str">"</span><span class="str">prototype</span><span class="str">"</span>, <span class="str">"</span><span class="str">Operation identifier should be </span><span class="str">'</span><span class="str">prototype</span><span class="str">'</span><span class="str">"</span>)
</code><code id="line-68" aria-labelledby="68">
</code><code id="line-69" aria-labelledby="69">    <span class="c"># Make sure that we can parse dictionary members with 'prototype' as identifier.</span>
</code><code id="line-70" aria-labelledby="70">    parser = parser.reset()
</code><code id="line-71" aria-labelledby="71">    parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-72" aria-labelledby="72"><span class="str">        dictionary TestDict {</span><span class="str">
</span></code><code id="line-73" aria-labelledby="73"><span class="str">          boolean prototype;</span><span class="str">
</span></code><code id="line-74" aria-labelledby="74"><span class="str">        };</span><span class="str">
</span></code><code id="line-75" aria-labelledby="75"><span class="str">    </span><span class="str">"""</span>)
</code><code id="line-76" aria-labelledby="76">    results = parser.finish()
</code><code id="line-77" aria-labelledby="77">
</code><code id="line-78" aria-labelledby="78">    testDict = results[0];
</code><code id="line-79" aria-labelledby="79">    harness.check(testDict.members[0].identifier.name, <span class="str">"</span><span class="str">prototype</span><span class="str">"</span>, <span class="str">"</span><span class="str">Dictionary member should be </span><span class="str">'</span><span class="str">prototype</span><span class="str">'</span><span class="str">"</span>)
</code><code id="line-80" aria-labelledby="80">
</code></pre>
        </td>
      </tr>
    </tbody>
  </table>

    </div>

    
      
        <div id="foot" class="footer">
          This page was generated by DXR
          <span class="pretty-date" data-datetime="Tue, 08 Mar 2016 11:22:17 +0000"></span>.
        </div>
      
    

    

    <!-- avoid inline JS and use data attributes instead. Hackey but hey... -->
    <span id="data" data-root="" data-search="/mozilla-central/search" data-tree="mozilla-central"></span>
    <span id="state" data-offset="0" data-limit="100" data-results-line-count="" data-eof="False"></span>

    
  
      <script src="/static/js/libs/jquery203.0a6e846b.js"></script>
      <script src="/static/js/libs/nunjucks-slim.43040a7a.min.js"></script>
      <script src="/static/js/templates.58d6208e.js"></script>
      <script src="/static/js/utils.994d6cf1.js"></script>
      <script src="/static/js/dxr.d4939bfc.js"></script>
      <script src="/static/js/context_menu.8b4315f5.js"></script>
      <script src="/static/js/filter.fd0341f1.js"></script>
    
  <script src="/static/js/panel.222eee37.js"></script>
  <script src="/static/js/tree-selector.58293846.js"></script>
  <script src="/static/js/code-highlighter.9d7636ad.js"></script>


    

  </body>
</html>