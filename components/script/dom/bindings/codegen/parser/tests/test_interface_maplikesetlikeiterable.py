<!DOCTYPE html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    
  <link rel="shortcut icon" href="/static/icons/mimetypes/py.5ef6367a.png" />

    <title>test_interface_maplikesetlikeiterable.py - DXR</title>

    
  
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
      
  
  <div class="breadcrumbs"><a href="/mozilla-central/source">mozilla-central</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom">dom</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings">bindings</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings/parser">parser</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings/parser/tests">tests</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings/parser/tests/test_interface_maplikesetlikeiterable.py">test_interface_maplikesetlikeiterable.py</a></div>

  
  
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
              <a href="/build-central/parallel/dom/bindings/parser/tests/test_interface_maplikesetlikeiterable.py" >
                <span class="selector-option-label">build-central</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/comm-central/parallel/dom/bindings/parser/tests/test_interface_maplikesetlikeiterable.py" >
                <span class="selector-option-label">comm-central</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/hgcustom_version-control-tools/parallel/dom/bindings/parser/tests/test_interface_maplikesetlikeiterable.py" >
                <span class="selector-option-label">hgcustom_version-control-tools</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/mozilla-central/parallel/dom/bindings/parser/tests/test_interface_maplikesetlikeiterable.py" class="selected" aria-checked="true">
                <span class="selector-option-label">mozilla-central</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/nss/parallel/dom/bindings/parser/tests/test_interface_maplikesetlikeiterable.py" >
                <span class="selector-option-label">nss</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/rust/parallel/dom/bindings/parser/tests/test_interface_maplikesetlikeiterable.py" >
                <span class="selector-option-label">rust</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/rustfmt/parallel/dom/bindings/parser/tests/test_interface_maplikesetlikeiterable.py" >
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
                <a href="/mozilla-central/rev/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_interface_maplikesetlikeiterable.py" title="Permalink" class="permalink icon">Permalink</a>
              </li>
          </ul>
        
          <h4>Untracked file</h4>
          <ul>
            
          </ul>
        
          <h4>VCS Links</h4>
          <ul>
            
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/filelog/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_interface_maplikesetlikeiterable.py" title="Log" class="log icon">Log</a>
              </li>
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/annotate/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_interface_maplikesetlikeiterable.py" title="Blame" class="blame icon">Blame</a>
              </li>
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/diff/410ef34da2e776afa850dfbc06a45ac54c1c758d/dom/bindings/parser/tests/test_interface_maplikesetlikeiterable.py" title="Diff" class="diff icon">Diff</a>
              </li>
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/raw-file/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_interface_maplikesetlikeiterable.py" title="Raw" class="raw icon">Raw</a>
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
      <div class="annotation-set" id="aset-80"></div>
      <div class="annotation-set" id="aset-81"></div>
      <div class="annotation-set" id="aset-82"></div>
      <div class="annotation-set" id="aset-83"></div>
      <div class="annotation-set" id="aset-84"></div>
      <div class="annotation-set" id="aset-85"></div>
      <div class="annotation-set" id="aset-86"></div>
      <div class="annotation-set" id="aset-87"></div>
      <div class="annotation-set" id="aset-88"></div>
      <div class="annotation-set" id="aset-89"></div>
      <div class="annotation-set" id="aset-90"></div>
      <div class="annotation-set" id="aset-91"></div>
      <div class="annotation-set" id="aset-92"></div>
      <div class="annotation-set" id="aset-93"></div>
      <div class="annotation-set" id="aset-94"></div>
      <div class="annotation-set" id="aset-95"></div>
      <div class="annotation-set" id="aset-96"></div>
      <div class="annotation-set" id="aset-97"></div>
      <div class="annotation-set" id="aset-98"></div>
      <div class="annotation-set" id="aset-99"></div>
      <div class="annotation-set" id="aset-100"></div>
      <div class="annotation-set" id="aset-101"></div>
      <div class="annotation-set" id="aset-102"></div>
      <div class="annotation-set" id="aset-103"></div>
      <div class="annotation-set" id="aset-104"></div>
      <div class="annotation-set" id="aset-105"></div>
      <div class="annotation-set" id="aset-106"></div>
      <div class="annotation-set" id="aset-107"></div>
      <div class="annotation-set" id="aset-108"></div>
      <div class="annotation-set" id="aset-109"></div>
      <div class="annotation-set" id="aset-110"></div>
      <div class="annotation-set" id="aset-111"></div>
      <div class="annotation-set" id="aset-112"></div>
      <div class="annotation-set" id="aset-113"></div>
      <div class="annotation-set" id="aset-114"></div>
      <div class="annotation-set" id="aset-115"></div>
      <div class="annotation-set" id="aset-116"></div>
      <div class="annotation-set" id="aset-117"></div>
      <div class="annotation-set" id="aset-118"></div>
      <div class="annotation-set" id="aset-119"></div>
      <div class="annotation-set" id="aset-120"></div>
      <div class="annotation-set" id="aset-121"></div>
      <div class="annotation-set" id="aset-122"></div>
      <div class="annotation-set" id="aset-123"></div>
      <div class="annotation-set" id="aset-124"></div>
      <div class="annotation-set" id="aset-125"></div>
      <div class="annotation-set" id="aset-126"></div>
      <div class="annotation-set" id="aset-127"></div>
      <div class="annotation-set" id="aset-128"></div>
      <div class="annotation-set" id="aset-129"></div>
      <div class="annotation-set" id="aset-130"></div>
      <div class="annotation-set" id="aset-131"></div>
      <div class="annotation-set" id="aset-132"></div>
      <div class="annotation-set" id="aset-133"></div>
      <div class="annotation-set" id="aset-134"></div>
      <div class="annotation-set" id="aset-135"></div>
      <div class="annotation-set" id="aset-136"></div>
      <div class="annotation-set" id="aset-137"></div>
      <div class="annotation-set" id="aset-138"></div>
      <div class="annotation-set" id="aset-139"></div>
      <div class="annotation-set" id="aset-140"></div>
      <div class="annotation-set" id="aset-141"></div>
      <div class="annotation-set" id="aset-142"></div>
      <div class="annotation-set" id="aset-143"></div>
      <div class="annotation-set" id="aset-144"></div>
      <div class="annotation-set" id="aset-145"></div>
      <div class="annotation-set" id="aset-146"></div>
      <div class="annotation-set" id="aset-147"></div>
      <div class="annotation-set" id="aset-148"></div>
      <div class="annotation-set" id="aset-149"></div>
      <div class="annotation-set" id="aset-150"></div>
      <div class="annotation-set" id="aset-151"></div>
      <div class="annotation-set" id="aset-152"></div>
      <div class="annotation-set" id="aset-153"></div>
      <div class="annotation-set" id="aset-154"></div>
      <div class="annotation-set" id="aset-155"></div>
      <div class="annotation-set" id="aset-156"></div>
      <div class="annotation-set" id="aset-157"></div>
      <div class="annotation-set" id="aset-158"></div>
      <div class="annotation-set" id="aset-159"></div>
      <div class="annotation-set" id="aset-160"></div>
      <div class="annotation-set" id="aset-161"></div>
      <div class="annotation-set" id="aset-162"></div>
      <div class="annotation-set" id="aset-163"></div>
      <div class="annotation-set" id="aset-164"></div>
      <div class="annotation-set" id="aset-165"></div>
      <div class="annotation-set" id="aset-166"></div>
      <div class="annotation-set" id="aset-167"></div>
      <div class="annotation-set" id="aset-168"></div>
      <div class="annotation-set" id="aset-169"></div>
      <div class="annotation-set" id="aset-170"></div>
      <div class="annotation-set" id="aset-171"></div>
      <div class="annotation-set" id="aset-172"></div>
      <div class="annotation-set" id="aset-173"></div>
      <div class="annotation-set" id="aset-174"></div>
      <div class="annotation-set" id="aset-175"></div>
      <div class="annotation-set" id="aset-176"></div>
      <div class="annotation-set" id="aset-177"></div>
      <div class="annotation-set" id="aset-178"></div>
      <div class="annotation-set" id="aset-179"></div>
      <div class="annotation-set" id="aset-180"></div>
      <div class="annotation-set" id="aset-181"></div>
      <div class="annotation-set" id="aset-182"></div>
      <div class="annotation-set" id="aset-183"></div>
      <div class="annotation-set" id="aset-184"></div>
      <div class="annotation-set" id="aset-185"></div>
      <div class="annotation-set" id="aset-186"></div>
      <div class="annotation-set" id="aset-187"></div>
      <div class="annotation-set" id="aset-188"></div>
      <div class="annotation-set" id="aset-189"></div>
      <div class="annotation-set" id="aset-190"></div>
      <div class="annotation-set" id="aset-191"></div>
      <div class="annotation-set" id="aset-192"></div>
      <div class="annotation-set" id="aset-193"></div>
      <div class="annotation-set" id="aset-194"></div>
      <div class="annotation-set" id="aset-195"></div>
      <div class="annotation-set" id="aset-196"></div>
      <div class="annotation-set" id="aset-197"></div>
      <div class="annotation-set" id="aset-198"></div>
      <div class="annotation-set" id="aset-199"></div>
      <div class="annotation-set" id="aset-200"></div>
      <div class="annotation-set" id="aset-201"></div>
      <div class="annotation-set" id="aset-202"></div>
      <div class="annotation-set" id="aset-203"></div>
      <div class="annotation-set" id="aset-204"></div>
      <div class="annotation-set" id="aset-205"></div>
      <div class="annotation-set" id="aset-206"></div>
      <div class="annotation-set" id="aset-207"></div>
      <div class="annotation-set" id="aset-208"></div>
      <div class="annotation-set" id="aset-209"></div>
      <div class="annotation-set" id="aset-210"></div>
      <div class="annotation-set" id="aset-211"></div>
      <div class="annotation-set" id="aset-212"></div>
      <div class="annotation-set" id="aset-213"></div>
      <div class="annotation-set" id="aset-214"></div>
      <div class="annotation-set" id="aset-215"></div>
      <div class="annotation-set" id="aset-216"></div>
      <div class="annotation-set" id="aset-217"></div>
      <div class="annotation-set" id="aset-218"></div>
      <div class="annotation-set" id="aset-219"></div>
      <div class="annotation-set" id="aset-220"></div>
      <div class="annotation-set" id="aset-221"></div>
      <div class="annotation-set" id="aset-222"></div>
      <div class="annotation-set" id="aset-223"></div>
      <div class="annotation-set" id="aset-224"></div>
      <div class="annotation-set" id="aset-225"></div>
      <div class="annotation-set" id="aset-226"></div>
      <div class="annotation-set" id="aset-227"></div>
      <div class="annotation-set" id="aset-228"></div>
      <div class="annotation-set" id="aset-229"></div>
      <div class="annotation-set" id="aset-230"></div>
      <div class="annotation-set" id="aset-231"></div>
      <div class="annotation-set" id="aset-232"></div>
      <div class="annotation-set" id="aset-233"></div>
      <div class="annotation-set" id="aset-234"></div>
      <div class="annotation-set" id="aset-235"></div>
      <div class="annotation-set" id="aset-236"></div>
      <div class="annotation-set" id="aset-237"></div>
      <div class="annotation-set" id="aset-238"></div>
      <div class="annotation-set" id="aset-239"></div>
      <div class="annotation-set" id="aset-240"></div>
      <div class="annotation-set" id="aset-241"></div>
      <div class="annotation-set" id="aset-242"></div>
      <div class="annotation-set" id="aset-243"></div>
      <div class="annotation-set" id="aset-244"></div>
      <div class="annotation-set" id="aset-245"></div>
      <div class="annotation-set" id="aset-246"></div>
      <div class="annotation-set" id="aset-247"></div>
      <div class="annotation-set" id="aset-248"></div>
      <div class="annotation-set" id="aset-249"></div>
      <div class="annotation-set" id="aset-250"></div>
      <div class="annotation-set" id="aset-251"></div>
      <div class="annotation-set" id="aset-252"></div>
      <div class="annotation-set" id="aset-253"></div>
      <div class="annotation-set" id="aset-254"></div>
      <div class="annotation-set" id="aset-255"></div>
      <div class="annotation-set" id="aset-256"></div>
      <div class="annotation-set" id="aset-257"></div>
      <div class="annotation-set" id="aset-258"></div>
      <div class="annotation-set" id="aset-259"></div>
      <div class="annotation-set" id="aset-260"></div>
      <div class="annotation-set" id="aset-261"></div>
      <div class="annotation-set" id="aset-262"></div>
      <div class="annotation-set" id="aset-263"></div>
      <div class="annotation-set" id="aset-264"></div>
      <div class="annotation-set" id="aset-265"></div>
      <div class="annotation-set" id="aset-266"></div>
      <div class="annotation-set" id="aset-267"></div>
      <div class="annotation-set" id="aset-268"></div>
      <div class="annotation-set" id="aset-269"></div>
      <div class="annotation-set" id="aset-270"></div>
      <div class="annotation-set" id="aset-271"></div>
      <div class="annotation-set" id="aset-272"></div>
      <div class="annotation-set" id="aset-273"></div>
      <div class="annotation-set" id="aset-274"></div>
      <div class="annotation-set" id="aset-275"></div>
      <div class="annotation-set" id="aset-276"></div>
      <div class="annotation-set" id="aset-277"></div>
      <div class="annotation-set" id="aset-278"></div>
      <div class="annotation-set" id="aset-279"></div>
      <div class="annotation-set" id="aset-280"></div>
      <div class="annotation-set" id="aset-281"></div>
      <div class="annotation-set" id="aset-282"></div>
      <div class="annotation-set" id="aset-283"></div>
      <div class="annotation-set" id="aset-284"></div>
      <div class="annotation-set" id="aset-285"></div>
      <div class="annotation-set" id="aset-286"></div>
      <div class="annotation-set" id="aset-287"></div>
      <div class="annotation-set" id="aset-288"></div>
      <div class="annotation-set" id="aset-289"></div>
      <div class="annotation-set" id="aset-290"></div>
      <div class="annotation-set" id="aset-291"></div>
      <div class="annotation-set" id="aset-292"></div>
      <div class="annotation-set" id="aset-293"></div>
      <div class="annotation-set" id="aset-294"></div>
      <div class="annotation-set" id="aset-295"></div>
      <div class="annotation-set" id="aset-296"></div>
      <div class="annotation-set" id="aset-297"></div>
      <div class="annotation-set" id="aset-298"></div>
      <div class="annotation-set" id="aset-299"></div>
      <div class="annotation-set" id="aset-300"></div>
      <div class="annotation-set" id="aset-301"></div>
      <div class="annotation-set" id="aset-302"></div>
      <div class="annotation-set" id="aset-303"></div>
      <div class="annotation-set" id="aset-304"></div>
      <div class="annotation-set" id="aset-305"></div>
      <div class="annotation-set" id="aset-306"></div>
      <div class="annotation-set" id="aset-307"></div>
      <div class="annotation-set" id="aset-308"></div>
      <div class="annotation-set" id="aset-309"></div>
      <div class="annotation-set" id="aset-310"></div>
      <div class="annotation-set" id="aset-311"></div>
      <div class="annotation-set" id="aset-312"></div>
      <div class="annotation-set" id="aset-313"></div>
      <div class="annotation-set" id="aset-314"></div>
      <div class="annotation-set" id="aset-315"></div>
      <div class="annotation-set" id="aset-316"></div>
      <div class="annotation-set" id="aset-317"></div>
      <div class="annotation-set" id="aset-318"></div>
      <div class="annotation-set" id="aset-319"></div>
      <div class="annotation-set" id="aset-320"></div>
      <div class="annotation-set" id="aset-321"></div>
      <div class="annotation-set" id="aset-322"></div>
      <div class="annotation-set" id="aset-323"></div>
      <div class="annotation-set" id="aset-324"></div>
      <div class="annotation-set" id="aset-325"></div>
      <div class="annotation-set" id="aset-326"></div>
      <div class="annotation-set" id="aset-327"></div>
      <div class="annotation-set" id="aset-328"></div>
      <div class="annotation-set" id="aset-329"></div>
      <div class="annotation-set" id="aset-330"></div>
      <div class="annotation-set" id="aset-331"></div>
      <div class="annotation-set" id="aset-332"></div>
      <div class="annotation-set" id="aset-333"></div>
      <div class="annotation-set" id="aset-334"></div>
      <div class="annotation-set" id="aset-335"></div>
      <div class="annotation-set" id="aset-336"></div>
      <div class="annotation-set" id="aset-337"></div>
      <div class="annotation-set" id="aset-338"></div>
      <div class="annotation-set" id="aset-339"></div>
      <div class="annotation-set" id="aset-340"></div>
      <div class="annotation-set" id="aset-341"></div>
      <div class="annotation-set" id="aset-342"></div>
      <div class="annotation-set" id="aset-343"></div>
      <div class="annotation-set" id="aset-344"></div>
      <div class="annotation-set" id="aset-345"></div>
      <div class="annotation-set" id="aset-346"></div>
      <div class="annotation-set" id="aset-347"></div>
      <div class="annotation-set" id="aset-348"></div>
      <div class="annotation-set" id="aset-349"></div>
      <div class="annotation-set" id="aset-350"></div>
      <div class="annotation-set" id="aset-351"></div>
      <div class="annotation-set" id="aset-352"></div>
      <div class="annotation-set" id="aset-353"></div>
      <div class="annotation-set" id="aset-354"></div>
      <div class="annotation-set" id="aset-355"></div>
      <div class="annotation-set" id="aset-356"></div>
      <div class="annotation-set" id="aset-357"></div>
      <div class="annotation-set" id="aset-358"></div>
      <div class="annotation-set" id="aset-359"></div>
      <div class="annotation-set" id="aset-360"></div>
      <div class="annotation-set" id="aset-361"></div>
      <div class="annotation-set" id="aset-362"></div>
      <div class="annotation-set" id="aset-363"></div>
      <div class="annotation-set" id="aset-364"></div>
      <div class="annotation-set" id="aset-365"></div>
      <div class="annotation-set" id="aset-366"></div>
      <div class="annotation-set" id="aset-367"></div>
      <div class="annotation-set" id="aset-368"></div>
      <div class="annotation-set" id="aset-369"></div>
      <div class="annotation-set" id="aset-370"></div>
      <div class="annotation-set" id="aset-371"></div>
      <div class="annotation-set" id="aset-372"></div>
      <div class="annotation-set" id="aset-373"></div>
      <div class="annotation-set" id="aset-374"></div>
      <div class="annotation-set" id="aset-375"></div>
      <div class="annotation-set" id="aset-376"></div>
      <div class="annotation-set" id="aset-377"></div>
      <div class="annotation-set" id="aset-378"></div>
      <div class="annotation-set" id="aset-379"></div>
      <div class="annotation-set" id="aset-380"></div>
      <div class="annotation-set" id="aset-381"></div>
      <div class="annotation-set" id="aset-382"></div>
      <div class="annotation-set" id="aset-383"></div>
      <div class="annotation-set" id="aset-384"></div>
      <div class="annotation-set" id="aset-385"></div>
      <div class="annotation-set" id="aset-386"></div>
      <div class="annotation-set" id="aset-387"></div>
      <div class="annotation-set" id="aset-388"></div>
      <div class="annotation-set" id="aset-389"></div>
      <div class="annotation-set" id="aset-390"></div>
      <div class="annotation-set" id="aset-391"></div>
      <div class="annotation-set" id="aset-392"></div>
      <div class="annotation-set" id="aset-393"></div>
      <div class="annotation-set" id="aset-394"></div>
      <div class="annotation-set" id="aset-395"></div>
      <div class="annotation-set" id="aset-396"></div>
      <div class="annotation-set" id="aset-397"></div>
      <div class="annotation-set" id="aset-398"></div>
      <div class="annotation-set" id="aset-399"></div>
      <div class="annotation-set" id="aset-400"></div>
      <div class="annotation-set" id="aset-401"></div>
      <div class="annotation-set" id="aset-402"></div>
      <div class="annotation-set" id="aset-403"></div>
      <div class="annotation-set" id="aset-404"></div>
      <div class="annotation-set" id="aset-405"></div>
      <div class="annotation-set" id="aset-406"></div>
      <div class="annotation-set" id="aset-407"></div>
      <div class="annotation-set" id="aset-408"></div>
      <div class="annotation-set" id="aset-409"></div>
      <div class="annotation-set" id="aset-410"></div>
      <div class="annotation-set" id="aset-411"></div>
      <div class="annotation-set" id="aset-412"></div>
      <div class="annotation-set" id="aset-413"></div>
      <div class="annotation-set" id="aset-414"></div>
      <div class="annotation-set" id="aset-415"></div>
      <div class="annotation-set" id="aset-416"></div>
      <div class="annotation-set" id="aset-417"></div>
      <div class="annotation-set" id="aset-418"></div>
      <div class="annotation-set" id="aset-419"></div>
      <div class="annotation-set" id="aset-420"></div>
      <div class="annotation-set" id="aset-421"></div>
      <div class="annotation-set" id="aset-422"></div>
      <div class="annotation-set" id="aset-423"></div>
      <div class="annotation-set" id="aset-424"></div>
      <div class="annotation-set" id="aset-425"></div>
      <div class="annotation-set" id="aset-426"></div>
      <div class="annotation-set" id="aset-427"></div>
      <div class="annotation-set" id="aset-428"></div>
      <div class="annotation-set" id="aset-429"></div>
      <div class="annotation-set" id="aset-430"></div>
      <div class="annotation-set" id="aset-431"></div>
      <div class="annotation-set" id="aset-432"></div>
      <div class="annotation-set" id="aset-433"></div>
      <div class="annotation-set" id="aset-434"></div>
      <div class="annotation-set" id="aset-435"></div>
      <div class="annotation-set" id="aset-436"></div>
      <div class="annotation-set" id="aset-437"></div>
      <div class="annotation-set" id="aset-438"></div>
      <div class="annotation-set" id="aset-439"></div>
      <div class="annotation-set" id="aset-440"></div>
      <div class="annotation-set" id="aset-441"></div>
      <div class="annotation-set" id="aset-442"></div>
      <div class="annotation-set" id="aset-443"></div>
      <div class="annotation-set" id="aset-444"></div>
      <div class="annotation-set" id="aset-445"></div>
      <div class="annotation-set" id="aset-446"></div>
      <div class="annotation-set" id="aset-447"></div>
      <div class="annotation-set" id="aset-448"></div>
      <div class="annotation-set" id="aset-449"></div>
      <div class="annotation-set" id="aset-450"></div>
      <div class="annotation-set" id="aset-451"></div>
      <div class="annotation-set" id="aset-452"></div>
      <div class="annotation-set" id="aset-453"></div>
      <div class="annotation-set" id="aset-454"></div>
      <div class="annotation-set" id="aset-455"></div>
      <div class="annotation-set" id="aset-456"></div>
      <div class="annotation-set" id="aset-457"></div>
      <div class="annotation-set" id="aset-458"></div>
      <div class="annotation-set" id="aset-459"></div>
      <div class="annotation-set" id="aset-460"></div>
      <div class="annotation-set" id="aset-461"></div>
      <div class="annotation-set" id="aset-462"></div>
      <div class="annotation-set" id="aset-463"></div>
      <div class="annotation-set" id="aset-464"></div>
      <div class="annotation-set" id="aset-465"></div>
      <div class="annotation-set" id="aset-466"></div>
      <div class="annotation-set" id="aset-467"></div>
      <div class="annotation-set" id="aset-468"></div>
      <div class="annotation-set" id="aset-469"></div>
      <div class="annotation-set" id="aset-470"></div>
      <div class="annotation-set" id="aset-471"></div>
      <div class="annotation-set" id="aset-472"></div>
      <div class="annotation-set" id="aset-473"></div>
      <div class="annotation-set" id="aset-474"></div>
      <div class="annotation-set" id="aset-475"></div>
      <div class="annotation-set" id="aset-476"></div>
      <div class="annotation-set" id="aset-477"></div>
      <div class="annotation-set" id="aset-478"></div>
      <div class="annotation-set" id="aset-479"></div>
      <div class="annotation-set" id="aset-480"></div>
      <div class="annotation-set" id="aset-481"></div>
      <div class="annotation-set" id="aset-482"></div>
      <div class="annotation-set" id="aset-483"></div>
      <div class="annotation-set" id="aset-484"></div>
      <div class="annotation-set" id="aset-485"></div>
      <div class="annotation-set" id="aset-486"></div>
      <div class="annotation-set" id="aset-487"></div>
      <div class="annotation-set" id="aset-488"></div>
      <div class="annotation-set" id="aset-489"></div>
      <div class="annotation-set" id="aset-490"></div>
      <div class="annotation-set" id="aset-491"></div>
      <div class="annotation-set" id="aset-492"></div>
      <div class="annotation-set" id="aset-493"></div>
      <div class="annotation-set" id="aset-494"></div>
      <div class="annotation-set" id="aset-495"></div>
      <div class="annotation-set" id="aset-496"></div>
      <div class="annotation-set" id="aset-497"></div>
      <div class="annotation-set" id="aset-498"></div>
      <div class="annotation-set" id="aset-499"></div>
      <div class="annotation-set" id="aset-500"></div>
      <div class="annotation-set" id="aset-501"></div>
      <div class="annotation-set" id="aset-502"></div>
      <div class="annotation-set" id="aset-503"></div>
      <div class="annotation-set" id="aset-504"></div>
      <div class="annotation-set" id="aset-505"></div>
      <div class="annotation-set" id="aset-506"></div>
      <div class="annotation-set" id="aset-507"></div>
      <div class="annotation-set" id="aset-508"></div>
      <div class="annotation-set" id="aset-509"></div>
      <div class="annotation-set" id="aset-510"></div>
      <div class="annotation-set" id="aset-511"></div>
      <div class="annotation-set" id="aset-512"></div>
      <div class="annotation-set" id="aset-513"></div>
      <div class="annotation-set" id="aset-514"></div>
      <div class="annotation-set" id="aset-515"></div>
      <div class="annotation-set" id="aset-516"></div>
      <div class="annotation-set" id="aset-517"></div>
      <div class="annotation-set" id="aset-518"></div>
      <div class="annotation-set" id="aset-519"></div>
      <div class="annotation-set" id="aset-520"></div>
      <div class="annotation-set" id="aset-521"></div>
      <div class="annotation-set" id="aset-522"></div>
      <div class="annotation-set" id="aset-523"></div>
      <div class="annotation-set" id="aset-524"></div>
      <div class="annotation-set" id="aset-525"></div>
      <div class="annotation-set" id="aset-526"></div>
      <div class="annotation-set" id="aset-527"></div>
      <div class="annotation-set" id="aset-528"></div>
      <div class="annotation-set" id="aset-529"></div>
      <div class="annotation-set" id="aset-530"></div>
      <div class="annotation-set" id="aset-531"></div>
      <div class="annotation-set" id="aset-532"></div>
      <div class="annotation-set" id="aset-533"></div>
      <div class="annotation-set" id="aset-534"></div>
      <div class="annotation-set" id="aset-535"></div>
      <div class="annotation-set" id="aset-536"></div>
      <div class="annotation-set" id="aset-537"></div>
      <div class="annotation-set" id="aset-538"></div>
      <div class="annotation-set" id="aset-539"></div>
      <div class="annotation-set" id="aset-540"></div>
      <div class="annotation-set" id="aset-541"></div>
      <div class="annotation-set" id="aset-542"></div>
      <div class="annotation-set" id="aset-543"></div>
      <div class="annotation-set" id="aset-544"></div>
      <div class="annotation-set" id="aset-545"></div>
      <div class="annotation-set" id="aset-546"></div>
      <div class="annotation-set" id="aset-547"></div>
      <div class="annotation-set" id="aset-548"></div>
      <div class="annotation-set" id="aset-549"></div>
      <div class="annotation-set" id="aset-550"></div>
      <div class="annotation-set" id="aset-551"></div>
      <div class="annotation-set" id="aset-552"></div>
      <div class="annotation-set" id="aset-553"></div>
      <div class="annotation-set" id="aset-554"></div>
      <div class="annotation-set" id="aset-555"></div>
      <div class="annotation-set" id="aset-556"></div>
      <div class="annotation-set" id="aset-557"></div>
      <div class="annotation-set" id="aset-558"></div>
      <div class="annotation-set" id="aset-559"></div>
      <div class="annotation-set" id="aset-560"></div>
      <div class="annotation-set" id="aset-561"></div>
      <div class="annotation-set" id="aset-562"></div>
      <div class="annotation-set" id="aset-563"></div>
      <div class="annotation-set" id="aset-564"></div>
      <div class="annotation-set" id="aset-565"></div>
      <div class="annotation-set" id="aset-566"></div>
      <div class="annotation-set" id="aset-567"></div>
      <div class="annotation-set" id="aset-568"></div>
      <div class="annotation-set" id="aset-569"></div>
      <div class="annotation-set" id="aset-570"></div>
      <div class="annotation-set" id="aset-571"></div>
      <div class="annotation-set" id="aset-572"></div>
      <div class="annotation-set" id="aset-573"></div>
      <div class="annotation-set" id="aset-574"></div>
      <div class="annotation-set" id="aset-575"></div>
      <div class="annotation-set" id="aset-576"></div>
      <div class="annotation-set" id="aset-577"></div>
      <div class="annotation-set" id="aset-578"></div>
      <div class="annotation-set" id="aset-579"></div>
      <div class="annotation-set" id="aset-580"></div>
      <div class="annotation-set" id="aset-581"></div>
      <div class="annotation-set" id="aset-582"></div>
      <div class="annotation-set" id="aset-583"></div>
      <div class="annotation-set" id="aset-584"></div>
      <div class="annotation-set" id="aset-585"></div>
      <div class="annotation-set" id="aset-586"></div>
      <div class="annotation-set" id="aset-587"></div>
      <div class="annotation-set" id="aset-588"></div>
      <div class="annotation-set" id="aset-589"></div>
      <div class="annotation-set" id="aset-590"></div>
      <div class="annotation-set" id="aset-591"></div>
      <div class="annotation-set" id="aset-592"></div>
      <div class="annotation-set" id="aset-593"></div>
      <div class="annotation-set" id="aset-594"></div></div>

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
          
            <span id="81" class="line-number" unselectable="on" rel="#81">81</span>
          
            <span id="82" class="line-number" unselectable="on" rel="#82">82</span>
          
            <span id="83" class="line-number" unselectable="on" rel="#83">83</span>
          
            <span id="84" class="line-number" unselectable="on" rel="#84">84</span>
          
            <span id="85" class="line-number" unselectable="on" rel="#85">85</span>
          
            <span id="86" class="line-number" unselectable="on" rel="#86">86</span>
          
            <span id="87" class="line-number" unselectable="on" rel="#87">87</span>
          
            <span id="88" class="line-number" unselectable="on" rel="#88">88</span>
          
            <span id="89" class="line-number" unselectable="on" rel="#89">89</span>
          
            <span id="90" class="line-number" unselectable="on" rel="#90">90</span>
          
            <span id="91" class="line-number" unselectable="on" rel="#91">91</span>
          
            <span id="92" class="line-number" unselectable="on" rel="#92">92</span>
          
            <span id="93" class="line-number" unselectable="on" rel="#93">93</span>
          
            <span id="94" class="line-number" unselectable="on" rel="#94">94</span>
          
            <span id="95" class="line-number" unselectable="on" rel="#95">95</span>
          
            <span id="96" class="line-number" unselectable="on" rel="#96">96</span>
          
            <span id="97" class="line-number" unselectable="on" rel="#97">97</span>
          
            <span id="98" class="line-number" unselectable="on" rel="#98">98</span>
          
            <span id="99" class="line-number" unselectable="on" rel="#99">99</span>
          
            <span id="100" class="line-number" unselectable="on" rel="#100">100</span>
          
            <span id="101" class="line-number" unselectable="on" rel="#101">101</span>
          
            <span id="102" class="line-number" unselectable="on" rel="#102">102</span>
          
            <span id="103" class="line-number" unselectable="on" rel="#103">103</span>
          
            <span id="104" class="line-number" unselectable="on" rel="#104">104</span>
          
            <span id="105" class="line-number" unselectable="on" rel="#105">105</span>
          
            <span id="106" class="line-number" unselectable="on" rel="#106">106</span>
          
            <span id="107" class="line-number" unselectable="on" rel="#107">107</span>
          
            <span id="108" class="line-number" unselectable="on" rel="#108">108</span>
          
            <span id="109" class="line-number" unselectable="on" rel="#109">109</span>
          
            <span id="110" class="line-number" unselectable="on" rel="#110">110</span>
          
            <span id="111" class="line-number" unselectable="on" rel="#111">111</span>
          
            <span id="112" class="line-number" unselectable="on" rel="#112">112</span>
          
            <span id="113" class="line-number" unselectable="on" rel="#113">113</span>
          
            <span id="114" class="line-number" unselectable="on" rel="#114">114</span>
          
            <span id="115" class="line-number" unselectable="on" rel="#115">115</span>
          
            <span id="116" class="line-number" unselectable="on" rel="#116">116</span>
          
            <span id="117" class="line-number" unselectable="on" rel="#117">117</span>
          
            <span id="118" class="line-number" unselectable="on" rel="#118">118</span>
          
            <span id="119" class="line-number" unselectable="on" rel="#119">119</span>
          
            <span id="120" class="line-number" unselectable="on" rel="#120">120</span>
          
            <span id="121" class="line-number" unselectable="on" rel="#121">121</span>
          
            <span id="122" class="line-number" unselectable="on" rel="#122">122</span>
          
            <span id="123" class="line-number" unselectable="on" rel="#123">123</span>
          
            <span id="124" class="line-number" unselectable="on" rel="#124">124</span>
          
            <span id="125" class="line-number" unselectable="on" rel="#125">125</span>
          
            <span id="126" class="line-number" unselectable="on" rel="#126">126</span>
          
            <span id="127" class="line-number" unselectable="on" rel="#127">127</span>
          
            <span id="128" class="line-number" unselectable="on" rel="#128">128</span>
          
            <span id="129" class="line-number" unselectable="on" rel="#129">129</span>
          
            <span id="130" class="line-number" unselectable="on" rel="#130">130</span>
          
            <span id="131" class="line-number" unselectable="on" rel="#131">131</span>
          
            <span id="132" class="line-number" unselectable="on" rel="#132">132</span>
          
            <span id="133" class="line-number" unselectable="on" rel="#133">133</span>
          
            <span id="134" class="line-number" unselectable="on" rel="#134">134</span>
          
            <span id="135" class="line-number" unselectable="on" rel="#135">135</span>
          
            <span id="136" class="line-number" unselectable="on" rel="#136">136</span>
          
            <span id="137" class="line-number" unselectable="on" rel="#137">137</span>
          
            <span id="138" class="line-number" unselectable="on" rel="#138">138</span>
          
            <span id="139" class="line-number" unselectable="on" rel="#139">139</span>
          
            <span id="140" class="line-number" unselectable="on" rel="#140">140</span>
          
            <span id="141" class="line-number" unselectable="on" rel="#141">141</span>
          
            <span id="142" class="line-number" unselectable="on" rel="#142">142</span>
          
            <span id="143" class="line-number" unselectable="on" rel="#143">143</span>
          
            <span id="144" class="line-number" unselectable="on" rel="#144">144</span>
          
            <span id="145" class="line-number" unselectable="on" rel="#145">145</span>
          
            <span id="146" class="line-number" unselectable="on" rel="#146">146</span>
          
            <span id="147" class="line-number" unselectable="on" rel="#147">147</span>
          
            <span id="148" class="line-number" unselectable="on" rel="#148">148</span>
          
            <span id="149" class="line-number" unselectable="on" rel="#149">149</span>
          
            <span id="150" class="line-number" unselectable="on" rel="#150">150</span>
          
            <span id="151" class="line-number" unselectable="on" rel="#151">151</span>
          
            <span id="152" class="line-number" unselectable="on" rel="#152">152</span>
          
            <span id="153" class="line-number" unselectable="on" rel="#153">153</span>
          
            <span id="154" class="line-number" unselectable="on" rel="#154">154</span>
          
            <span id="155" class="line-number" unselectable="on" rel="#155">155</span>
          
            <span id="156" class="line-number" unselectable="on" rel="#156">156</span>
          
            <span id="157" class="line-number" unselectable="on" rel="#157">157</span>
          
            <span id="158" class="line-number" unselectable="on" rel="#158">158</span>
          
            <span id="159" class="line-number" unselectable="on" rel="#159">159</span>
          
            <span id="160" class="line-number" unselectable="on" rel="#160">160</span>
          
            <span id="161" class="line-number" unselectable="on" rel="#161">161</span>
          
            <span id="162" class="line-number" unselectable="on" rel="#162">162</span>
          
            <span id="163" class="line-number" unselectable="on" rel="#163">163</span>
          
            <span id="164" class="line-number" unselectable="on" rel="#164">164</span>
          
            <span id="165" class="line-number" unselectable="on" rel="#165">165</span>
          
            <span id="166" class="line-number" unselectable="on" rel="#166">166</span>
          
            <span id="167" class="line-number" unselectable="on" rel="#167">167</span>
          
            <span id="168" class="line-number" unselectable="on" rel="#168">168</span>
          
            <span id="169" class="line-number" unselectable="on" rel="#169">169</span>
          
            <span id="170" class="line-number" unselectable="on" rel="#170">170</span>
          
            <span id="171" class="line-number" unselectable="on" rel="#171">171</span>
          
            <span id="172" class="line-number" unselectable="on" rel="#172">172</span>
          
            <span id="173" class="line-number" unselectable="on" rel="#173">173</span>
          
            <span id="174" class="line-number" unselectable="on" rel="#174">174</span>
          
            <span id="175" class="line-number" unselectable="on" rel="#175">175</span>
          
            <span id="176" class="line-number" unselectable="on" rel="#176">176</span>
          
            <span id="177" class="line-number" unselectable="on" rel="#177">177</span>
          
            <span id="178" class="line-number" unselectable="on" rel="#178">178</span>
          
            <span id="179" class="line-number" unselectable="on" rel="#179">179</span>
          
            <span id="180" class="line-number" unselectable="on" rel="#180">180</span>
          
            <span id="181" class="line-number" unselectable="on" rel="#181">181</span>
          
            <span id="182" class="line-number" unselectable="on" rel="#182">182</span>
          
            <span id="183" class="line-number" unselectable="on" rel="#183">183</span>
          
            <span id="184" class="line-number" unselectable="on" rel="#184">184</span>
          
            <span id="185" class="line-number" unselectable="on" rel="#185">185</span>
          
            <span id="186" class="line-number" unselectable="on" rel="#186">186</span>
          
            <span id="187" class="line-number" unselectable="on" rel="#187">187</span>
          
            <span id="188" class="line-number" unselectable="on" rel="#188">188</span>
          
            <span id="189" class="line-number" unselectable="on" rel="#189">189</span>
          
            <span id="190" class="line-number" unselectable="on" rel="#190">190</span>
          
            <span id="191" class="line-number" unselectable="on" rel="#191">191</span>
          
            <span id="192" class="line-number" unselectable="on" rel="#192">192</span>
          
            <span id="193" class="line-number" unselectable="on" rel="#193">193</span>
          
            <span id="194" class="line-number" unselectable="on" rel="#194">194</span>
          
            <span id="195" class="line-number" unselectable="on" rel="#195">195</span>
          
            <span id="196" class="line-number" unselectable="on" rel="#196">196</span>
          
            <span id="197" class="line-number" unselectable="on" rel="#197">197</span>
          
            <span id="198" class="line-number" unselectable="on" rel="#198">198</span>
          
            <span id="199" class="line-number" unselectable="on" rel="#199">199</span>
          
            <span id="200" class="line-number" unselectable="on" rel="#200">200</span>
          
            <span id="201" class="line-number" unselectable="on" rel="#201">201</span>
          
            <span id="202" class="line-number" unselectable="on" rel="#202">202</span>
          
            <span id="203" class="line-number" unselectable="on" rel="#203">203</span>
          
            <span id="204" class="line-number" unselectable="on" rel="#204">204</span>
          
            <span id="205" class="line-number" unselectable="on" rel="#205">205</span>
          
            <span id="206" class="line-number" unselectable="on" rel="#206">206</span>
          
            <span id="207" class="line-number" unselectable="on" rel="#207">207</span>
          
            <span id="208" class="line-number" unselectable="on" rel="#208">208</span>
          
            <span id="209" class="line-number" unselectable="on" rel="#209">209</span>
          
            <span id="210" class="line-number" unselectable="on" rel="#210">210</span>
          
            <span id="211" class="line-number" unselectable="on" rel="#211">211</span>
          
            <span id="212" class="line-number" unselectable="on" rel="#212">212</span>
          
            <span id="213" class="line-number" unselectable="on" rel="#213">213</span>
          
            <span id="214" class="line-number" unselectable="on" rel="#214">214</span>
          
            <span id="215" class="line-number" unselectable="on" rel="#215">215</span>
          
            <span id="216" class="line-number" unselectable="on" rel="#216">216</span>
          
            <span id="217" class="line-number" unselectable="on" rel="#217">217</span>
          
            <span id="218" class="line-number" unselectable="on" rel="#218">218</span>
          
            <span id="219" class="line-number" unselectable="on" rel="#219">219</span>
          
            <span id="220" class="line-number" unselectable="on" rel="#220">220</span>
          
            <span id="221" class="line-number" unselectable="on" rel="#221">221</span>
          
            <span id="222" class="line-number" unselectable="on" rel="#222">222</span>
          
            <span id="223" class="line-number" unselectable="on" rel="#223">223</span>
          
            <span id="224" class="line-number" unselectable="on" rel="#224">224</span>
          
            <span id="225" class="line-number" unselectable="on" rel="#225">225</span>
          
            <span id="226" class="line-number" unselectable="on" rel="#226">226</span>
          
            <span id="227" class="line-number" unselectable="on" rel="#227">227</span>
          
            <span id="228" class="line-number" unselectable="on" rel="#228">228</span>
          
            <span id="229" class="line-number" unselectable="on" rel="#229">229</span>
          
            <span id="230" class="line-number" unselectable="on" rel="#230">230</span>
          
            <span id="231" class="line-number" unselectable="on" rel="#231">231</span>
          
            <span id="232" class="line-number" unselectable="on" rel="#232">232</span>
          
            <span id="233" class="line-number" unselectable="on" rel="#233">233</span>
          
            <span id="234" class="line-number" unselectable="on" rel="#234">234</span>
          
            <span id="235" class="line-number" unselectable="on" rel="#235">235</span>
          
            <span id="236" class="line-number" unselectable="on" rel="#236">236</span>
          
            <span id="237" class="line-number" unselectable="on" rel="#237">237</span>
          
            <span id="238" class="line-number" unselectable="on" rel="#238">238</span>
          
            <span id="239" class="line-number" unselectable="on" rel="#239">239</span>
          
            <span id="240" class="line-number" unselectable="on" rel="#240">240</span>
          
            <span id="241" class="line-number" unselectable="on" rel="#241">241</span>
          
            <span id="242" class="line-number" unselectable="on" rel="#242">242</span>
          
            <span id="243" class="line-number" unselectable="on" rel="#243">243</span>
          
            <span id="244" class="line-number" unselectable="on" rel="#244">244</span>
          
            <span id="245" class="line-number" unselectable="on" rel="#245">245</span>
          
            <span id="246" class="line-number" unselectable="on" rel="#246">246</span>
          
            <span id="247" class="line-number" unselectable="on" rel="#247">247</span>
          
            <span id="248" class="line-number" unselectable="on" rel="#248">248</span>
          
            <span id="249" class="line-number" unselectable="on" rel="#249">249</span>
          
            <span id="250" class="line-number" unselectable="on" rel="#250">250</span>
          
            <span id="251" class="line-number" unselectable="on" rel="#251">251</span>
          
            <span id="252" class="line-number" unselectable="on" rel="#252">252</span>
          
            <span id="253" class="line-number" unselectable="on" rel="#253">253</span>
          
            <span id="254" class="line-number" unselectable="on" rel="#254">254</span>
          
            <span id="255" class="line-number" unselectable="on" rel="#255">255</span>
          
            <span id="256" class="line-number" unselectable="on" rel="#256">256</span>
          
            <span id="257" class="line-number" unselectable="on" rel="#257">257</span>
          
            <span id="258" class="line-number" unselectable="on" rel="#258">258</span>
          
            <span id="259" class="line-number" unselectable="on" rel="#259">259</span>
          
            <span id="260" class="line-number" unselectable="on" rel="#260">260</span>
          
            <span id="261" class="line-number" unselectable="on" rel="#261">261</span>
          
            <span id="262" class="line-number" unselectable="on" rel="#262">262</span>
          
            <span id="263" class="line-number" unselectable="on" rel="#263">263</span>
          
            <span id="264" class="line-number" unselectable="on" rel="#264">264</span>
          
            <span id="265" class="line-number" unselectable="on" rel="#265">265</span>
          
            <span id="266" class="line-number" unselectable="on" rel="#266">266</span>
          
            <span id="267" class="line-number" unselectable="on" rel="#267">267</span>
          
            <span id="268" class="line-number" unselectable="on" rel="#268">268</span>
          
            <span id="269" class="line-number" unselectable="on" rel="#269">269</span>
          
            <span id="270" class="line-number" unselectable="on" rel="#270">270</span>
          
            <span id="271" class="line-number" unselectable="on" rel="#271">271</span>
          
            <span id="272" class="line-number" unselectable="on" rel="#272">272</span>
          
            <span id="273" class="line-number" unselectable="on" rel="#273">273</span>
          
            <span id="274" class="line-number" unselectable="on" rel="#274">274</span>
          
            <span id="275" class="line-number" unselectable="on" rel="#275">275</span>
          
            <span id="276" class="line-number" unselectable="on" rel="#276">276</span>
          
            <span id="277" class="line-number" unselectable="on" rel="#277">277</span>
          
            <span id="278" class="line-number" unselectable="on" rel="#278">278</span>
          
            <span id="279" class="line-number" unselectable="on" rel="#279">279</span>
          
            <span id="280" class="line-number" unselectable="on" rel="#280">280</span>
          
            <span id="281" class="line-number" unselectable="on" rel="#281">281</span>
          
            <span id="282" class="line-number" unselectable="on" rel="#282">282</span>
          
            <span id="283" class="line-number" unselectable="on" rel="#283">283</span>
          
            <span id="284" class="line-number" unselectable="on" rel="#284">284</span>
          
            <span id="285" class="line-number" unselectable="on" rel="#285">285</span>
          
            <span id="286" class="line-number" unselectable="on" rel="#286">286</span>
          
            <span id="287" class="line-number" unselectable="on" rel="#287">287</span>
          
            <span id="288" class="line-number" unselectable="on" rel="#288">288</span>
          
            <span id="289" class="line-number" unselectable="on" rel="#289">289</span>
          
            <span id="290" class="line-number" unselectable="on" rel="#290">290</span>
          
            <span id="291" class="line-number" unselectable="on" rel="#291">291</span>
          
            <span id="292" class="line-number" unselectable="on" rel="#292">292</span>
          
            <span id="293" class="line-number" unselectable="on" rel="#293">293</span>
          
            <span id="294" class="line-number" unselectable="on" rel="#294">294</span>
          
            <span id="295" class="line-number" unselectable="on" rel="#295">295</span>
          
            <span id="296" class="line-number" unselectable="on" rel="#296">296</span>
          
            <span id="297" class="line-number" unselectable="on" rel="#297">297</span>
          
            <span id="298" class="line-number" unselectable="on" rel="#298">298</span>
          
            <span id="299" class="line-number" unselectable="on" rel="#299">299</span>
          
            <span id="300" class="line-number" unselectable="on" rel="#300">300</span>
          
            <span id="301" class="line-number" unselectable="on" rel="#301">301</span>
          
            <span id="302" class="line-number" unselectable="on" rel="#302">302</span>
          
            <span id="303" class="line-number" unselectable="on" rel="#303">303</span>
          
            <span id="304" class="line-number" unselectable="on" rel="#304">304</span>
          
            <span id="305" class="line-number" unselectable="on" rel="#305">305</span>
          
            <span id="306" class="line-number" unselectable="on" rel="#306">306</span>
          
            <span id="307" class="line-number" unselectable="on" rel="#307">307</span>
          
            <span id="308" class="line-number" unselectable="on" rel="#308">308</span>
          
            <span id="309" class="line-number" unselectable="on" rel="#309">309</span>
          
            <span id="310" class="line-number" unselectable="on" rel="#310">310</span>
          
            <span id="311" class="line-number" unselectable="on" rel="#311">311</span>
          
            <span id="312" class="line-number" unselectable="on" rel="#312">312</span>
          
            <span id="313" class="line-number" unselectable="on" rel="#313">313</span>
          
            <span id="314" class="line-number" unselectable="on" rel="#314">314</span>
          
            <span id="315" class="line-number" unselectable="on" rel="#315">315</span>
          
            <span id="316" class="line-number" unselectable="on" rel="#316">316</span>
          
            <span id="317" class="line-number" unselectable="on" rel="#317">317</span>
          
            <span id="318" class="line-number" unselectable="on" rel="#318">318</span>
          
            <span id="319" class="line-number" unselectable="on" rel="#319">319</span>
          
            <span id="320" class="line-number" unselectable="on" rel="#320">320</span>
          
            <span id="321" class="line-number" unselectable="on" rel="#321">321</span>
          
            <span id="322" class="line-number" unselectable="on" rel="#322">322</span>
          
            <span id="323" class="line-number" unselectable="on" rel="#323">323</span>
          
            <span id="324" class="line-number" unselectable="on" rel="#324">324</span>
          
            <span id="325" class="line-number" unselectable="on" rel="#325">325</span>
          
            <span id="326" class="line-number" unselectable="on" rel="#326">326</span>
          
            <span id="327" class="line-number" unselectable="on" rel="#327">327</span>
          
            <span id="328" class="line-number" unselectable="on" rel="#328">328</span>
          
            <span id="329" class="line-number" unselectable="on" rel="#329">329</span>
          
            <span id="330" class="line-number" unselectable="on" rel="#330">330</span>
          
            <span id="331" class="line-number" unselectable="on" rel="#331">331</span>
          
            <span id="332" class="line-number" unselectable="on" rel="#332">332</span>
          
            <span id="333" class="line-number" unselectable="on" rel="#333">333</span>
          
            <span id="334" class="line-number" unselectable="on" rel="#334">334</span>
          
            <span id="335" class="line-number" unselectable="on" rel="#335">335</span>
          
            <span id="336" class="line-number" unselectable="on" rel="#336">336</span>
          
            <span id="337" class="line-number" unselectable="on" rel="#337">337</span>
          
            <span id="338" class="line-number" unselectable="on" rel="#338">338</span>
          
            <span id="339" class="line-number" unselectable="on" rel="#339">339</span>
          
            <span id="340" class="line-number" unselectable="on" rel="#340">340</span>
          
            <span id="341" class="line-number" unselectable="on" rel="#341">341</span>
          
            <span id="342" class="line-number" unselectable="on" rel="#342">342</span>
          
            <span id="343" class="line-number" unselectable="on" rel="#343">343</span>
          
            <span id="344" class="line-number" unselectable="on" rel="#344">344</span>
          
            <span id="345" class="line-number" unselectable="on" rel="#345">345</span>
          
            <span id="346" class="line-number" unselectable="on" rel="#346">346</span>
          
            <span id="347" class="line-number" unselectable="on" rel="#347">347</span>
          
            <span id="348" class="line-number" unselectable="on" rel="#348">348</span>
          
            <span id="349" class="line-number" unselectable="on" rel="#349">349</span>
          
            <span id="350" class="line-number" unselectable="on" rel="#350">350</span>
          
            <span id="351" class="line-number" unselectable="on" rel="#351">351</span>
          
            <span id="352" class="line-number" unselectable="on" rel="#352">352</span>
          
            <span id="353" class="line-number" unselectable="on" rel="#353">353</span>
          
            <span id="354" class="line-number" unselectable="on" rel="#354">354</span>
          
            <span id="355" class="line-number" unselectable="on" rel="#355">355</span>
          
            <span id="356" class="line-number" unselectable="on" rel="#356">356</span>
          
            <span id="357" class="line-number" unselectable="on" rel="#357">357</span>
          
            <span id="358" class="line-number" unselectable="on" rel="#358">358</span>
          
            <span id="359" class="line-number" unselectable="on" rel="#359">359</span>
          
            <span id="360" class="line-number" unselectable="on" rel="#360">360</span>
          
            <span id="361" class="line-number" unselectable="on" rel="#361">361</span>
          
            <span id="362" class="line-number" unselectable="on" rel="#362">362</span>
          
            <span id="363" class="line-number" unselectable="on" rel="#363">363</span>
          
            <span id="364" class="line-number" unselectable="on" rel="#364">364</span>
          
            <span id="365" class="line-number" unselectable="on" rel="#365">365</span>
          
            <span id="366" class="line-number" unselectable="on" rel="#366">366</span>
          
            <span id="367" class="line-number" unselectable="on" rel="#367">367</span>
          
            <span id="368" class="line-number" unselectable="on" rel="#368">368</span>
          
            <span id="369" class="line-number" unselectable="on" rel="#369">369</span>
          
            <span id="370" class="line-number" unselectable="on" rel="#370">370</span>
          
            <span id="371" class="line-number" unselectable="on" rel="#371">371</span>
          
            <span id="372" class="line-number" unselectable="on" rel="#372">372</span>
          
            <span id="373" class="line-number" unselectable="on" rel="#373">373</span>
          
            <span id="374" class="line-number" unselectable="on" rel="#374">374</span>
          
            <span id="375" class="line-number" unselectable="on" rel="#375">375</span>
          
            <span id="376" class="line-number" unselectable="on" rel="#376">376</span>
          
            <span id="377" class="line-number" unselectable="on" rel="#377">377</span>
          
            <span id="378" class="line-number" unselectable="on" rel="#378">378</span>
          
            <span id="379" class="line-number" unselectable="on" rel="#379">379</span>
          
            <span id="380" class="line-number" unselectable="on" rel="#380">380</span>
          
            <span id="381" class="line-number" unselectable="on" rel="#381">381</span>
          
            <span id="382" class="line-number" unselectable="on" rel="#382">382</span>
          
            <span id="383" class="line-number" unselectable="on" rel="#383">383</span>
          
            <span id="384" class="line-number" unselectable="on" rel="#384">384</span>
          
            <span id="385" class="line-number" unselectable="on" rel="#385">385</span>
          
            <span id="386" class="line-number" unselectable="on" rel="#386">386</span>
          
            <span id="387" class="line-number" unselectable="on" rel="#387">387</span>
          
            <span id="388" class="line-number" unselectable="on" rel="#388">388</span>
          
            <span id="389" class="line-number" unselectable="on" rel="#389">389</span>
          
            <span id="390" class="line-number" unselectable="on" rel="#390">390</span>
          
            <span id="391" class="line-number" unselectable="on" rel="#391">391</span>
          
            <span id="392" class="line-number" unselectable="on" rel="#392">392</span>
          
            <span id="393" class="line-number" unselectable="on" rel="#393">393</span>
          
            <span id="394" class="line-number" unselectable="on" rel="#394">394</span>
          
            <span id="395" class="line-number" unselectable="on" rel="#395">395</span>
          
            <span id="396" class="line-number" unselectable="on" rel="#396">396</span>
          
            <span id="397" class="line-number" unselectable="on" rel="#397">397</span>
          
            <span id="398" class="line-number" unselectable="on" rel="#398">398</span>
          
            <span id="399" class="line-number" unselectable="on" rel="#399">399</span>
          
            <span id="400" class="line-number" unselectable="on" rel="#400">400</span>
          
            <span id="401" class="line-number" unselectable="on" rel="#401">401</span>
          
            <span id="402" class="line-number" unselectable="on" rel="#402">402</span>
          
            <span id="403" class="line-number" unselectable="on" rel="#403">403</span>
          
            <span id="404" class="line-number" unselectable="on" rel="#404">404</span>
          
            <span id="405" class="line-number" unselectable="on" rel="#405">405</span>
          
            <span id="406" class="line-number" unselectable="on" rel="#406">406</span>
          
            <span id="407" class="line-number" unselectable="on" rel="#407">407</span>
          
            <span id="408" class="line-number" unselectable="on" rel="#408">408</span>
          
            <span id="409" class="line-number" unselectable="on" rel="#409">409</span>
          
            <span id="410" class="line-number" unselectable="on" rel="#410">410</span>
          
            <span id="411" class="line-number" unselectable="on" rel="#411">411</span>
          
            <span id="412" class="line-number" unselectable="on" rel="#412">412</span>
          
            <span id="413" class="line-number" unselectable="on" rel="#413">413</span>
          
            <span id="414" class="line-number" unselectable="on" rel="#414">414</span>
          
            <span id="415" class="line-number" unselectable="on" rel="#415">415</span>
          
            <span id="416" class="line-number" unselectable="on" rel="#416">416</span>
          
            <span id="417" class="line-number" unselectable="on" rel="#417">417</span>
          
            <span id="418" class="line-number" unselectable="on" rel="#418">418</span>
          
            <span id="419" class="line-number" unselectable="on" rel="#419">419</span>
          
            <span id="420" class="line-number" unselectable="on" rel="#420">420</span>
          
            <span id="421" class="line-number" unselectable="on" rel="#421">421</span>
          
            <span id="422" class="line-number" unselectable="on" rel="#422">422</span>
          
            <span id="423" class="line-number" unselectable="on" rel="#423">423</span>
          
            <span id="424" class="line-number" unselectable="on" rel="#424">424</span>
          
            <span id="425" class="line-number" unselectable="on" rel="#425">425</span>
          
            <span id="426" class="line-number" unselectable="on" rel="#426">426</span>
          
            <span id="427" class="line-number" unselectable="on" rel="#427">427</span>
          
            <span id="428" class="line-number" unselectable="on" rel="#428">428</span>
          
            <span id="429" class="line-number" unselectable="on" rel="#429">429</span>
          
            <span id="430" class="line-number" unselectable="on" rel="#430">430</span>
          
            <span id="431" class="line-number" unselectable="on" rel="#431">431</span>
          
            <span id="432" class="line-number" unselectable="on" rel="#432">432</span>
          
            <span id="433" class="line-number" unselectable="on" rel="#433">433</span>
          
            <span id="434" class="line-number" unselectable="on" rel="#434">434</span>
          
            <span id="435" class="line-number" unselectable="on" rel="#435">435</span>
          
            <span id="436" class="line-number" unselectable="on" rel="#436">436</span>
          
            <span id="437" class="line-number" unselectable="on" rel="#437">437</span>
          
            <span id="438" class="line-number" unselectable="on" rel="#438">438</span>
          
            <span id="439" class="line-number" unselectable="on" rel="#439">439</span>
          
            <span id="440" class="line-number" unselectable="on" rel="#440">440</span>
          
            <span id="441" class="line-number" unselectable="on" rel="#441">441</span>
          
            <span id="442" class="line-number" unselectable="on" rel="#442">442</span>
          
            <span id="443" class="line-number" unselectable="on" rel="#443">443</span>
          
            <span id="444" class="line-number" unselectable="on" rel="#444">444</span>
          
            <span id="445" class="line-number" unselectable="on" rel="#445">445</span>
          
            <span id="446" class="line-number" unselectable="on" rel="#446">446</span>
          
            <span id="447" class="line-number" unselectable="on" rel="#447">447</span>
          
            <span id="448" class="line-number" unselectable="on" rel="#448">448</span>
          
            <span id="449" class="line-number" unselectable="on" rel="#449">449</span>
          
            <span id="450" class="line-number" unselectable="on" rel="#450">450</span>
          
            <span id="451" class="line-number" unselectable="on" rel="#451">451</span>
          
            <span id="452" class="line-number" unselectable="on" rel="#452">452</span>
          
            <span id="453" class="line-number" unselectable="on" rel="#453">453</span>
          
            <span id="454" class="line-number" unselectable="on" rel="#454">454</span>
          
            <span id="455" class="line-number" unselectable="on" rel="#455">455</span>
          
            <span id="456" class="line-number" unselectable="on" rel="#456">456</span>
          
            <span id="457" class="line-number" unselectable="on" rel="#457">457</span>
          
            <span id="458" class="line-number" unselectable="on" rel="#458">458</span>
          
            <span id="459" class="line-number" unselectable="on" rel="#459">459</span>
          
            <span id="460" class="line-number" unselectable="on" rel="#460">460</span>
          
            <span id="461" class="line-number" unselectable="on" rel="#461">461</span>
          
            <span id="462" class="line-number" unselectable="on" rel="#462">462</span>
          
            <span id="463" class="line-number" unselectable="on" rel="#463">463</span>
          
            <span id="464" class="line-number" unselectable="on" rel="#464">464</span>
          
            <span id="465" class="line-number" unselectable="on" rel="#465">465</span>
          
            <span id="466" class="line-number" unselectable="on" rel="#466">466</span>
          
            <span id="467" class="line-number" unselectable="on" rel="#467">467</span>
          
            <span id="468" class="line-number" unselectable="on" rel="#468">468</span>
          
            <span id="469" class="line-number" unselectable="on" rel="#469">469</span>
          
            <span id="470" class="line-number" unselectable="on" rel="#470">470</span>
          
            <span id="471" class="line-number" unselectable="on" rel="#471">471</span>
          
            <span id="472" class="line-number" unselectable="on" rel="#472">472</span>
          
            <span id="473" class="line-number" unselectable="on" rel="#473">473</span>
          
            <span id="474" class="line-number" unselectable="on" rel="#474">474</span>
          
            <span id="475" class="line-number" unselectable="on" rel="#475">475</span>
          
            <span id="476" class="line-number" unselectable="on" rel="#476">476</span>
          
            <span id="477" class="line-number" unselectable="on" rel="#477">477</span>
          
            <span id="478" class="line-number" unselectable="on" rel="#478">478</span>
          
            <span id="479" class="line-number" unselectable="on" rel="#479">479</span>
          
            <span id="480" class="line-number" unselectable="on" rel="#480">480</span>
          
            <span id="481" class="line-number" unselectable="on" rel="#481">481</span>
          
            <span id="482" class="line-number" unselectable="on" rel="#482">482</span>
          
            <span id="483" class="line-number" unselectable="on" rel="#483">483</span>
          
            <span id="484" class="line-number" unselectable="on" rel="#484">484</span>
          
            <span id="485" class="line-number" unselectable="on" rel="#485">485</span>
          
            <span id="486" class="line-number" unselectable="on" rel="#486">486</span>
          
            <span id="487" class="line-number" unselectable="on" rel="#487">487</span>
          
            <span id="488" class="line-number" unselectable="on" rel="#488">488</span>
          
            <span id="489" class="line-number" unselectable="on" rel="#489">489</span>
          
            <span id="490" class="line-number" unselectable="on" rel="#490">490</span>
          
            <span id="491" class="line-number" unselectable="on" rel="#491">491</span>
          
            <span id="492" class="line-number" unselectable="on" rel="#492">492</span>
          
            <span id="493" class="line-number" unselectable="on" rel="#493">493</span>
          
            <span id="494" class="line-number" unselectable="on" rel="#494">494</span>
          
            <span id="495" class="line-number" unselectable="on" rel="#495">495</span>
          
            <span id="496" class="line-number" unselectable="on" rel="#496">496</span>
          
            <span id="497" class="line-number" unselectable="on" rel="#497">497</span>
          
            <span id="498" class="line-number" unselectable="on" rel="#498">498</span>
          
            <span id="499" class="line-number" unselectable="on" rel="#499">499</span>
          
            <span id="500" class="line-number" unselectable="on" rel="#500">500</span>
          
            <span id="501" class="line-number" unselectable="on" rel="#501">501</span>
          
            <span id="502" class="line-number" unselectable="on" rel="#502">502</span>
          
            <span id="503" class="line-number" unselectable="on" rel="#503">503</span>
          
            <span id="504" class="line-number" unselectable="on" rel="#504">504</span>
          
            <span id="505" class="line-number" unselectable="on" rel="#505">505</span>
          
            <span id="506" class="line-number" unselectable="on" rel="#506">506</span>
          
            <span id="507" class="line-number" unselectable="on" rel="#507">507</span>
          
            <span id="508" class="line-number" unselectable="on" rel="#508">508</span>
          
            <span id="509" class="line-number" unselectable="on" rel="#509">509</span>
          
            <span id="510" class="line-number" unselectable="on" rel="#510">510</span>
          
            <span id="511" class="line-number" unselectable="on" rel="#511">511</span>
          
            <span id="512" class="line-number" unselectable="on" rel="#512">512</span>
          
            <span id="513" class="line-number" unselectable="on" rel="#513">513</span>
          
            <span id="514" class="line-number" unselectable="on" rel="#514">514</span>
          
            <span id="515" class="line-number" unselectable="on" rel="#515">515</span>
          
            <span id="516" class="line-number" unselectable="on" rel="#516">516</span>
          
            <span id="517" class="line-number" unselectable="on" rel="#517">517</span>
          
            <span id="518" class="line-number" unselectable="on" rel="#518">518</span>
          
            <span id="519" class="line-number" unselectable="on" rel="#519">519</span>
          
            <span id="520" class="line-number" unselectable="on" rel="#520">520</span>
          
            <span id="521" class="line-number" unselectable="on" rel="#521">521</span>
          
            <span id="522" class="line-number" unselectable="on" rel="#522">522</span>
          
            <span id="523" class="line-number" unselectable="on" rel="#523">523</span>
          
            <span id="524" class="line-number" unselectable="on" rel="#524">524</span>
          
            <span id="525" class="line-number" unselectable="on" rel="#525">525</span>
          
            <span id="526" class="line-number" unselectable="on" rel="#526">526</span>
          
            <span id="527" class="line-number" unselectable="on" rel="#527">527</span>
          
            <span id="528" class="line-number" unselectable="on" rel="#528">528</span>
          
            <span id="529" class="line-number" unselectable="on" rel="#529">529</span>
          
            <span id="530" class="line-number" unselectable="on" rel="#530">530</span>
          
            <span id="531" class="line-number" unselectable="on" rel="#531">531</span>
          
            <span id="532" class="line-number" unselectable="on" rel="#532">532</span>
          
            <span id="533" class="line-number" unselectable="on" rel="#533">533</span>
          
            <span id="534" class="line-number" unselectable="on" rel="#534">534</span>
          
            <span id="535" class="line-number" unselectable="on" rel="#535">535</span>
          
            <span id="536" class="line-number" unselectable="on" rel="#536">536</span>
          
            <span id="537" class="line-number" unselectable="on" rel="#537">537</span>
          
            <span id="538" class="line-number" unselectable="on" rel="#538">538</span>
          
            <span id="539" class="line-number" unselectable="on" rel="#539">539</span>
          
            <span id="540" class="line-number" unselectable="on" rel="#540">540</span>
          
            <span id="541" class="line-number" unselectable="on" rel="#541">541</span>
          
            <span id="542" class="line-number" unselectable="on" rel="#542">542</span>
          
            <span id="543" class="line-number" unselectable="on" rel="#543">543</span>
          
            <span id="544" class="line-number" unselectable="on" rel="#544">544</span>
          
            <span id="545" class="line-number" unselectable="on" rel="#545">545</span>
          
            <span id="546" class="line-number" unselectable="on" rel="#546">546</span>
          
            <span id="547" class="line-number" unselectable="on" rel="#547">547</span>
          
            <span id="548" class="line-number" unselectable="on" rel="#548">548</span>
          
            <span id="549" class="line-number" unselectable="on" rel="#549">549</span>
          
            <span id="550" class="line-number" unselectable="on" rel="#550">550</span>
          
            <span id="551" class="line-number" unselectable="on" rel="#551">551</span>
          
            <span id="552" class="line-number" unselectable="on" rel="#552">552</span>
          
            <span id="553" class="line-number" unselectable="on" rel="#553">553</span>
          
            <span id="554" class="line-number" unselectable="on" rel="#554">554</span>
          
            <span id="555" class="line-number" unselectable="on" rel="#555">555</span>
          
            <span id="556" class="line-number" unselectable="on" rel="#556">556</span>
          
            <span id="557" class="line-number" unselectable="on" rel="#557">557</span>
          
            <span id="558" class="line-number" unselectable="on" rel="#558">558</span>
          
            <span id="559" class="line-number" unselectable="on" rel="#559">559</span>
          
            <span id="560" class="line-number" unselectable="on" rel="#560">560</span>
          
            <span id="561" class="line-number" unselectable="on" rel="#561">561</span>
          
            <span id="562" class="line-number" unselectable="on" rel="#562">562</span>
          
            <span id="563" class="line-number" unselectable="on" rel="#563">563</span>
          
            <span id="564" class="line-number" unselectable="on" rel="#564">564</span>
          
            <span id="565" class="line-number" unselectable="on" rel="#565">565</span>
          
            <span id="566" class="line-number" unselectable="on" rel="#566">566</span>
          
            <span id="567" class="line-number" unselectable="on" rel="#567">567</span>
          
            <span id="568" class="line-number" unselectable="on" rel="#568">568</span>
          
            <span id="569" class="line-number" unselectable="on" rel="#569">569</span>
          
            <span id="570" class="line-number" unselectable="on" rel="#570">570</span>
          
            <span id="571" class="line-number" unselectable="on" rel="#571">571</span>
          
            <span id="572" class="line-number" unselectable="on" rel="#572">572</span>
          
            <span id="573" class="line-number" unselectable="on" rel="#573">573</span>
          
            <span id="574" class="line-number" unselectable="on" rel="#574">574</span>
          
            <span id="575" class="line-number" unselectable="on" rel="#575">575</span>
          
            <span id="576" class="line-number" unselectable="on" rel="#576">576</span>
          
            <span id="577" class="line-number" unselectable="on" rel="#577">577</span>
          
            <span id="578" class="line-number" unselectable="on" rel="#578">578</span>
          
            <span id="579" class="line-number" unselectable="on" rel="#579">579</span>
          
            <span id="580" class="line-number" unselectable="on" rel="#580">580</span>
          
            <span id="581" class="line-number" unselectable="on" rel="#581">581</span>
          
            <span id="582" class="line-number" unselectable="on" rel="#582">582</span>
          
            <span id="583" class="line-number" unselectable="on" rel="#583">583</span>
          
            <span id="584" class="line-number" unselectable="on" rel="#584">584</span>
          
            <span id="585" class="line-number" unselectable="on" rel="#585">585</span>
          
            <span id="586" class="line-number" unselectable="on" rel="#586">586</span>
          
            <span id="587" class="line-number" unselectable="on" rel="#587">587</span>
          
            <span id="588" class="line-number" unselectable="on" rel="#588">588</span>
          
            <span id="589" class="line-number" unselectable="on" rel="#589">589</span>
          
            <span id="590" class="line-number" unselectable="on" rel="#590">590</span>
          
            <span id="591" class="line-number" unselectable="on" rel="#591">591</span>
          
            <span id="592" class="line-number" unselectable="on" rel="#592">592</span>
          
            <span id="593" class="line-number" unselectable="on" rel="#593">593</span>
          
            <span id="594" class="line-number" unselectable="on" rel="#594">594</span>
          
        </td>
        <td class="code">
          
<pre>
<code id="line-1" aria-labelledby="1"><span class="k">import</span> WebIDL
</code><code id="line-2" aria-labelledby="2"><span class="k">import</span> traceback
</code><code id="line-3" aria-labelledby="3"><span class="k">def</span> WebIDLTest(parser, harness):
</code><code id="line-4" aria-labelledby="4">
</code><code id="line-5" aria-labelledby="5">    <span class="k">def</span> shouldPass(prefix, iface, expectedMembers, numProductions=1):
</code><code id="line-6" aria-labelledby="6">        p = parser.reset()
</code><code id="line-7" aria-labelledby="7">        p.parse(iface)
</code><code id="line-8" aria-labelledby="8">        results = p.finish()
</code><code id="line-9" aria-labelledby="9">        harness.check(len(results), numProductions,
</code><code id="line-10" aria-labelledby="10">                      <span class="str">"</span><span class="str">%s</span><span class="str"> - Should have production count </span><span class="str">%d</span><span class="str">"</span> % (prefix, numProductions))
</code><code id="line-11" aria-labelledby="11">        harness.ok(isinstance(results[0], WebIDL.IDLInterface),
</code><code id="line-12" aria-labelledby="12">                   <span class="str">"</span><span class="str">%s</span><span class="str"> - Should be an IDLInterface</span><span class="str">"</span> % (prefix))
</code><code id="line-13" aria-labelledby="13">        <span class="c"># Make a copy, since we plan to modify it</span>
</code><code id="line-14" aria-labelledby="14">        expectedMembers = list(expectedMembers)
</code><code id="line-15" aria-labelledby="15">        <span class="k">for</span> m in results[0].members:
</code><code id="line-16" aria-labelledby="16">            name = m.identifier.name
</code><code id="line-17" aria-labelledby="17">            <span class="k">if</span> (name, type(m)) in expectedMembers:
</code><code id="line-18" aria-labelledby="18">                harness.ok(True, <span class="str">"</span><span class="str">%s</span><span class="str"> - </span><span class="str">%s</span><span class="str"> - Should be a </span><span class="str">%s</span><span class="str">"</span> % (prefix, name,
</code><code id="line-19" aria-labelledby="19">                                                               type(m)))
</code><code id="line-20" aria-labelledby="20">                expectedMembers.remove((name, type(m)))
</code><code id="line-21" aria-labelledby="21">            <span class="k">else</span>:
</code><code id="line-22" aria-labelledby="22">                harness.ok(False, <span class="str">"</span><span class="str">%s</span><span class="str"> - </span><span class="str">%s</span><span class="str"> - Unknown symbol of type </span><span class="str">%s</span><span class="str">"</span> %
</code><code id="line-23" aria-labelledby="23">                           (prefix, name, type(m)))
</code><code id="line-24" aria-labelledby="24">        <span class="c"># A bit of a hoop because we can't generate the error string if we pass</span>
</code><code id="line-25" aria-labelledby="25">        <span class="k">if</span> len(expectedMembers) == 0:
</code><code id="line-26" aria-labelledby="26">            harness.ok(True, <span class="str">"</span><span class="str">Found all the members</span><span class="str">"</span>)
</code><code id="line-27" aria-labelledby="27">        <span class="k">else</span>:
</code><code id="line-28" aria-labelledby="28">            harness.ok(False,
</code><code id="line-29" aria-labelledby="29">                       <span class="str">"</span><span class="str">Expected member not found: </span><span class="str">%s</span><span class="str"> of type </span><span class="str">%s</span><span class="str">"</span> %
</code><code id="line-30" aria-labelledby="30">                       (expectedMembers[0][0], expectedMembers[0][1]))
</code><code id="line-31" aria-labelledby="31">        <span class="k">return</span> results
</code><code id="line-32" aria-labelledby="32">
</code><code id="line-33" aria-labelledby="33">    <span class="k">def</span> shouldFail(prefix, iface):
</code><code id="line-34" aria-labelledby="34">        <span class="k">try</span>:
</code><code id="line-35" aria-labelledby="35">            p = parser.reset()
</code><code id="line-36" aria-labelledby="36">            p.parse(iface)
</code><code id="line-37" aria-labelledby="37">            p.finish()
</code><code id="line-38" aria-labelledby="38">            harness.ok(False,
</code><code id="line-39" aria-labelledby="39">                       prefix + <span class="str">"</span><span class="str"> - Interface passed when should</span><span class="str">'</span><span class="str">ve failed</span><span class="str">"</span>)
</code><code id="line-40" aria-labelledby="40">        <span class="k">except</span> WebIDL.WebIDLError, e:
</code><code id="line-41" aria-labelledby="41">            harness.ok(True,
</code><code id="line-42" aria-labelledby="42">                       prefix + <span class="str">"</span><span class="str"> - Interface failed as expected</span><span class="str">"</span>)
</code><code id="line-43" aria-labelledby="43">        <span class="k">except</span> Exception, e:
</code><code id="line-44" aria-labelledby="44">            harness.ok(False,
</code><code id="line-45" aria-labelledby="45">                       prefix + <span class="str">"</span><span class="str"> - Interface failed but not as a WebIDLError exception: </span><span class="str">%s</span><span class="str">"</span> % e)
</code><code id="line-46" aria-labelledby="46">
</code><code id="line-47" aria-labelledby="47">    iterableMembers = [(x, WebIDL.IDLMethod) <span class="k">for</span> x in [<span class="str">"</span><span class="str">entries</span><span class="str">"</span>, <span class="str">"</span><span class="str">keys</span><span class="str">"</span>,
</code><code id="line-48" aria-labelledby="48">                                                       <span class="str">"</span><span class="str">values</span><span class="str">"</span>, <span class="str">"</span><span class="str">forEach</span><span class="str">"</span>]]
</code><code id="line-49" aria-labelledby="49">    setROMembers = ([(x, WebIDL.IDLMethod) <span class="k">for</span> x in [<span class="str">"</span><span class="str">has</span><span class="str">"</span>]] +
</code><code id="line-50" aria-labelledby="50">                    [(<span class="str">"</span><span class="str">__setlike</span><span class="str">"</span>, WebIDL.IDLMaplikeOrSetlike)] +
</code><code id="line-51" aria-labelledby="51">                    iterableMembers)
</code><code id="line-52" aria-labelledby="52">    setROMembers.extend([(<span class="str">"</span><span class="str">size</span><span class="str">"</span>, WebIDL.IDLAttribute)])
</code><code id="line-53" aria-labelledby="53">    setRWMembers = ([(x, WebIDL.IDLMethod) <span class="k">for</span> x in [<span class="str">"</span><span class="str">add</span><span class="str">"</span>,
</code><code id="line-54" aria-labelledby="54">                                                     <span class="str">"</span><span class="str">clear</span><span class="str">"</span>,
</code><code id="line-55" aria-labelledby="55">                                                     <span class="str">"</span><span class="str">delete</span><span class="str">"</span>]] +
</code><code id="line-56" aria-labelledby="56">                    setROMembers)
</code><code id="line-57" aria-labelledby="57">    setROChromeMembers = ([(x, WebIDL.IDLMethod) <span class="k">for</span> x in [<span class="str">"</span><span class="str">__add</span><span class="str">"</span>,
</code><code id="line-58" aria-labelledby="58">                                                           <span class="str">"</span><span class="str">__clear</span><span class="str">"</span>,
</code><code id="line-59" aria-labelledby="59">                                                           <span class="str">"</span><span class="str">__delete</span><span class="str">"</span>]] +
</code><code id="line-60" aria-labelledby="60">                          setROMembers)
</code><code id="line-61" aria-labelledby="61">    setRWChromeMembers = ([(x, WebIDL.IDLMethod) <span class="k">for</span> x in [<span class="str">"</span><span class="str">__add</span><span class="str">"</span>,
</code><code id="line-62" aria-labelledby="62">                                                           <span class="str">"</span><span class="str">__clear</span><span class="str">"</span>,
</code><code id="line-63" aria-labelledby="63">                                                           <span class="str">"</span><span class="str">__delete</span><span class="str">"</span>]] +
</code><code id="line-64" aria-labelledby="64">                          setRWMembers)
</code><code id="line-65" aria-labelledby="65">    mapROMembers = ([(x, WebIDL.IDLMethod) <span class="k">for</span> x in [<span class="str">"</span><span class="str">get</span><span class="str">"</span>, <span class="str">"</span><span class="str">has</span><span class="str">"</span>]] +
</code><code id="line-66" aria-labelledby="66">                    [(<span class="str">"</span><span class="str">__maplike</span><span class="str">"</span>, WebIDL.IDLMaplikeOrSetlike)] +
</code><code id="line-67" aria-labelledby="67">                    iterableMembers)
</code><code id="line-68" aria-labelledby="68">    mapROMembers.extend([(<span class="str">"</span><span class="str">size</span><span class="str">"</span>, WebIDL.IDLAttribute)])
</code><code id="line-69" aria-labelledby="69">    mapRWMembers = ([(x, WebIDL.IDLMethod) <span class="k">for</span> x in [<span class="str">"</span><span class="str">set</span><span class="str">"</span>,
</code><code id="line-70" aria-labelledby="70">                                                     <span class="str">"</span><span class="str">clear</span><span class="str">"</span>,
</code><code id="line-71" aria-labelledby="71">                                                     <span class="str">"</span><span class="str">delete</span><span class="str">"</span>]] + mapROMembers)
</code><code id="line-72" aria-labelledby="72">    mapRWChromeMembers = ([(x, WebIDL.IDLMethod) <span class="k">for</span> x in [<span class="str">"</span><span class="str">__set</span><span class="str">"</span>,
</code><code id="line-73" aria-labelledby="73">                                                           <span class="str">"</span><span class="str">__clear</span><span class="str">"</span>,
</code><code id="line-74" aria-labelledby="74">                                                           <span class="str">"</span><span class="str">__delete</span><span class="str">"</span>]] +
</code><code id="line-75" aria-labelledby="75">                          mapRWMembers)
</code><code id="line-76" aria-labelledby="76">
</code><code id="line-77" aria-labelledby="77">    <span class="c"># OK, now that we've used iterableMembers to set up the above, append</span>
</code><code id="line-78" aria-labelledby="78">    <span class="c"># __iterable to it for the iterable&lt;&gt; case.</span>
</code><code id="line-79" aria-labelledby="79">    iterableMembers.append((<span class="str">"</span><span class="str">__iterable</span><span class="str">"</span>, WebIDL.IDLIterable))
</code><code id="line-80" aria-labelledby="80">
</code><code id="line-81" aria-labelledby="81">    valueIterableMembers = [(<span class="str">"</span><span class="str">__iterable</span><span class="str">"</span>, WebIDL.IDLIterable)]
</code><code id="line-82" aria-labelledby="82">    valueIterableMembers.append((<span class="str">"</span><span class="str">__indexedgetter</span><span class="str">"</span>, WebIDL.IDLMethod))
</code><code id="line-83" aria-labelledby="83">    valueIterableMembers.append((<span class="str">"</span><span class="str">length</span><span class="str">"</span>, WebIDL.IDLAttribute))
</code><code id="line-84" aria-labelledby="84">
</code><code id="line-85" aria-labelledby="85">    disallowedIterableNames = [<span class="str">"</span><span class="str">keys</span><span class="str">"</span>, <span class="str">"</span><span class="str">entries</span><span class="str">"</span>, <span class="str">"</span><span class="str">values</span><span class="str">"</span>]
</code><code id="line-86" aria-labelledby="86">    disallowedMemberNames = [<span class="str">"</span><span class="str">forEach</span><span class="str">"</span>, <span class="str">"</span><span class="str">has</span><span class="str">"</span>, <span class="str">"</span><span class="str">size</span><span class="str">"</span>] + disallowedIterableNames
</code><code id="line-87" aria-labelledby="87">    mapDisallowedMemberNames = [<span class="str">"</span><span class="str">get</span><span class="str">"</span>] + disallowedMemberNames
</code><code id="line-88" aria-labelledby="88">    disallowedNonMethodNames = [<span class="str">"</span><span class="str">clear</span><span class="str">"</span>, <span class="str">"</span><span class="str">delete</span><span class="str">"</span>]
</code><code id="line-89" aria-labelledby="89">    mapDisallowedNonMethodNames = [<span class="str">"</span><span class="str">set</span><span class="str">"</span>] + disallowedNonMethodNames
</code><code id="line-90" aria-labelledby="90">    setDisallowedNonMethodNames = [<span class="str">"</span><span class="str">add</span><span class="str">"</span>] + disallowedNonMethodNames
</code><code id="line-91" aria-labelledby="91">
</code><code id="line-92" aria-labelledby="92">    <span class="c">#</span>
</code><code id="line-93" aria-labelledby="93">    <span class="c"># Simple Usage Tests</span>
</code><code id="line-94" aria-labelledby="94">    <span class="c">#</span>
</code><code id="line-95" aria-labelledby="95">
</code><code id="line-96" aria-labelledby="96">    shouldPass(<span class="str">"</span><span class="str">Iterable (key only)</span><span class="str">"</span>,
</code><code id="line-97" aria-labelledby="97">               <span class="str">"""
</span></code><code id="line-98" aria-labelledby="98"><span class="str">               interface Foo1 {
</span></code><code id="line-99" aria-labelledby="99"><span class="str">               iterable&lt;long&gt;;
</span></code><code id="line-100" aria-labelledby="100"><span class="str">               readonly attribute unsigned long length;
</span></code><code id="line-101" aria-labelledby="101"><span class="str">               getter long(unsigned long index);
</span></code><code id="line-102" aria-labelledby="102"><span class="str">               };
</span></code><code id="line-103" aria-labelledby="103"><span class="str">               """</span>, valueIterableMembers)
</code><code id="line-104" aria-labelledby="104">
</code><code id="line-105" aria-labelledby="105">    shouldPass(<span class="str">"</span><span class="str">Iterable (key and value)</span><span class="str">"</span>,
</code><code id="line-106" aria-labelledby="106">               <span class="str">"""
</span></code><code id="line-107" aria-labelledby="107"><span class="str">               interface Foo1 {
</span></code><code id="line-108" aria-labelledby="108"><span class="str">               iterable&lt;long, long&gt;;
</span></code><code id="line-109" aria-labelledby="109"><span class="str">               };
</span></code><code id="line-110" aria-labelledby="110"><span class="str">               """</span>, iterableMembers,
</code><code id="line-111" aria-labelledby="111">               <span class="c"># numProductions == 2 because of the generated iterator iface,</span>
</code><code id="line-112" aria-labelledby="112">               numProductions=2)
</code><code id="line-113" aria-labelledby="113">
</code><code id="line-114" aria-labelledby="114">    shouldPass(<span class="str">"</span><span class="str">Maplike (readwrite)</span><span class="str">"</span>,
</code><code id="line-115" aria-labelledby="115">               <span class="str">"""
</span></code><code id="line-116" aria-labelledby="116"><span class="str">               interface Foo1 {
</span></code><code id="line-117" aria-labelledby="117"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-118" aria-labelledby="118"><span class="str">               };
</span></code><code id="line-119" aria-labelledby="119"><span class="str">               """</span>, mapRWMembers)
</code><code id="line-120" aria-labelledby="120">
</code><code id="line-121" aria-labelledby="121">    shouldPass(<span class="str">"</span><span class="str">Maplike (readwrite)</span><span class="str">"</span>,
</code><code id="line-122" aria-labelledby="122">               <span class="str">"""
</span></code><code id="line-123" aria-labelledby="123"><span class="str">               interface Foo1 {
</span></code><code id="line-124" aria-labelledby="124"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-125" aria-labelledby="125"><span class="str">               };
</span></code><code id="line-126" aria-labelledby="126"><span class="str">               """</span>, mapRWMembers)
</code><code id="line-127" aria-labelledby="127">
</code><code id="line-128" aria-labelledby="128">    shouldPass(<span class="str">"</span><span class="str">Maplike (readonly)</span><span class="str">"</span>,
</code><code id="line-129" aria-labelledby="129">               <span class="str">"""
</span></code><code id="line-130" aria-labelledby="130"><span class="str">               interface Foo1 {
</span></code><code id="line-131" aria-labelledby="131"><span class="str">               readonly maplike&lt;long, long&gt;;
</span></code><code id="line-132" aria-labelledby="132"><span class="str">               };
</span></code><code id="line-133" aria-labelledby="133"><span class="str">               """</span>, mapROMembers)
</code><code id="line-134" aria-labelledby="134">
</code><code id="line-135" aria-labelledby="135">    shouldPass(<span class="str">"</span><span class="str">Setlike (readwrite)</span><span class="str">"</span>,
</code><code id="line-136" aria-labelledby="136">               <span class="str">"""
</span></code><code id="line-137" aria-labelledby="137"><span class="str">               interface Foo1 {
</span></code><code id="line-138" aria-labelledby="138"><span class="str">               setlike&lt;long&gt;;
</span></code><code id="line-139" aria-labelledby="139"><span class="str">               };
</span></code><code id="line-140" aria-labelledby="140"><span class="str">               """</span>, setRWMembers)
</code><code id="line-141" aria-labelledby="141">
</code><code id="line-142" aria-labelledby="142">    shouldPass(<span class="str">"</span><span class="str">Setlike (readonly)</span><span class="str">"</span>,
</code><code id="line-143" aria-labelledby="143">               <span class="str">"""
</span></code><code id="line-144" aria-labelledby="144"><span class="str">               interface Foo1 {
</span></code><code id="line-145" aria-labelledby="145"><span class="str">               readonly setlike&lt;long&gt;;
</span></code><code id="line-146" aria-labelledby="146"><span class="str">               };
</span></code><code id="line-147" aria-labelledby="147"><span class="str">               """</span>, setROMembers)
</code><code id="line-148" aria-labelledby="148">
</code><code id="line-149" aria-labelledby="149">    shouldPass(<span class="str">"</span><span class="str">Inheritance of maplike/setlike</span><span class="str">"</span>,
</code><code id="line-150" aria-labelledby="150">               <span class="str">"""
</span></code><code id="line-151" aria-labelledby="151"><span class="str">               interface Foo1 {
</span></code><code id="line-152" aria-labelledby="152"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-153" aria-labelledby="153"><span class="str">               };
</span></code><code id="line-154" aria-labelledby="154"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-155" aria-labelledby="155"><span class="str">               };
</span></code><code id="line-156" aria-labelledby="156"><span class="str">               """</span>, mapRWMembers, numProductions=2)
</code><code id="line-157" aria-labelledby="157">
</code><code id="line-158" aria-labelledby="158">    shouldPass(<span class="str">"</span><span class="str">Implements with maplike/setlike</span><span class="str">"</span>,
</code><code id="line-159" aria-labelledby="159">               <span class="str">"""
</span></code><code id="line-160" aria-labelledby="160"><span class="str">               interface Foo1 {
</span></code><code id="line-161" aria-labelledby="161"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-162" aria-labelledby="162"><span class="str">               };
</span></code><code id="line-163" aria-labelledby="163"><span class="str">               interface Foo2 {
</span></code><code id="line-164" aria-labelledby="164"><span class="str">               };
</span></code><code id="line-165" aria-labelledby="165"><span class="str">               Foo2 implements Foo1;
</span></code><code id="line-166" aria-labelledby="166"><span class="str">               """</span>, mapRWMembers, numProductions=3)
</code><code id="line-167" aria-labelledby="167">
</code><code id="line-168" aria-labelledby="168">    shouldPass(<span class="str">"</span><span class="str">JS Implemented maplike interface</span><span class="str">"</span>,
</code><code id="line-169" aria-labelledby="169">               <span class="str">"""
</span></code><code id="line-170" aria-labelledby="170"><span class="str">               [JSImplementation="@mozilla.org/dom/test-interface-js-maplike;1",
</span></code><code id="line-171" aria-labelledby="171"><span class="str">               Constructor()]
</span></code><code id="line-172" aria-labelledby="172"><span class="str">               interface Foo1 {
</span></code><code id="line-173" aria-labelledby="173"><span class="str">               setlike&lt;long&gt;;
</span></code><code id="line-174" aria-labelledby="174"><span class="str">               };
</span></code><code id="line-175" aria-labelledby="175"><span class="str">               """</span>, setRWChromeMembers)
</code><code id="line-176" aria-labelledby="176">
</code><code id="line-177" aria-labelledby="177">    shouldPass(<span class="str">"</span><span class="str">JS Implemented maplike interface</span><span class="str">"</span>,
</code><code id="line-178" aria-labelledby="178">               <span class="str">"""
</span></code><code id="line-179" aria-labelledby="179"><span class="str">               [JSImplementation="@mozilla.org/dom/test-interface-js-maplike;1",
</span></code><code id="line-180" aria-labelledby="180"><span class="str">               Constructor()]
</span></code><code id="line-181" aria-labelledby="181"><span class="str">               interface Foo1 {
</span></code><code id="line-182" aria-labelledby="182"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-183" aria-labelledby="183"><span class="str">               };
</span></code><code id="line-184" aria-labelledby="184"><span class="str">               """</span>, mapRWChromeMembers)
</code><code id="line-185" aria-labelledby="185">
</code><code id="line-186" aria-labelledby="186">    <span class="c">#</span>
</code><code id="line-187" aria-labelledby="187">    <span class="c"># Multiple maplike/setlike tests</span>
</code><code id="line-188" aria-labelledby="188">    <span class="c">#</span>
</code><code id="line-189" aria-labelledby="189">
</code><code id="line-190" aria-labelledby="190">    shouldFail(<span class="str">"</span><span class="str">Two maplike/setlikes on same interface</span><span class="str">"</span>,
</code><code id="line-191" aria-labelledby="191">               <span class="str">"""
</span></code><code id="line-192" aria-labelledby="192"><span class="str">               interface Foo1 {
</span></code><code id="line-193" aria-labelledby="193"><span class="str">               setlike&lt;long&gt;;
</span></code><code id="line-194" aria-labelledby="194"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-195" aria-labelledby="195"><span class="str">               };
</span></code><code id="line-196" aria-labelledby="196"><span class="str">               """</span>)
</code><code id="line-197" aria-labelledby="197">
</code><code id="line-198" aria-labelledby="198">    shouldFail(<span class="str">"</span><span class="str">Two iterable/setlikes on same interface</span><span class="str">"</span>,
</code><code id="line-199" aria-labelledby="199">               <span class="str">"""
</span></code><code id="line-200" aria-labelledby="200"><span class="str">               interface Foo1 {
</span></code><code id="line-201" aria-labelledby="201"><span class="str">               iterable&lt;long&gt;;
</span></code><code id="line-202" aria-labelledby="202"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-203" aria-labelledby="203"><span class="str">               };
</span></code><code id="line-204" aria-labelledby="204"><span class="str">               """</span>)
</code><code id="line-205" aria-labelledby="205">
</code><code id="line-206" aria-labelledby="206">    shouldFail(<span class="str">"</span><span class="str">Two iterables on same interface</span><span class="str">"</span>,
</code><code id="line-207" aria-labelledby="207">               <span class="str">"""
</span></code><code id="line-208" aria-labelledby="208"><span class="str">               interface Foo1 {
</span></code><code id="line-209" aria-labelledby="209"><span class="str">               iterable&lt;long&gt;;
</span></code><code id="line-210" aria-labelledby="210"><span class="str">               iterable&lt;long, long&gt;;
</span></code><code id="line-211" aria-labelledby="211"><span class="str">               };
</span></code><code id="line-212" aria-labelledby="212"><span class="str">               """</span>)
</code><code id="line-213" aria-labelledby="213">
</code><code id="line-214" aria-labelledby="214">    shouldFail(<span class="str">"</span><span class="str">Two maplike/setlikes in partials</span><span class="str">"</span>,
</code><code id="line-215" aria-labelledby="215">               <span class="str">"""
</span></code><code id="line-216" aria-labelledby="216"><span class="str">               interface Foo1 {
</span></code><code id="line-217" aria-labelledby="217"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-218" aria-labelledby="218"><span class="str">               };
</span></code><code id="line-219" aria-labelledby="219"><span class="str">               partial interface Foo1 {
</span></code><code id="line-220" aria-labelledby="220"><span class="str">               setlike&lt;long&gt;;
</span></code><code id="line-221" aria-labelledby="221"><span class="str">               };
</span></code><code id="line-222" aria-labelledby="222"><span class="str">               """</span>)
</code><code id="line-223" aria-labelledby="223">
</code><code id="line-224" aria-labelledby="224">    shouldFail(<span class="str">"</span><span class="str">Conflicting maplike/setlikes across inheritance</span><span class="str">"</span>,
</code><code id="line-225" aria-labelledby="225">               <span class="str">"""
</span></code><code id="line-226" aria-labelledby="226"><span class="str">               interface Foo1 {
</span></code><code id="line-227" aria-labelledby="227"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-228" aria-labelledby="228"><span class="str">               };
</span></code><code id="line-229" aria-labelledby="229"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-230" aria-labelledby="230"><span class="str">               setlike&lt;long&gt;;
</span></code><code id="line-231" aria-labelledby="231"><span class="str">               };
</span></code><code id="line-232" aria-labelledby="232"><span class="str">               """</span>)
</code><code id="line-233" aria-labelledby="233">
</code><code id="line-234" aria-labelledby="234">    shouldFail(<span class="str">"</span><span class="str">Conflicting maplike/iterable across inheritance</span><span class="str">"</span>,
</code><code id="line-235" aria-labelledby="235">               <span class="str">"""
</span></code><code id="line-236" aria-labelledby="236"><span class="str">               interface Foo1 {
</span></code><code id="line-237" aria-labelledby="237"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-238" aria-labelledby="238"><span class="str">               };
</span></code><code id="line-239" aria-labelledby="239"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-240" aria-labelledby="240"><span class="str">               iterable&lt;long&gt;;
</span></code><code id="line-241" aria-labelledby="241"><span class="str">               };
</span></code><code id="line-242" aria-labelledby="242"><span class="str">               """</span>)
</code><code id="line-243" aria-labelledby="243">
</code><code id="line-244" aria-labelledby="244">    shouldFail(<span class="str">"</span><span class="str">Conflicting maplike/setlikes across multistep inheritance</span><span class="str">"</span>,
</code><code id="line-245" aria-labelledby="245">               <span class="str">"""
</span></code><code id="line-246" aria-labelledby="246"><span class="str">               interface Foo1 {
</span></code><code id="line-247" aria-labelledby="247"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-248" aria-labelledby="248"><span class="str">               };
</span></code><code id="line-249" aria-labelledby="249"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-250" aria-labelledby="250"><span class="str">               };
</span></code><code id="line-251" aria-labelledby="251"><span class="str">               interface Foo3 : Foo2 {
</span></code><code id="line-252" aria-labelledby="252"><span class="str">               setlike&lt;long&gt;;
</span></code><code id="line-253" aria-labelledby="253"><span class="str">               };
</span></code><code id="line-254" aria-labelledby="254"><span class="str">               """</span>)
</code><code id="line-255" aria-labelledby="255">
</code><code id="line-256" aria-labelledby="256">    shouldFail(<span class="str">"</span><span class="str">Consequential interface with conflicting maplike/setlike</span><span class="str">"</span>,
</code><code id="line-257" aria-labelledby="257">               <span class="str">"""
</span></code><code id="line-258" aria-labelledby="258"><span class="str">               interface Foo1 {
</span></code><code id="line-259" aria-labelledby="259"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-260" aria-labelledby="260"><span class="str">               };
</span></code><code id="line-261" aria-labelledby="261"><span class="str">               interface Foo2 {
</span></code><code id="line-262" aria-labelledby="262"><span class="str">               setlike&lt;long&gt;;
</span></code><code id="line-263" aria-labelledby="263"><span class="str">               };
</span></code><code id="line-264" aria-labelledby="264"><span class="str">               Foo2 implements Foo1;
</span></code><code id="line-265" aria-labelledby="265"><span class="str">               """</span>)
</code><code id="line-266" aria-labelledby="266">
</code><code id="line-267" aria-labelledby="267">    shouldFail(<span class="str">"</span><span class="str">Consequential interfaces with conflicting maplike/setlike</span><span class="str">"</span>,
</code><code id="line-268" aria-labelledby="268">               <span class="str">"""
</span></code><code id="line-269" aria-labelledby="269"><span class="str">               interface Foo1 {
</span></code><code id="line-270" aria-labelledby="270"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-271" aria-labelledby="271"><span class="str">               };
</span></code><code id="line-272" aria-labelledby="272"><span class="str">               interface Foo2 {
</span></code><code id="line-273" aria-labelledby="273"><span class="str">               setlike&lt;long&gt;;
</span></code><code id="line-274" aria-labelledby="274"><span class="str">               };
</span></code><code id="line-275" aria-labelledby="275"><span class="str">               interface Foo3 {
</span></code><code id="line-276" aria-labelledby="276"><span class="str">               };
</span></code><code id="line-277" aria-labelledby="277"><span class="str">               Foo3 implements Foo1;
</span></code><code id="line-278" aria-labelledby="278"><span class="str">               Foo3 implements Foo2;
</span></code><code id="line-279" aria-labelledby="279"><span class="str">               """</span>)
</code><code id="line-280" aria-labelledby="280">
</code><code id="line-281" aria-labelledby="281">    <span class="c">#</span>
</code><code id="line-282" aria-labelledby="282">    <span class="c"># Member name collision tests</span>
</code><code id="line-283" aria-labelledby="283">    <span class="c">#</span>
</code><code id="line-284" aria-labelledby="284">
</code><code id="line-285" aria-labelledby="285">    <span class="k">def</span> testConflictingMembers(likeMember, conflictName, expectedMembers, methodPasses):
</code><code id="line-286" aria-labelledby="286">        <span class="str">"""
</span></code><code id="line-287" aria-labelledby="287"><span class="str">        Tests for maplike/setlike member generation against conflicting member
</span></code><code id="line-288" aria-labelledby="288"><span class="str">        names. If methodPasses is True, this means we expect the interface to
</span></code><code id="line-289" aria-labelledby="289"><span class="str">        pass in the case of method shadowing, and expectedMembers should be the
</span></code><code id="line-290" aria-labelledby="290"><span class="str">        list of interface members to check against on the passing interface.
</span></code><code id="line-291" aria-labelledby="291"><span class="str">
</span></code><code id="line-292" aria-labelledby="292"><span class="str">        """</span>
</code><code id="line-293" aria-labelledby="293">        <span class="k">if</span> methodPasses:
</code><code id="line-294" aria-labelledby="294">            shouldPass(<span class="str">"</span><span class="str">Conflicting method: </span><span class="str">%s</span><span class="str"> and </span><span class="str">%s</span><span class="str">"</span> % (likeMember, conflictName),
</code><code id="line-295" aria-labelledby="295">                       <span class="str">"""
</span></code><code id="line-296" aria-labelledby="296"><span class="str">                       interface Foo1 {
</span></code><code id="line-297" aria-labelledby="297"><span class="str">                       %s;
</span></code><code id="line-298" aria-labelledby="298"><span class="str">                       [Throws]
</span></code><code id="line-299" aria-labelledby="299"><span class="str">                       void %s(long test1, double test2, double test3);
</span></code><code id="line-300" aria-labelledby="300"><span class="str">                       };
</span></code><code id="line-301" aria-labelledby="301"><span class="str">                       """</span> % (likeMember, conflictName), expectedMembers)
</code><code id="line-302" aria-labelledby="302">        <span class="k">else</span>:
</code><code id="line-303" aria-labelledby="303">            shouldFail(<span class="str">"</span><span class="str">Conflicting method: </span><span class="str">%s</span><span class="str"> and </span><span class="str">%s</span><span class="str">"</span> % (likeMember, conflictName),
</code><code id="line-304" aria-labelledby="304">                       <span class="str">"""
</span></code><code id="line-305" aria-labelledby="305"><span class="str">                       interface Foo1 {
</span></code><code id="line-306" aria-labelledby="306"><span class="str">                       %s;
</span></code><code id="line-307" aria-labelledby="307"><span class="str">                       [Throws]
</span></code><code id="line-308" aria-labelledby="308"><span class="str">                       void %s(long test1, double test2, double test3);
</span></code><code id="line-309" aria-labelledby="309"><span class="str">                       };
</span></code><code id="line-310" aria-labelledby="310"><span class="str">                       """</span> % (likeMember, conflictName))
</code><code id="line-311" aria-labelledby="311">        <span class="c"># Inherited conflicting methods should ALWAYS fail</span>
</code><code id="line-312" aria-labelledby="312">        shouldFail(<span class="str">"</span><span class="str">Conflicting inherited method: </span><span class="str">%s</span><span class="str"> and </span><span class="str">%s</span><span class="str">"</span> % (likeMember, conflictName),
</code><code id="line-313" aria-labelledby="313">                   <span class="str">"""
</span></code><code id="line-314" aria-labelledby="314"><span class="str">                   interface Foo1 {
</span></code><code id="line-315" aria-labelledby="315"><span class="str">                   void %s(long test1, double test2, double test3);
</span></code><code id="line-316" aria-labelledby="316"><span class="str">                   };
</span></code><code id="line-317" aria-labelledby="317"><span class="str">                   interface Foo2 : Foo1 {
</span></code><code id="line-318" aria-labelledby="318"><span class="str">                   %s;
</span></code><code id="line-319" aria-labelledby="319"><span class="str">                   };
</span></code><code id="line-320" aria-labelledby="320"><span class="str">                   """</span> % (conflictName, likeMember))
</code><code id="line-321" aria-labelledby="321">        shouldFail(<span class="str">"</span><span class="str">Conflicting static method: </span><span class="str">%s</span><span class="str"> and </span><span class="str">%s</span><span class="str">"</span> % (likeMember, conflictName),
</code><code id="line-322" aria-labelledby="322">                   <span class="str">"""
</span></code><code id="line-323" aria-labelledby="323"><span class="str">                   interface Foo1 {
</span></code><code id="line-324" aria-labelledby="324"><span class="str">                   %s;
</span></code><code id="line-325" aria-labelledby="325"><span class="str">                   static void %s(long test1, double test2, double test3);
</span></code><code id="line-326" aria-labelledby="326"><span class="str">                   };
</span></code><code id="line-327" aria-labelledby="327"><span class="str">                   """</span> % (likeMember, conflictName))
</code><code id="line-328" aria-labelledby="328">        shouldFail(<span class="str">"</span><span class="str">Conflicting attribute: </span><span class="str">%s</span><span class="str"> and </span><span class="str">%s</span><span class="str">"</span> % (likeMember, conflictName),
</code><code id="line-329" aria-labelledby="329">                   <span class="str">"""
</span></code><code id="line-330" aria-labelledby="330"><span class="str">                   interface Foo1 {
</span></code><code id="line-331" aria-labelledby="331"><span class="str">                   %s
</span></code><code id="line-332" aria-labelledby="332"><span class="str">                   attribute double %s;
</span></code><code id="line-333" aria-labelledby="333"><span class="str">                   };
</span></code><code id="line-334" aria-labelledby="334"><span class="str">                   """</span> % (likeMember, conflictName))
</code><code id="line-335" aria-labelledby="335">        shouldFail(<span class="str">"</span><span class="str">Conflicting const: </span><span class="str">%s</span><span class="str"> and </span><span class="str">%s</span><span class="str">"</span> % (likeMember, conflictName),
</code><code id="line-336" aria-labelledby="336">                   <span class="str">"""
</span></code><code id="line-337" aria-labelledby="337"><span class="str">                   interface Foo1 {
</span></code><code id="line-338" aria-labelledby="338"><span class="str">                   %s;
</span></code><code id="line-339" aria-labelledby="339"><span class="str">                   const double %s = 0;
</span></code><code id="line-340" aria-labelledby="340"><span class="str">                   };
</span></code><code id="line-341" aria-labelledby="341"><span class="str">                   """</span> % (likeMember, conflictName))
</code><code id="line-342" aria-labelledby="342">        shouldFail(<span class="str">"</span><span class="str">Conflicting static attribute: </span><span class="str">%s</span><span class="str"> and </span><span class="str">%s</span><span class="str">"</span> % (likeMember, conflictName),
</code><code id="line-343" aria-labelledby="343">                   <span class="str">"""
</span></code><code id="line-344" aria-labelledby="344"><span class="str">                   interface Foo1 {
</span></code><code id="line-345" aria-labelledby="345"><span class="str">                   %s;
</span></code><code id="line-346" aria-labelledby="346"><span class="str">                   static attribute long %s;
</span></code><code id="line-347" aria-labelledby="347"><span class="str">                   };
</span></code><code id="line-348" aria-labelledby="348"><span class="str">                   """</span> % (likeMember, conflictName))
</code><code id="line-349" aria-labelledby="349">
</code><code id="line-350" aria-labelledby="350">    <span class="k">for</span> member in disallowedIterableNames:
</code><code id="line-351" aria-labelledby="351">        testConflictingMembers(<span class="str">"</span><span class="str">iterable&lt;long, long&gt;</span><span class="str">"</span>, member, iterableMembers, False)
</code><code id="line-352" aria-labelledby="352">    <span class="k">for</span> member in mapDisallowedMemberNames:
</code><code id="line-353" aria-labelledby="353">        testConflictingMembers(<span class="str">"</span><span class="str">maplike&lt;long, long&gt;</span><span class="str">"</span>, member, mapRWMembers, False)
</code><code id="line-354" aria-labelledby="354">    <span class="k">for</span> member in disallowedMemberNames:
</code><code id="line-355" aria-labelledby="355">        testConflictingMembers(<span class="str">"</span><span class="str">setlike&lt;long&gt;</span><span class="str">"</span>, member, setRWMembers, False)
</code><code id="line-356" aria-labelledby="356">    <span class="k">for</span> member in mapDisallowedNonMethodNames:
</code><code id="line-357" aria-labelledby="357">        testConflictingMembers(<span class="str">"</span><span class="str">maplike&lt;long, long&gt;</span><span class="str">"</span>, member, mapRWMembers, True)
</code><code id="line-358" aria-labelledby="358">    <span class="k">for</span> member in setDisallowedNonMethodNames:
</code><code id="line-359" aria-labelledby="359">        testConflictingMembers(<span class="str">"</span><span class="str">setlike&lt;long&gt;</span><span class="str">"</span>, member, setRWMembers, True)
</code><code id="line-360" aria-labelledby="360">
</code><code id="line-361" aria-labelledby="361">    shouldPass(<span class="str">"</span><span class="str">Inheritance of maplike/setlike with child member collision</span><span class="str">"</span>,
</code><code id="line-362" aria-labelledby="362">               <span class="str">"""
</span></code><code id="line-363" aria-labelledby="363"><span class="str">               interface Foo1 {
</span></code><code id="line-364" aria-labelledby="364"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-365" aria-labelledby="365"><span class="str">               };
</span></code><code id="line-366" aria-labelledby="366"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-367" aria-labelledby="367"><span class="str">               void entries();
</span></code><code id="line-368" aria-labelledby="368"><span class="str">               };
</span></code><code id="line-369" aria-labelledby="369"><span class="str">               """</span>, mapRWMembers, numProductions=2)
</code><code id="line-370" aria-labelledby="370">
</code><code id="line-371" aria-labelledby="371">    shouldPass(<span class="str">"</span><span class="str">Inheritance of multi-level maplike/setlike with child member collision</span><span class="str">"</span>,
</code><code id="line-372" aria-labelledby="372">               <span class="str">"""
</span></code><code id="line-373" aria-labelledby="373"><span class="str">               interface Foo1 {
</span></code><code id="line-374" aria-labelledby="374"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-375" aria-labelledby="375"><span class="str">               };
</span></code><code id="line-376" aria-labelledby="376"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-377" aria-labelledby="377"><span class="str">               };
</span></code><code id="line-378" aria-labelledby="378"><span class="str">               interface Foo3 : Foo2 {
</span></code><code id="line-379" aria-labelledby="379"><span class="str">               void entries();
</span></code><code id="line-380" aria-labelledby="380"><span class="str">               };
</span></code><code id="line-381" aria-labelledby="381"><span class="str">               """</span>, mapRWMembers, numProductions=3)
</code><code id="line-382" aria-labelledby="382">
</code><code id="line-383" aria-labelledby="383">    shouldFail(<span class="str">"</span><span class="str">Interface with consequential maplike/setlike interface member collision</span><span class="str">"</span>,
</code><code id="line-384" aria-labelledby="384">               <span class="str">"""
</span></code><code id="line-385" aria-labelledby="385"><span class="str">               interface Foo1 {
</span></code><code id="line-386" aria-labelledby="386"><span class="str">               void entries();
</span></code><code id="line-387" aria-labelledby="387"><span class="str">               };
</span></code><code id="line-388" aria-labelledby="388"><span class="str">               interface Foo2 {
</span></code><code id="line-389" aria-labelledby="389"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-390" aria-labelledby="390"><span class="str">               };
</span></code><code id="line-391" aria-labelledby="391"><span class="str">               Foo1 implements Foo2;
</span></code><code id="line-392" aria-labelledby="392"><span class="str">               """</span>)
</code><code id="line-393" aria-labelledby="393">
</code><code id="line-394" aria-labelledby="394">    shouldFail(<span class="str">"</span><span class="str">Maplike interface with consequential interface member collision</span><span class="str">"</span>,
</code><code id="line-395" aria-labelledby="395">               <span class="str">"""
</span></code><code id="line-396" aria-labelledby="396"><span class="str">               interface Foo1 {
</span></code><code id="line-397" aria-labelledby="397"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-398" aria-labelledby="398"><span class="str">               };
</span></code><code id="line-399" aria-labelledby="399"><span class="str">               interface Foo2 {
</span></code><code id="line-400" aria-labelledby="400"><span class="str">               void entries();
</span></code><code id="line-401" aria-labelledby="401"><span class="str">               };
</span></code><code id="line-402" aria-labelledby="402"><span class="str">               Foo1 implements Foo2;
</span></code><code id="line-403" aria-labelledby="403"><span class="str">               """</span>)
</code><code id="line-404" aria-labelledby="404">
</code><code id="line-405" aria-labelledby="405">    shouldPass(<span class="str">"</span><span class="str">Consequential Maplike interface with inherited interface member collision</span><span class="str">"</span>,
</code><code id="line-406" aria-labelledby="406">               <span class="str">"""
</span></code><code id="line-407" aria-labelledby="407"><span class="str">               interface Foo1 {
</span></code><code id="line-408" aria-labelledby="408"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-409" aria-labelledby="409"><span class="str">               };
</span></code><code id="line-410" aria-labelledby="410"><span class="str">               interface Foo2 {
</span></code><code id="line-411" aria-labelledby="411"><span class="str">               void entries();
</span></code><code id="line-412" aria-labelledby="412"><span class="str">               };
</span></code><code id="line-413" aria-labelledby="413"><span class="str">               interface Foo3 : Foo2 {
</span></code><code id="line-414" aria-labelledby="414"><span class="str">               };
</span></code><code id="line-415" aria-labelledby="415"><span class="str">               Foo3 implements Foo1;
</span></code><code id="line-416" aria-labelledby="416"><span class="str">               """</span>, mapRWMembers, numProductions=4)
</code><code id="line-417" aria-labelledby="417">
</code><code id="line-418" aria-labelledby="418">    shouldPass(<span class="str">"</span><span class="str">Inherited Maplike interface with consequential interface member collision</span><span class="str">"</span>,
</code><code id="line-419" aria-labelledby="419">               <span class="str">"""
</span></code><code id="line-420" aria-labelledby="420"><span class="str">               interface Foo1 {
</span></code><code id="line-421" aria-labelledby="421"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-422" aria-labelledby="422"><span class="str">               };
</span></code><code id="line-423" aria-labelledby="423"><span class="str">               interface Foo2 {
</span></code><code id="line-424" aria-labelledby="424"><span class="str">               void entries();
</span></code><code id="line-425" aria-labelledby="425"><span class="str">               };
</span></code><code id="line-426" aria-labelledby="426"><span class="str">               interface Foo3 : Foo1 {
</span></code><code id="line-427" aria-labelledby="427"><span class="str">               };
</span></code><code id="line-428" aria-labelledby="428"><span class="str">               Foo3 implements Foo2;
</span></code><code id="line-429" aria-labelledby="429"><span class="str">               """</span>, mapRWMembers, numProductions=4)
</code><code id="line-430" aria-labelledby="430">
</code><code id="line-431" aria-labelledby="431">    shouldFail(<span class="str">"</span><span class="str">Inheritance of name collision with child maplike/setlike</span><span class="str">"</span>,
</code><code id="line-432" aria-labelledby="432">               <span class="str">"""
</span></code><code id="line-433" aria-labelledby="433"><span class="str">               interface Foo1 {
</span></code><code id="line-434" aria-labelledby="434"><span class="str">               void entries();
</span></code><code id="line-435" aria-labelledby="435"><span class="str">               };
</span></code><code id="line-436" aria-labelledby="436"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-437" aria-labelledby="437"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-438" aria-labelledby="438"><span class="str">               };
</span></code><code id="line-439" aria-labelledby="439"><span class="str">               """</span>)
</code><code id="line-440" aria-labelledby="440">
</code><code id="line-441" aria-labelledby="441">    shouldFail(<span class="str">"</span><span class="str">Inheritance of multi-level name collision with child maplike/setlike</span><span class="str">"</span>,
</code><code id="line-442" aria-labelledby="442">               <span class="str">"""
</span></code><code id="line-443" aria-labelledby="443"><span class="str">               interface Foo1 {
</span></code><code id="line-444" aria-labelledby="444"><span class="str">               void entries();
</span></code><code id="line-445" aria-labelledby="445"><span class="str">               };
</span></code><code id="line-446" aria-labelledby="446"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-447" aria-labelledby="447"><span class="str">               };
</span></code><code id="line-448" aria-labelledby="448"><span class="str">               interface Foo3 : Foo2 {
</span></code><code id="line-449" aria-labelledby="449"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-450" aria-labelledby="450"><span class="str">               };
</span></code><code id="line-451" aria-labelledby="451"><span class="str">               """</span>)
</code><code id="line-452" aria-labelledby="452">
</code><code id="line-453" aria-labelledby="453">    shouldPass(<span class="str">"</span><span class="str">Inheritance of attribute collision with parent maplike/setlike</span><span class="str">"</span>,
</code><code id="line-454" aria-labelledby="454">               <span class="str">"""
</span></code><code id="line-455" aria-labelledby="455"><span class="str">               interface Foo1 {
</span></code><code id="line-456" aria-labelledby="456"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-457" aria-labelledby="457"><span class="str">               };
</span></code><code id="line-458" aria-labelledby="458"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-459" aria-labelledby="459"><span class="str">               attribute double size;
</span></code><code id="line-460" aria-labelledby="460"><span class="str">               };
</span></code><code id="line-461" aria-labelledby="461"><span class="str">               """</span>, mapRWMembers, numProductions=2)
</code><code id="line-462" aria-labelledby="462">
</code><code id="line-463" aria-labelledby="463">    shouldPass(<span class="str">"</span><span class="str">Inheritance of multi-level attribute collision with parent maplike/setlike</span><span class="str">"</span>,
</code><code id="line-464" aria-labelledby="464">               <span class="str">"""
</span></code><code id="line-465" aria-labelledby="465"><span class="str">               interface Foo1 {
</span></code><code id="line-466" aria-labelledby="466"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-467" aria-labelledby="467"><span class="str">               };
</span></code><code id="line-468" aria-labelledby="468"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-469" aria-labelledby="469"><span class="str">               };
</span></code><code id="line-470" aria-labelledby="470"><span class="str">               interface Foo3 : Foo2 {
</span></code><code id="line-471" aria-labelledby="471"><span class="str">               attribute double size;
</span></code><code id="line-472" aria-labelledby="472"><span class="str">               };
</span></code><code id="line-473" aria-labelledby="473"><span class="str">               """</span>, mapRWMembers, numProductions=3)
</code><code id="line-474" aria-labelledby="474">
</code><code id="line-475" aria-labelledby="475">    shouldFail(<span class="str">"</span><span class="str">Inheritance of attribute collision with child maplike/setlike</span><span class="str">"</span>,
</code><code id="line-476" aria-labelledby="476">               <span class="str">"""
</span></code><code id="line-477" aria-labelledby="477"><span class="str">               interface Foo1 {
</span></code><code id="line-478" aria-labelledby="478"><span class="str">               attribute double size;
</span></code><code id="line-479" aria-labelledby="479"><span class="str">               };
</span></code><code id="line-480" aria-labelledby="480"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-481" aria-labelledby="481"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-482" aria-labelledby="482"><span class="str">               };
</span></code><code id="line-483" aria-labelledby="483"><span class="str">               """</span>)
</code><code id="line-484" aria-labelledby="484">
</code><code id="line-485" aria-labelledby="485">    shouldFail(<span class="str">"</span><span class="str">Inheritance of multi-level attribute collision with child maplike/setlike</span><span class="str">"</span>,
</code><code id="line-486" aria-labelledby="486">               <span class="str">"""
</span></code><code id="line-487" aria-labelledby="487"><span class="str">               interface Foo1 {
</span></code><code id="line-488" aria-labelledby="488"><span class="str">               attribute double size;
</span></code><code id="line-489" aria-labelledby="489"><span class="str">               };
</span></code><code id="line-490" aria-labelledby="490"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-491" aria-labelledby="491"><span class="str">               };
</span></code><code id="line-492" aria-labelledby="492"><span class="str">               interface Foo3 : Foo2 {
</span></code><code id="line-493" aria-labelledby="493"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-494" aria-labelledby="494"><span class="str">               };
</span></code><code id="line-495" aria-labelledby="495"><span class="str">               """</span>)
</code><code id="line-496" aria-labelledby="496">
</code><code id="line-497" aria-labelledby="497">    shouldFail(<span class="str">"</span><span class="str">Inheritance of attribute/rw function collision with child maplike/setlike</span><span class="str">"</span>,
</code><code id="line-498" aria-labelledby="498">               <span class="str">"""
</span></code><code id="line-499" aria-labelledby="499"><span class="str">               interface Foo1 {
</span></code><code id="line-500" aria-labelledby="500"><span class="str">               attribute double set;
</span></code><code id="line-501" aria-labelledby="501"><span class="str">               };
</span></code><code id="line-502" aria-labelledby="502"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-503" aria-labelledby="503"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-504" aria-labelledby="504"><span class="str">               };
</span></code><code id="line-505" aria-labelledby="505"><span class="str">               """</span>)
</code><code id="line-506" aria-labelledby="506">
</code><code id="line-507" aria-labelledby="507">    shouldFail(<span class="str">"</span><span class="str">Inheritance of const/rw function collision with child maplike/setlike</span><span class="str">"</span>,
</code><code id="line-508" aria-labelledby="508">               <span class="str">"""
</span></code><code id="line-509" aria-labelledby="509"><span class="str">               interface Foo1 {
</span></code><code id="line-510" aria-labelledby="510"><span class="str">               const double set = 0;
</span></code><code id="line-511" aria-labelledby="511"><span class="str">               };
</span></code><code id="line-512" aria-labelledby="512"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-513" aria-labelledby="513"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-514" aria-labelledby="514"><span class="str">               };
</span></code><code id="line-515" aria-labelledby="515"><span class="str">               """</span>)
</code><code id="line-516" aria-labelledby="516">
</code><code id="line-517" aria-labelledby="517">    shouldPass(<span class="str">"</span><span class="str">Inheritance of rw function with same name in child maplike/setlike</span><span class="str">"</span>,
</code><code id="line-518" aria-labelledby="518">               <span class="str">"""
</span></code><code id="line-519" aria-labelledby="519"><span class="str">               interface Foo1 {
</span></code><code id="line-520" aria-labelledby="520"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-521" aria-labelledby="521"><span class="str">               };
</span></code><code id="line-522" aria-labelledby="522"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-523" aria-labelledby="523"><span class="str">               void clear();
</span></code><code id="line-524" aria-labelledby="524"><span class="str">               };
</span></code><code id="line-525" aria-labelledby="525"><span class="str">               """</span>, mapRWMembers, numProductions=2)
</code><code id="line-526" aria-labelledby="526">
</code><code id="line-527" aria-labelledby="527">    shouldFail(<span class="str">"</span><span class="str">Inheritance of unforgeable attribute collision with child maplike/setlike</span><span class="str">"</span>,
</code><code id="line-528" aria-labelledby="528">               <span class="str">"""
</span></code><code id="line-529" aria-labelledby="529"><span class="str">               interface Foo1 {
</span></code><code id="line-530" aria-labelledby="530"><span class="str">               [Unforgeable]
</span></code><code id="line-531" aria-labelledby="531"><span class="str">               attribute double size;
</span></code><code id="line-532" aria-labelledby="532"><span class="str">               };
</span></code><code id="line-533" aria-labelledby="533"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-534" aria-labelledby="534"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-535" aria-labelledby="535"><span class="str">               };
</span></code><code id="line-536" aria-labelledby="536"><span class="str">               """</span>)
</code><code id="line-537" aria-labelledby="537">
</code><code id="line-538" aria-labelledby="538">    shouldFail(<span class="str">"</span><span class="str">Inheritance of multi-level unforgeable attribute collision with child maplike/setlike</span><span class="str">"</span>,
</code><code id="line-539" aria-labelledby="539">               <span class="str">"""
</span></code><code id="line-540" aria-labelledby="540"><span class="str">               interface Foo1 {
</span></code><code id="line-541" aria-labelledby="541"><span class="str">               [Unforgeable]
</span></code><code id="line-542" aria-labelledby="542"><span class="str">               attribute double size;
</span></code><code id="line-543" aria-labelledby="543"><span class="str">               };
</span></code><code id="line-544" aria-labelledby="544"><span class="str">               interface Foo2 : Foo1 {
</span></code><code id="line-545" aria-labelledby="545"><span class="str">               };
</span></code><code id="line-546" aria-labelledby="546"><span class="str">               interface Foo3 : Foo2 {
</span></code><code id="line-547" aria-labelledby="547"><span class="str">               maplike&lt;long, long&gt;;
</span></code><code id="line-548" aria-labelledby="548"><span class="str">               };
</span></code><code id="line-549" aria-labelledby="549"><span class="str">               """</span>)
</code><code id="line-550" aria-labelledby="550">
</code><code id="line-551" aria-labelledby="551">    shouldPass(<span class="str">"</span><span class="str">Implemented interface with readonly allowable overrides</span><span class="str">"</span>,
</code><code id="line-552" aria-labelledby="552">               <span class="str">"""
</span></code><code id="line-553" aria-labelledby="553"><span class="str">               interface Foo1 {
</span></code><code id="line-554" aria-labelledby="554"><span class="str">               readonly setlike&lt;long&gt;;
</span></code><code id="line-555" aria-labelledby="555"><span class="str">               readonly attribute boolean clear;
</span></code><code id="line-556" aria-labelledby="556"><span class="str">               };
</span></code><code id="line-557" aria-labelledby="557"><span class="str">               """</span>, setROMembers + [(<span class="str">"</span><span class="str">clear</span><span class="str">"</span>, WebIDL.IDLAttribute)])
</code><code id="line-558" aria-labelledby="558">
</code><code id="line-559" aria-labelledby="559">    shouldPass(<span class="str">"</span><span class="str">JS Implemented read-only interface with readonly allowable overrides</span><span class="str">"</span>,
</code><code id="line-560" aria-labelledby="560">               <span class="str">"""
</span></code><code id="line-561" aria-labelledby="561"><span class="str">               [JSImplementation="@mozilla.org/dom/test-interface-js-maplike;1",
</span></code><code id="line-562" aria-labelledby="562"><span class="str">               Constructor()]
</span></code><code id="line-563" aria-labelledby="563"><span class="str">               interface Foo1 {
</span></code><code id="line-564" aria-labelledby="564"><span class="str">               readonly setlike&lt;long&gt;;
</span></code><code id="line-565" aria-labelledby="565"><span class="str">               readonly attribute boolean clear;
</span></code><code id="line-566" aria-labelledby="566"><span class="str">               };
</span></code><code id="line-567" aria-labelledby="567"><span class="str">               """</span>, setROChromeMembers + [(<span class="str">"</span><span class="str">clear</span><span class="str">"</span>, WebIDL.IDLAttribute)])
</code><code id="line-568" aria-labelledby="568">
</code><code id="line-569" aria-labelledby="569">    shouldFail(<span class="str">"</span><span class="str">JS Implemented read-write interface with non-readwrite allowable overrides</span><span class="str">"</span>,
</code><code id="line-570" aria-labelledby="570">               <span class="str">"""
</span></code><code id="line-571" aria-labelledby="571"><span class="str">               [JSImplementation="@mozilla.org/dom/test-interface-js-maplike;1",
</span></code><code id="line-572" aria-labelledby="572"><span class="str">               Constructor()]
</span></code><code id="line-573" aria-labelledby="573"><span class="str">               interface Foo1 {
</span></code><code id="line-574" aria-labelledby="574"><span class="str">               setlike&lt;long&gt;;
</span></code><code id="line-575" aria-labelledby="575"><span class="str">               readonly attribute boolean clear;
</span></code><code id="line-576" aria-labelledby="576"><span class="str">               };
</span></code><code id="line-577" aria-labelledby="577"><span class="str">               """</span>)
</code><code id="line-578" aria-labelledby="578">
</code><code id="line-579" aria-labelledby="579">    r = shouldPass(<span class="str">"</span><span class="str">Check proper override of clear/delete/set</span><span class="str">"</span>,
</code><code id="line-580" aria-labelledby="580">                   <span class="str">"""
</span></code><code id="line-581" aria-labelledby="581"><span class="str">                   interface Foo1 {
</span></code><code id="line-582" aria-labelledby="582"><span class="str">                   maplike&lt;long, long&gt;;
</span></code><code id="line-583" aria-labelledby="583"><span class="str">                   long clear(long a, long b, double c, double d);
</span></code><code id="line-584" aria-labelledby="584"><span class="str">                   long set(long a, long b, double c, double d);
</span></code><code id="line-585" aria-labelledby="585"><span class="str">                   long delete(long a, long b, double c, double d);
</span></code><code id="line-586" aria-labelledby="586"><span class="str">                   };
</span></code><code id="line-587" aria-labelledby="587"><span class="str">                   """</span>, mapRWMembers)
</code><code id="line-588" aria-labelledby="588">
</code><code id="line-589" aria-labelledby="589">    <span class="k">for</span> m in r[0].members:
</code><code id="line-590" aria-labelledby="590">        <span class="k">if</span> m.identifier.name in [<span class="str">"</span><span class="str">clear</span><span class="str">"</span>, <span class="str">"</span><span class="str">set</span><span class="str">"</span>, <span class="str">"</span><span class="str">delete</span><span class="str">"</span>]:
</code><code id="line-591" aria-labelledby="591">            harness.ok(m.isMethod(), <span class="str">"</span><span class="str">%s</span><span class="str"> should be a method</span><span class="str">"</span> % m.identifier.name)
</code><code id="line-592" aria-labelledby="592">            harness.check(m.maxArgCount, 4, <span class="str">"</span><span class="str">%s</span><span class="str"> should have 4 arguments</span><span class="str">"</span> % m.identifier.name)
</code><code id="line-593" aria-labelledby="593">            harness.ok(not m.isMaplikeOrSetlikeOrIterableMethod(),
</code><code id="line-594" aria-labelledby="594">                       <span class="str">"</span><span class="str">%s</span><span class="str"> should not be a maplike/setlike function</span><span class="str">"</span> % m.identifier.name)
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