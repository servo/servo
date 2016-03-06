<!DOCTYPE html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    
  <link rel="shortcut icon" href="/static/icons/mimetypes/py.5ef6367a.png" />

    <title>test_float_types.py - DXR</title>

    
  
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
      
  
  <div class="breadcrumbs"><a href="/mozilla-central/source">mozilla-central</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom">dom</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings">bindings</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings/parser">parser</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings/parser/tests">tests</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings/parser/tests/test_float_types.py">test_float_types.py</a></div>

  
  
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
              <a href="/build-central/parallel/dom/bindings/parser/tests/test_float_types.py" >
                <span class="selector-option-label">build-central</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/comm-central/parallel/dom/bindings/parser/tests/test_float_types.py" >
                <span class="selector-option-label">comm-central</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/hgcustom_version-control-tools/parallel/dom/bindings/parser/tests/test_float_types.py" >
                <span class="selector-option-label">hgcustom_version-control-tools</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/mozilla-central/parallel/dom/bindings/parser/tests/test_float_types.py" class="selected" aria-checked="true">
                <span class="selector-option-label">mozilla-central</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/nss/parallel/dom/bindings/parser/tests/test_float_types.py" >
                <span class="selector-option-label">nss</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/rust/parallel/dom/bindings/parser/tests/test_float_types.py" >
                <span class="selector-option-label">rust</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/rustfmt/parallel/dom/bindings/parser/tests/test_float_types.py" >
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
                <a href="/mozilla-central/rev/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_float_types.py" title="Permalink" class="permalink icon">Permalink</a>
              </li>
          </ul>
        
          <h4>Untracked file</h4>
          <ul>
            
          </ul>
        
          <h4>VCS Links</h4>
          <ul>
            
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/filelog/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_float_types.py" title="Log" class="log icon">Log</a>
              </li>
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/annotate/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_float_types.py" title="Blame" class="blame icon">Blame</a>
              </li>
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/diff/8375549bb321502e2cc963dcda89c516203452b7/dom/bindings/parser/tests/test_float_types.py" title="Diff" class="diff icon">Diff</a>
              </li>
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/raw-file/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_float_types.py" title="Raw" class="raw icon">Raw</a>
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
      <div class="annotation-set" id="aset-125"></div></div>

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
          
        </td>
        <td class="code">
          
<pre>
<code id="line-1" aria-labelledby="1"><span class="k">import</span> WebIDL
</code><code id="line-2" aria-labelledby="2">
</code><code id="line-3" aria-labelledby="3"><span class="k">def</span> WebIDLTest(parser, harness):
</code><code id="line-4" aria-labelledby="4">    parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-5" aria-labelledby="5"><span class="str">        typedef float myFloat;</span><span class="str">
</span></code><code id="line-6" aria-labelledby="6"><span class="str">        typedef unrestricted float myUnrestrictedFloat;</span><span class="str">
</span></code><code id="line-7" aria-labelledby="7"><span class="str">        interface FloatTypes {</span><span class="str">
</span></code><code id="line-8" aria-labelledby="8"><span class="str">          attribute float f;</span><span class="str">
</span></code><code id="line-9" aria-labelledby="9"><span class="str">          attribute unrestricted float uf;</span><span class="str">
</span></code><code id="line-10" aria-labelledby="10"><span class="str">          attribute double d;</span><span class="str">
</span></code><code id="line-11" aria-labelledby="11"><span class="str">          attribute unrestricted double ud;</span><span class="str">
</span></code><code id="line-12" aria-labelledby="12"><span class="str">          [LenientFloat]</span><span class="str">
</span></code><code id="line-13" aria-labelledby="13"><span class="str">          attribute float lf;</span><span class="str">
</span></code><code id="line-14" aria-labelledby="14"><span class="str">          [LenientFloat]</span><span class="str">
</span></code><code id="line-15" aria-labelledby="15"><span class="str">          attribute double ld;</span><span class="str">
</span></code><code id="line-16" aria-labelledby="16"><span class="str">
</span></code><code id="line-17" aria-labelledby="17"><span class="str">          void m1(float arg1, double arg2, float? arg3, double? arg4,</span><span class="str">
</span></code><code id="line-18" aria-labelledby="18"><span class="str">                  myFloat arg5, unrestricted float arg6,</span><span class="str">
</span></code><code id="line-19" aria-labelledby="19"><span class="str">                  unrestricted double arg7, unrestricted float? arg8,</span><span class="str">
</span></code><code id="line-20" aria-labelledby="20"><span class="str">                  unrestricted double? arg9, myUnrestrictedFloat arg10);</span><span class="str">
</span></code><code id="line-21" aria-labelledby="21"><span class="str">          [LenientFloat]</span><span class="str">
</span></code><code id="line-22" aria-labelledby="22"><span class="str">          void m2(float arg1, double arg2, float? arg3, double? arg4,</span><span class="str">
</span></code><code id="line-23" aria-labelledby="23"><span class="str">                  myFloat arg5, unrestricted float arg6,</span><span class="str">
</span></code><code id="line-24" aria-labelledby="24"><span class="str">                  unrestricted double arg7, unrestricted float? arg8,</span><span class="str">
</span></code><code id="line-25" aria-labelledby="25"><span class="str">                  unrestricted double? arg9, myUnrestrictedFloat arg10);</span><span class="str">
</span></code><code id="line-26" aria-labelledby="26"><span class="str">          [LenientFloat]</span><span class="str">
</span></code><code id="line-27" aria-labelledby="27"><span class="str">          void m3(float arg);</span><span class="str">
</span></code><code id="line-28" aria-labelledby="28"><span class="str">          [LenientFloat]</span><span class="str">
</span></code><code id="line-29" aria-labelledby="29"><span class="str">          void m4(double arg);</span><span class="str">
</span></code><code id="line-30" aria-labelledby="30"><span class="str">          [LenientFloat]</span><span class="str">
</span></code><code id="line-31" aria-labelledby="31"><span class="str">          void m5((float or FloatTypes) arg);</span><span class="str">
</span></code><code id="line-32" aria-labelledby="32"><span class="str">          [LenientFloat]</span><span class="str">
</span></code><code id="line-33" aria-labelledby="33"><span class="str">          void m6(sequence&lt;float&gt; arg);</span><span class="str">
</span></code><code id="line-34" aria-labelledby="34"><span class="str">        };</span><span class="str">
</span></code><code id="line-35" aria-labelledby="35"><span class="str">    </span><span class="str">"""</span>)
</code><code id="line-36" aria-labelledby="36">
</code><code id="line-37" aria-labelledby="37">    results = parser.finish()
</code><code id="line-38" aria-labelledby="38">
</code><code id="line-39" aria-labelledby="39">    harness.check(len(results), 3, <span class="str">"</span><span class="str">Should be two typedefs and one interface.</span><span class="str">"</span>)
</code><code id="line-40" aria-labelledby="40">    iface = results[2]
</code><code id="line-41" aria-labelledby="41">    harness.ok(isinstance(iface, WebIDL.IDLInterface),
</code><code id="line-42" aria-labelledby="42">               <span class="str">"</span><span class="str">Should be an IDLInterface</span><span class="str">"</span>)
</code><code id="line-43" aria-labelledby="43">    types = [a.type <span class="k">for</span> a in iface.members <span class="k">if</span> a.isAttr()]
</code><code id="line-44" aria-labelledby="44">    harness.ok(types[0].isFloat(), <span class="str">"</span><span class="str">'</span><span class="str">float</span><span class="str">'</span><span class="str"> is a float</span><span class="str">"</span>)
</code><code id="line-45" aria-labelledby="45">    harness.ok(not types[0].isUnrestricted(), <span class="str">"</span><span class="str">'</span><span class="str">float</span><span class="str">'</span><span class="str"> is not unrestricted</span><span class="str">"</span>)
</code><code id="line-46" aria-labelledby="46">    harness.ok(types[1].isFloat(), <span class="str">"</span><span class="str">'</span><span class="str">unrestricted float</span><span class="str">'</span><span class="str"> is a float</span><span class="str">"</span>)
</code><code id="line-47" aria-labelledby="47">    harness.ok(types[1].isUnrestricted(), <span class="str">"</span><span class="str">'</span><span class="str">unrestricted float</span><span class="str">'</span><span class="str"> is unrestricted</span><span class="str">"</span>)
</code><code id="line-48" aria-labelledby="48">    harness.ok(types[2].isFloat(), <span class="str">"</span><span class="str">'</span><span class="str">double</span><span class="str">'</span><span class="str"> is a float</span><span class="str">"</span>)
</code><code id="line-49" aria-labelledby="49">    harness.ok(not types[2].isUnrestricted(), <span class="str">"</span><span class="str">'</span><span class="str">double</span><span class="str">'</span><span class="str"> is not unrestricted</span><span class="str">"</span>)
</code><code id="line-50" aria-labelledby="50">    harness.ok(types[3].isFloat(), <span class="str">"</span><span class="str">'</span><span class="str">unrestricted double</span><span class="str">'</span><span class="str"> is a float</span><span class="str">"</span>)
</code><code id="line-51" aria-labelledby="51">    harness.ok(types[3].isUnrestricted(), <span class="str">"</span><span class="str">'</span><span class="str">unrestricted double</span><span class="str">'</span><span class="str"> is unrestricted</span><span class="str">"</span>)
</code><code id="line-52" aria-labelledby="52">
</code><code id="line-53" aria-labelledby="53">    method = iface.members[6]
</code><code id="line-54" aria-labelledby="54">    harness.ok(isinstance(method, WebIDL.IDLMethod), <span class="str">"</span><span class="str">Should be an IDLMethod</span><span class="str">"</span>)
</code><code id="line-55" aria-labelledby="55">    argtypes = [a.type <span class="k">for</span> a in method.signatures()[0][1]]
</code><code id="line-56" aria-labelledby="56">    <span class="k">for</span> (idx, type) in enumerate(argtypes):
</code><code id="line-57" aria-labelledby="57">        harness.ok(type.isFloat(), <span class="str">"</span><span class="str">Type </span><span class="str">%d</span><span class="str"> should be float</span><span class="str">"</span> % idx)
</code><code id="line-58" aria-labelledby="58">        harness.check(type.isUnrestricted(), idx &gt;= 5,
</code><code id="line-59" aria-labelledby="59">                      <span class="str">"</span><span class="str">Type </span><span class="str">%d</span><span class="str"> should </span><span class="str">%s</span><span class="str">be unrestricted</span><span class="str">"</span> % (
</code><code id="line-60" aria-labelledby="60">                idx, <span class="str">"</span><span class="str">"</span> <span class="k">if</span> idx &gt;= 4 <span class="k">else</span> <span class="str">"</span><span class="str">not </span><span class="str">"</span>))
</code><code id="line-61" aria-labelledby="61">
</code><code id="line-62" aria-labelledby="62">    parser = parser.reset()
</code><code id="line-63" aria-labelledby="63">    threw = False
</code><code id="line-64" aria-labelledby="64">    <span class="k">try</span>:
</code><code id="line-65" aria-labelledby="65">        parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-66" aria-labelledby="66"><span class="str">            interface FloatTypes {</span><span class="str">
</span></code><code id="line-67" aria-labelledby="67"><span class="str">              [LenientFloat]</span><span class="str">
</span></code><code id="line-68" aria-labelledby="68"><span class="str">              long m(float arg);</span><span class="str">
</span></code><code id="line-69" aria-labelledby="69"><span class="str">            };</span><span class="str">
</span></code><code id="line-70" aria-labelledby="70"><span class="str">        </span><span class="str">"""</span>)
</code><code id="line-71" aria-labelledby="71">    <span class="k">except</span> Exception, x:
</code><code id="line-72" aria-labelledby="72">        threw = True
</code><code id="line-73" aria-labelledby="73">    harness.ok(threw, <span class="str">"</span><span class="str">[LenientFloat] only allowed on void methods</span><span class="str">"</span>)
</code><code id="line-74" aria-labelledby="74">
</code><code id="line-75" aria-labelledby="75">    parser = parser.reset()
</code><code id="line-76" aria-labelledby="76">    threw = False
</code><code id="line-77" aria-labelledby="77">    <span class="k">try</span>:
</code><code id="line-78" aria-labelledby="78">        parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-79" aria-labelledby="79"><span class="str">            interface FloatTypes {</span><span class="str">
</span></code><code id="line-80" aria-labelledby="80"><span class="str">              [LenientFloat]</span><span class="str">
</span></code><code id="line-81" aria-labelledby="81"><span class="str">              void m(unrestricted float arg);</span><span class="str">
</span></code><code id="line-82" aria-labelledby="82"><span class="str">            };</span><span class="str">
</span></code><code id="line-83" aria-labelledby="83"><span class="str">        </span><span class="str">"""</span>)
</code><code id="line-84" aria-labelledby="84">    <span class="k">except</span> Exception, x:
</code><code id="line-85" aria-labelledby="85">        threw = True
</code><code id="line-86" aria-labelledby="86">    harness.ok(threw, <span class="str">"</span><span class="str">[LenientFloat] only allowed on methods with unrestricted float args</span><span class="str">"</span>)
</code><code id="line-87" aria-labelledby="87">
</code><code id="line-88" aria-labelledby="88">    parser = parser.reset()
</code><code id="line-89" aria-labelledby="89">    threw = False
</code><code id="line-90" aria-labelledby="90">    <span class="k">try</span>:
</code><code id="line-91" aria-labelledby="91">        parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-92" aria-labelledby="92"><span class="str">            interface FloatTypes {</span><span class="str">
</span></code><code id="line-93" aria-labelledby="93"><span class="str">              [LenientFloat]</span><span class="str">
</span></code><code id="line-94" aria-labelledby="94"><span class="str">              void m(sequence&lt;unrestricted float&gt; arg);</span><span class="str">
</span></code><code id="line-95" aria-labelledby="95"><span class="str">            };</span><span class="str">
</span></code><code id="line-96" aria-labelledby="96"><span class="str">        </span><span class="str">"""</span>)
</code><code id="line-97" aria-labelledby="97">    <span class="k">except</span> Exception, x:
</code><code id="line-98" aria-labelledby="98">        threw = True
</code><code id="line-99" aria-labelledby="99">    harness.ok(threw, <span class="str">"</span><span class="str">[LenientFloat] only allowed on methods with unrestricted float args (2)</span><span class="str">"</span>)
</code><code id="line-100" aria-labelledby="100">
</code><code id="line-101" aria-labelledby="101">    parser = parser.reset()
</code><code id="line-102" aria-labelledby="102">    threw = False
</code><code id="line-103" aria-labelledby="103">    <span class="k">try</span>:
</code><code id="line-104" aria-labelledby="104">        parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-105" aria-labelledby="105"><span class="str">            interface FloatTypes {</span><span class="str">
</span></code><code id="line-106" aria-labelledby="106"><span class="str">              [LenientFloat]</span><span class="str">
</span></code><code id="line-107" aria-labelledby="107"><span class="str">              void m((unrestricted float or FloatTypes) arg);</span><span class="str">
</span></code><code id="line-108" aria-labelledby="108"><span class="str">            };</span><span class="str">
</span></code><code id="line-109" aria-labelledby="109"><span class="str">        </span><span class="str">"""</span>)
</code><code id="line-110" aria-labelledby="110">    <span class="k">except</span> Exception, x:
</code><code id="line-111" aria-labelledby="111">        threw = True
</code><code id="line-112" aria-labelledby="112">    harness.ok(threw, <span class="str">"</span><span class="str">[LenientFloat] only allowed on methods with unrestricted float args (3)</span><span class="str">"</span>)
</code><code id="line-113" aria-labelledby="113">
</code><code id="line-114" aria-labelledby="114">    parser = parser.reset()
</code><code id="line-115" aria-labelledby="115">    threw = False
</code><code id="line-116" aria-labelledby="116">    <span class="k">try</span>:
</code><code id="line-117" aria-labelledby="117">        parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-118" aria-labelledby="118"><span class="str">            interface FloatTypes {</span><span class="str">
</span></code><code id="line-119" aria-labelledby="119"><span class="str">              [LenientFloat]</span><span class="str">
</span></code><code id="line-120" aria-labelledby="120"><span class="str">              readonly attribute float foo;</span><span class="str">
</span></code><code id="line-121" aria-labelledby="121"><span class="str">            };</span><span class="str">
</span></code><code id="line-122" aria-labelledby="122"><span class="str">        </span><span class="str">"""</span>)
</code><code id="line-123" aria-labelledby="123">    <span class="k">except</span> Exception, x:
</code><code id="line-124" aria-labelledby="124">        threw = True
</code><code id="line-125" aria-labelledby="125">    harness.ok(threw, <span class="str">"</span><span class="str">[LenientFloat] only allowed on writable attributes</span><span class="str">"</span>)
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