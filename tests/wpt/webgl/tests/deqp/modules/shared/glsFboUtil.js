/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
 *
 * Copyright 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

'use strict';
goog.provide('modules.shared.glsFboUtil');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.opengl.gluStrUtil');

goog.scope(function() {

    var glsFboUtil = modules.shared.glsFboUtil;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var gluStrUtil = framework.opengl.gluStrUtil;
    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    /**
    * @constructor
    * @template KeyT
    * @template ValueT
    * @param {function(!KeyT, !KeyT):boolean} comparefnc
    */
    glsFboUtil.Map = function(comparefnc) {
        /** @type {Array<{first: !KeyT, second: ValueT}>} */
        this.store = [];
        this.compare = comparefnc;
        this.length = 0;
    };

    /**
    * @param {number} num1
    * @param {number} num2
    * @return {boolean}
    */
    glsFboUtil.Map.compareNumber = function(num1, num2) {
        return num1 < num2;
    };

    /**
    * @param {!KeyT} key
    * @param {ValueT} value
    * @return {{first: !KeyT, second: ValueT}}
    */
    glsFboUtil.Map.prototype.pair = function(key, value) {
        return {
            first: key,
            second: value
        };
    };

    /**
    * @protected
    * @param {!KeyT} key
    * @return {number}
    */
    glsFboUtil.Map.prototype.findInsertionPoint = function(key) {
        var /** @type {number} */i, /** @type {number} */length;
        for (i = 0, length = this.store.length; i < length; ++i) {
            if (!this.compare(key, this.store[i].first)) break;
        }
        return i;
    };

    /**
    * index should be a value returned from findInsertionPoint.
    * returns true if the compare function returns false reflexively
    * (i.e. no matter the order in which the keys are passed as arguments).
    * @protected
    * @param {!KeyT} key
    * @param {number} index
    * @return {boolean}
    */
    glsFboUtil.Map.prototype.foundIndexMatches = function(key, index) {
        return (
            this.store[index] !== undefined &&
            !this.compare(this.store[index].first, key)
        );
    };

    /**
    * @param {!KeyT} key
    * @return {boolean}
    */
    glsFboUtil.Map.prototype.isset = function(key) {
        return this.foundIndexMatches(key, this.findInsertionPoint(key));
    };

    /**
    * @param {!KeyT} key
    * @param {ValueT} value
    */
    glsFboUtil.Map.prototype.set = function(key, value) {
        var index = this.findInsertionPoint(key);
        if (this.foundIndexMatches(key, index)) {
            this.store[index].second = value;
        } else {
            this.store.splice(index, 0, this.pair(key, value));
            ++this.length;
        }
    };

    /**
    * @param {!KeyT} key
    * @return {?ValueT}
    */
    glsFboUtil.Map.prototype.remove = function(key) {
        var index = this.findInsertionPoint(key);
        /** @type {?ValueT} */ var ret = null;
        if (this.foundIndexMatches(key, index)) {
            ret = this.store[index].second;
            this.store.splice(index, 1);
            --this.length;
        }
        return ret;
    };

    /**
    * @param {KeyT} key
    * @return {?{first: KeyT, second: ValueT}}
    */
    glsFboUtil.Map.prototype.get = function(key) {
        var index = this.findInsertionPoint(key);
        if (this.foundIndexMatches(key, index)) return this.store[index];
        return null;
    };

    /**
    * @param {KeyT} key
    * @return {?ValueT}
    */
    glsFboUtil.Map.prototype.getValue = function(key) {
        var index = this.findInsertionPoint(key);
        if (this.foundIndexMatches(key, index)) return this.store[index].second;
        return null;
    };

    /**
    * @param {!KeyT} key
    * @param {ValueT} fallback
    * @return {ValueT}
    */
    glsFboUtil.Map.prototype.lookupDefault = function(key, fallback) {
        var index = this.findInsertionPoint(key);
        if (this.foundIndexMatches(key, index)) return this.store[index].second;
        return fallback;
    };

    /**
    * @param {number} index
    * @return {{first: KeyT, second: ValueT}|undefined}
    */
    glsFboUtil.Map.prototype.getIndex = function(index) {
        return this.store[index];
    };

    /**
    * Use the callback to set the value to be identified by key.
    * If a value is already identified by the key, it will be passed to the callback
    * @param {!KeyT} key
    * @param {function(ValueT=):!ValueT} callback
    */
    glsFboUtil.Map.prototype.transform = function(key, callback) {
        var index = this.findInsertionPoint(key);
        if (this.foundIndexMatches(key, index)) {
            this.store[index].second = callback(this.store[index].second);
        } else {
            this.store.splice(index, 0, this.pair(key, callback()));
            ++this.length;
        }
    };

    /**
    * removed all elements from the Map
    */
    glsFboUtil.Map.prototype.clear = function() {
        this.store.splice(0, this.length);
        this.length = 0;
    };

    /**
    * @constructor
    */
    glsFboUtil.FormatDB = function() {
        this.m_map = /** @type {glsFboUtil.Map<glsFboUtil.ImageFormat,number>} */(
            new glsFboUtil.Map(glsFboUtil.ImageFormat.lessthan)
        );
    };

    /**
    * @param {glsFboUtil.ImageFormat} format
    * @param {number} newFlags
    */
    glsFboUtil.FormatDB.prototype.addFormat = function(format, newFlags) {
        this.m_map.transform(format, function(flags) {
            return flags | newFlags;
        });
    };

    /**
    * @param {number} requirements
    * @return {Array<glsFboUtil.ImageFormat>}
    */
    glsFboUtil.FormatDB.prototype.getFormats = function(requirements) {
        /** @type {Array<glsFboUtil.ImageFormat>} */ var ret = [];
        for (var i = 0; i < this.m_map.length; ++i) {
            var pair = this.m_map.getIndex(i);
            if ((pair.second & requirements) == requirements)
            ret.push(pair.first);
        }

        return ret;
    };

    /**
    * @param {glsFboUtil.ImageFormat} format
    * @param {number} fallback
    * @return {number}
    */
    glsFboUtil.FormatDB.prototype.getFormatInfo = function(format, fallback) {
        return this.m_map.lookupDefault(format, fallback);
    };

    /**
    * @param {Object} map
    * @param {number} key
    * @param {number} fallback
    * @return {number}
    */
    glsFboUtil.lookupDefault = function(map, key, fallback) {
        return (map[key] !== undefined) ? map[key] : fallback;
    };

    /**
    * @param {Array<number>} array
    * @param {number} item
    * @return {boolean}
    */
    glsFboUtil.contains = function(array, item) {
        var l = array.length;
        for (var i = 0; i < l; ++i)
            if (array[i] == item) return true;
        return false;
    };

    /**
    * @typedef {Array<(number | glsFboUtil.Range<number>)>}
    */
    glsFboUtil.formatT;

    /**
    * @param {glsFboUtil.FormatDB} db
    * @param {glsFboUtil.Range<glsFboUtil.formatT>} stdFmts
    */
    glsFboUtil.addFormats = function(db, stdFmts) {
        for (var set = stdFmts.reset(); set = stdFmts.current(); stdFmts.next()) {
            for (var fmt = set[1].reset(); fmt = set[1].current(); set[1].next()) {
                db.addFormat(glsFboUtil.formatKeyInfo(fmt), set[0]);
            }
        }
    };

    /**
    * @param {glsFboUtil.FormatDB} db
    * @param {glsFboUtil.Range} extFmts
    * @param {WebGLRenderingContextBase=} gl
    * @throws {Error}
    */
    glsFboUtil.addExtFormats = function(db, extFmts, gl) {
        if (!(gl = gl || window.gl)) throw new Error('Invalid gl object');
        var extensions = gl.getSupportedExtensions();

        // loop through the range, looking at the extentions.
        for (var ext = extFmts.reset(); ext = extFmts.current(); extFmts.next()) {
            var tokens = ext.extensions.split(/\s+/);

            var supported = function() {
                for (var i = 0, l = tokens.length; i < l; ++i)
                    if (extensions.indexOf(tokens[i]) === -1) return false;
                return true;
            }();

            if (supported) {
                for (var format = ext.formats.reset(); format = ext.formats.current(); ext.formats.next()) {
                    db.addFormat(glsFboUtil.formatKeyInfo(format), ext.flags);
                }
            }

        }

    };

    /**
    * @param {number} glenum
    * @param {WebGLRenderingContextBase=} gl
    * @return {number}
    * @throws {Error}
    */
    glsFboUtil.formatFlag = function(glenum, gl) {
        if (!(gl = gl || window.gl)) throw new Error('Invalid gl object');

        switch (glenum) {
         case gl.NONE:
            return glsFboUtil.FormatFlags.ANY_FORMAT;
         case gl.RENDERBUFFER:
            return glsFboUtil.FormatFlags.RENDERBUFFER_VALID;
         case gl.TEXTURE:
            return glsFboUtil.FormatFlags.TEXTURE_VALID;
         case gl.STENCIL_ATTACHMENT:
            return glsFboUtil.FormatFlags.STENCIL_RENDERABLE;
         case gl.DEPTH_ATTACHMENT:
            return glsFboUtil.FormatFlags.DEPTH_RENDERABLE;
         default:
            if (glenum < gl.COLOR_ATTACHMENT0 || glenum > gl.COLOR_ATTACHMENT15) {
                throw new Error('glenum out of range');
            }
        }
        return glsFboUtil.FormatFlags.COLOR_RENDERABLE;
    };

    /**
    * Remove value from array
    * @param {Array} array
    * @param {number} value
    */
    glsFboUtil.remove_from_array = function(array, value) {
        var index = 0;
        while ((index = array.indexOf(value)) != -1) {
            array.splice(index, 1);
        }
    };

    /**
     * glsFboUtil.FormatExtEntry
     * @constructor
     * @struct
     * @param {string=} extensions
     * @param {number=} flags
     * @param {glsFboUtil.Range=} formats
     */
    glsFboUtil.FormatExtEntry = function(extensions, flags, formats) {
        this.extensions = null;
        this.flags = null;
        this.formats = null;

        if (extensions !== undefined) {
            this.extensions = extensions;
            if (flags !== undefined) {
                this.flags = flags;
                if (formats !== undefined)
                    this.formats = formats;
            }
        }

    };

    /**
     * glsFboUtil.Range
     * @constructor
     * @struct
     * @template T
     * @param {Array<T>} array
     * @param {number=} begin
     * @param {number=} end
     */
    glsFboUtil.Range = function(array, begin, end) {
        // @private
        this.m_begin = (begin === undefined ? 0 : begin);
        // @private
        this.m_end = end || array.length;
        /**
        * @private
        * @type {Array<T>}
        */
        this.m_array = array;
        // @private
        this.m_index = this.m_begin;
    };

    /**
    * @return {Array<T>}
    */
    glsFboUtil.Range.prototype.array = function() {
        return this.m_array;
    };

    /**
    * @return {number}
    */
    glsFboUtil.Range.prototype.begin = function() {
        return this.m_begin;
    };

    /**  *generated by script*
    * @return {number}
    */
    glsFboUtil.Range.prototype.end = function() {
        return this.m_end;
    };

    /**
    * Returns the current pointer index as well as the current object
    * @param {number} id
    * @return {{first: number, second: T}}
    */
    glsFboUtil.Range.prototype.get = function(id) {
        return {
            first: id,
            second: this.m_array[id]
        };
    };

    /**
    * Sets the internal pointer to the beginning of the range, and returns the first object.
    * @return {T}
    */
    glsFboUtil.Range.prototype.reset = function() {
        this.m_index = this.m_begin;
        return this.current();
    };

    /**
    * returns the current object within the specified range. The internal pointer is unaltered.
    * @return {T}
    */
    glsFboUtil.Range.prototype.current = function() {
        return this.m_index < this.m_end ? this.m_array[this.m_index] : null;
    };

    /**
    * Increments the internal pointer
    */
    glsFboUtil.Range.prototype.next = function() {
        ++this.m_index;
    };

    /**
     * glsFboUtil.rangeArray
     * replaces the macro GLS_ARRAY_RANGE
     * Creates a new Range object from the specified array, spanning the whole array.
     * @template T
     * @param {Array<T>} array
     * @return {glsFboUtil.Range<T>}
     */
    glsFboUtil.rangeArray = function(array) {
        return new glsFboUtil.Range(array);
    };

    /**
    * @constructor
    * @struct
    * @param {number=} format
    * @param {number=} unsizedType
    */
    glsFboUtil.ImageFormat = function(format, unsizedType) {
        this.format = format || 0;
        //! Type if format is unsized, gl.NONE if sized.
        this.unsizedType = unsizedType || 0;

    };

    /**
    * @param {!glsFboUtil.ImageFormat} obj1
    * @param {!glsFboUtil.ImageFormat} obj2
    * @return {boolean}
    */
    glsFboUtil.ImageFormat.lessthan = function(obj1, obj2) {
        return (
            (obj1.format < obj2.format) ||
            (obj1.format == obj2.format && obj1.unsizedType < obj2.unsizedType)
        );
    };

    /**
    * Sets the object's parameters to gl.NONE
    */
    glsFboUtil.ImageFormat.prototype.none = function() {
        this.format = 0;
        this.unsizedType = 0;
    };

    /**
    * @return {glsFboUtil.ImageFormat}
    */
    glsFboUtil.ImageFormat.none = function() {
        var obj = new glsFboUtil.ImageFormat();
        obj.none();
        return obj;
    };

    // where key is a FormatKey, and a FormatKey is a unsigned 32bit int.

    /**
    * @param {number} key
    * @return {glsFboUtil.ImageFormat}
    */
    glsFboUtil.formatKeyInfo = function(key) {
        return new glsFboUtil.ImageFormat(
            (key & 0x0000ffff),
            (key & 0xffff0000) >>> 16
        );
    };

    /**
     * glsFboUtil.Config Class.
     * @constructor
     */
    glsFboUtil.Config = function() {
        this.type = glsFboUtil.Config.s_types.CONFIG;
        this.target = glsFboUtil.Config.s_target.NONE;
    };
    /**
     * @enum {number}
     */
    glsFboUtil.Config.s_target = {
        NONE: 0,
        RENDERBUFFER: 0x8D41,
        TEXTURE_2D: 0x0DE1,
        TEXTURE_CUBE_MAP: 0x8513,
        TEXTURE_3D: 0x806F,
        TEXTURE_2D_ARRAY: 0x8C1A,

        FRAMEBUFFER: 0x8D40
    };

    // the c++ uses dynamic casts to determain if an object inherits from a
    // given class. Here, each class' constructor assigns a bit to obj.type.
    // look for the bit to see if an object inherits that class.

    /**
    * @enum
    */
    glsFboUtil.Config.s_types = {
        CONFIG: 0x000001,

        IMAGE: 0x000010,
        RENDERBUFFER: 0x000020,
        TEXTURE: 0x000040,
        TEXTURE_FLAT: 0x000080,
        TEXTURE_2D: 0x000100,
        TEXTURE_CUBE_MAP: 0x000200,
        TEXTURE_LAYERED: 0x000400,
        TEXTURE_3D: 0x000800,
        TEXTURE_2D_ARRAY: 0x001000,

        ATTACHMENT: 0x010000,
        ATT_RENDERBUFFER: 0x020000,
        ATT_TEXTURE: 0x040000,
        ATT_TEXTURE_FLAT: 0x080000,
        ATT_TEXTURE_LAYER: 0x100000,

        UNUSED: 0xFFE0E00E
    };

    /**
     * glsFboUtil.Image Class.
     * @constructor
     * @extends {glsFboUtil.Config}
     */
    glsFboUtil.Image = function() {
        glsFboUtil.Config.call(this);
        this.type |= glsFboUtil.Config.s_types.IMAGE;
        this.width = 0;
        this.height = 0;
        this.internalFormat = new glsFboUtil.ImageFormat();
    };

    /**
     * glsFboUtil.Renderbuffer Class.
     * @constructor
     * @extends {glsFboUtil.Image}
     */
    glsFboUtil.Renderbuffer = function() {
        glsFboUtil.Image.call(this);
        this.type |= glsFboUtil.Config.s_types.RENDERBUFFER;
        this.target = glsFboUtil.Config.s_target.RENDERBUFFER;
        this.numSamples = 0;
    };

    /**
     * glsFboUtil.Texture Class.
     * @constructor
     * @extends {glsFboUtil.Image}
     */
    glsFboUtil.Texture = function() {
        glsFboUtil.Image.call(this);
        this.type |= glsFboUtil.Config.s_types.TEXTURE;
        this.numLevels = 1;
    };

    /**
     * glsFboUtil.TextureFlat Class.
     * @constructor
     * @extends {glsFboUtil.Texture}
     */
    glsFboUtil.TextureFlat = function() {
        glsFboUtil.Texture.call(this);
        this.type |= glsFboUtil.Config.s_types.TEXTURE_FLAT;
    };

    /**
     * glsFboUtil.Texture2D Class.
     * @constructor
     * @extends {glsFboUtil.TextureFlat}
     */
    glsFboUtil.Texture2D = function() {
        glsFboUtil.TextureFlat.call(this);
        this.type |= glsFboUtil.Config.s_types.TEXTURE_2D;
        this.target = glsFboUtil.Config.s_target.TEXTURE_2D;
    };

    /**
     * glsFboUtil.TextureCubeMap Class.
     * @constructor
     * @extends {glsFboUtil.TextureFlat}
     */
    glsFboUtil.TextureCubeMap = function() {
        glsFboUtil.TextureFlat.call(this);
        this.type |= glsFboUtil.Config.s_types.TEXTURE_CUBE_MAP;
        this.target = glsFboUtil.Config.s_target.TEXTURE_CUBE_MAP;
    };

    /**
     * glsFboUtil.TextureLayered Class.
     * @constructor
     * @extends {glsFboUtil.Texture}
     */
    glsFboUtil.TextureLayered = function() {
        glsFboUtil.Texture.call(this);
        this.type |= glsFboUtil.Config.s_types.TEXTURE_LAYERED;
        this.numLayers = 1;
    };

    /**
     * glsFboUtil.Texture3D Class.
     * @constructor
     * @extends {glsFboUtil.TextureLayered}
     */
    glsFboUtil.Texture3D = function() {
        glsFboUtil.TextureLayered.call(this);
        this.type |= glsFboUtil.Config.s_types.TEXTURE_3D;
        this.target = glsFboUtil.Config.s_target.TEXTURE_3D;
    };

    /**
     * glsFboUtil.Texture2DArray Class.
     * @constructor
     * @extends {glsFboUtil.TextureLayered}
     */
    glsFboUtil.Texture2DArray = function() {
        glsFboUtil.TextureLayered.call(this);
        this.type |= glsFboUtil.Config.s_types.TEXTURE_2D_ARRAY;
        this.target = glsFboUtil.Config.s_target.TEXTURE_2D_ARRAY;
    };

    /**
     * glsFboUtil.Attachment Class.
     * @constructor
     * @extends {glsFboUtil.Config}
     */
    glsFboUtil.Attachment = function() {
        glsFboUtil.Config.call(this);

        this.type |= glsFboUtil.Config.s_types.ATTACHMENT;

        /** @type {glsFboUtil.Config.s_target} */
        this.target = glsFboUtil.Config.s_target.FRAMEBUFFER;

        /** @type {WebGLObject} */
        this.imageName = null;
    };

    /**
    * this function is declared, but has no definition/is unused in the c++
    * @param {number} attPoint
    * @param {number} image
    * @param {number} vfr
    */
    glsFboUtil.Attachment.prototype.isComplete = function(attPoint, image, vfr) { };

    /**
     * glsFboUtil.RenderBufferAttachments Class.
     * @constructor
     * @extends {glsFboUtil.Attachment}
     */
    glsFboUtil.RenderbufferAttachment = function() {
        glsFboUtil.Attachment.call(this);
        this.type |= glsFboUtil.Config.s_types.ATT_RENDERBUFFER;
        this.renderbufferTarget = glsFboUtil.Config.s_target.RENDERBUFFER;
    };
    glsFboUtil.RenderbufferAttachment.prototype = Object.create(glsFboUtil.Attachment.prototype);
    glsFboUtil.RenderbufferAttachment.prototype.constructor = glsFboUtil.RenderbufferAttachment;

    /**
     * glsFboUtil.TextureAttachment Class.
     * @constructor
     * @extends {glsFboUtil.Attachment}
     */
    glsFboUtil.TextureAttachment = function() {
        glsFboUtil.Attachment.call(this);
        this.type |= glsFboUtil.Config.s_types.ATT_TEXTURE;
        this.level = 0;
    };
    glsFboUtil.TextureAttachment.prototype = Object.create(glsFboUtil.Attachment.prototype);
    glsFboUtil.TextureAttachment.prototype.constructor = glsFboUtil.TextureAttachment;

    /**
     * glsFboUtil.TextureFlatAttachment Class.
     * @constructor
     * @extends {glsFboUtil.TextureAttachment}
     */
    glsFboUtil.TextureFlatAttachment = function() {
        glsFboUtil.TextureAttachment.call(this);
        this.type |= glsFboUtil.Config.s_types.ATT_TEXTURE_FLAT;
        this.texTarget = glsFboUtil.Config.s_target.NONE;
    };
    glsFboUtil.TextureFlatAttachment.prototype = Object.create(glsFboUtil.TextureAttachment.prototype);
    glsFboUtil.TextureFlatAttachment.prototype.constructor = glsFboUtil.TextureFlatAttachment;

    /**
     * glsFboUtil.TextureLayerAttachment Class.
     * @constructor
     * @extends {glsFboUtil.TextureAttachment}
     */
    glsFboUtil.TextureLayerAttachment = function() {
        glsFboUtil.TextureAttachment.call(this);
        this.type |= glsFboUtil.Config.s_types.ATT_TEXTURE_LAYER;
        this.layer = 0;
    };
    glsFboUtil.TextureLayerAttachment.prototype = Object.create(glsFboUtil.TextureAttachment.prototype);
    glsFboUtil.TextureLayerAttachment.prototype.constructor = glsFboUtil.TextureLayerAttachment;

    // these are a collection of helper functions for creating various gl textures.
    glsFboUtil.glsup = function() {

        var glInit = function(cfg, gl) {
            if ((cfg.type & glsFboUtil.Config.s_types.TEXTURE_2D) != 0) {
                glInitFlat(cfg, glTarget(cfg, gl), gl);

            } else if ((cfg.type & glsFboUtil.Config.s_types.TEXTURE_CUBE_MAP) != 0) {
                for (var i = gl.TEXTURE_CUBE_MAP_NEGATIVE_X; i <= gl.TEXTURE_CUBE_MAP_POSITIVE_Z; ++i)
                    glInitFlat(cfg, i, gl);
            } else if ((cfg.type & glsFboUtil.Config.s_types.TEXTURE_3D) != 0) {
                glInitLayered(cfg, 2, gl);

            } else if ((cfg.type & glsFboUtil.Config.s_types.TEXTURE_2D_ARRAY) != 0) {
                glInitLayered(cfg, 1, gl);
            }
        };

        var glInitFlat = function(cfg, target, gl) {
            var format = glsFboUtil.transferImageFormat(cfg.internalFormat, gl);
            var w = cfg.width;
            var h = cfg.height;
            for (var level = 0; level < cfg.numLevels; ++level) {
                gl.texImage2D(
                    target, level, cfg.internalFormat.format,
                    w, h, 0, format.format, format.dataType, null
                );
                w = Math.max(1, w / 2);
                h = Math.max(1, h / 2);
            }
        };

        var glInitLayered = function(cfg, depth_divider, gl) {
            var format = glsFboUtil.transferImageFormat(cfg.internalFormat, gl);
            var w = cfg.width;
            var h = cfg.height;
            var depth = cfg.numLayers;
            for (var level = 0; level < cfg.numLevels; ++level) {
                gl.texImage3D(
                    glTarget(cfg, gl), level, cfg.internalFormat.format,
                    w, h, depth, 0, format.format, format.dataType, null
                );
                w = Math.max(1, w / 2);
                h = Math.max(1, h / 2);
                depth = Math.max(1, depth / depth_divider);
            }
        };

        var glCreate = function(cfg, gl) {
            if (!(gl = gl || window.gl)) throw new Error('Invalid gl object');

            if (cfg.type & glsFboUtil.Config.s_types.RENDERBUFFER) {
                var ret = gl.createRenderbuffer();
                gl.bindRenderbuffer(gl.RENDERBUFFER, ret);

                if (cfg.numSamples == 0) {
                    gl.renderbufferStorage(
                        gl.RENDERBUFFER,
                        cfg.internalFormat.format,
                        cfg.width, cfg.height
                    );
                } else {
                    gl.renderbufferStorageMultisample(
                        gl.RENDERBUFFER,
                        cfg.numSamples,
                        cfg.internalFormat.format,
                        cfg.width, cfg.height
                    );
                }
                gl.bindRenderbuffer(gl.RENDERBUFFER, null);

            } else if (cfg.type & glsFboUtil.Config.s_types.TEXTURE) {
                var ret = gl.createTexture();
                gl.bindTexture(glTarget(cfg, gl), ret);
                glInit(cfg, gl);
                gl.bindTexture(glTarget(cfg, gl), null);

            } else {
                throw new Error('Impossible image type');
            }
            return ret;
        };

        var glTarget = function(cfg, gl) {
            if (!(gl = gl || window.gl)) throw new Error('Invalid gl object');
            var mask = (
                glsFboUtil.Config.s_types.RENDERBUFFER |
                glsFboUtil.Config.s_types.TEXTURE_2D |
                glsFboUtil.Config.s_types.TEXTURE_CUBE_MAP |
                glsFboUtil.Config.s_types.TEXTURE_3D |
                glsFboUtil.Config.s_types.TEXTURE_2D_ARRAY
            );
            switch (cfg.type & mask) {
                case glsFboUtil.Config.s_types.RENDERBUFFER: return gl.RENDERBUFFER;
                case glsFboUtil.Config.s_types.TEXTURE_2D: return gl.TEXTURE_2D;
                case glsFboUtil.Config.s_types.TEXTURE_CUBE_MAP: return gl.TEXTURE_CUBE_MAP;
                case glsFboUtil.Config.s_types.TEXTURE_3D: return gl.TEXTURE_3D;
                case glsFboUtil.Config.s_types.TEXTURE_2D_ARRAY: return gl.TEXTURE_2D_ARRAY;
                default: break;
            }
            throw new Error('Impossible image type.');
        };

        var glDelete = function(cfg, img, gl) {
            if (cfg.type & glsFboUtil.Config.s_types.RENDERBUFFER)
                gl.deleteRenderbuffer(img);
            else if (cfg.type & glsFboUtil.Config.s_types.TEXTURE)
                gl.deleteTexture(img);
            else
                throw new Error('Impossible image type');
        };

        return {
            create: glCreate,
            remove: glDelete
        };

    }();

    /**  *generated by script*
    * @param {number} img
    * @return {number}
    */
    glsFboUtil.imageNumSamples = function(img) {
        return (img.numSamples != undefined) ? img.numSamples : 0;
    };

    /**  *generated by script*
    * @param {glsFboUtil.Attachment} att
    * @param {number} attPoint
    * @param {WebGLRenderingContextBase=} gl
    * @throws {Error}
    */
    glsFboUtil.attachAttachment = function(att, attPoint, gl) {
        if (!(gl = gl || window.gl)) throw new Error('Invalid gl object');

        var mask = (
            glsFboUtil.Config.s_types.ATT_RENDERBUFFER |
            glsFboUtil.Config.s_types.ATT_TEXTURE_FLAT |
            glsFboUtil.Config.s_types.ATT_TEXTURE_LAYER
        );

        switch (att.type & mask) {
            case glsFboUtil.Config.s_types.ATT_RENDERBUFFER:
                gl.framebufferRenderbuffer(
                    att.target, attPoint, att.renderbufferTarget, /** @type {WebGLRenderbuffer} */(att.imageName)
                );
                break;
            case glsFboUtil.Config.s_types.ATT_TEXTURE_FLAT:
                gl.framebufferTexture2D(
                    att.target, attPoint, att.texTarget, /** @type {WebGLTexture} */(att.imageName), att.level
                );
                break;
            case glsFboUtil.Config.s_types.ATT_TEXTURE_LAYER:
                gl.framebufferTextureLayer(
                    att.target, attPoint, /** @type {WebGLTexture} */(att.imageName), att.level, att.layer
                );
                break;
            default:
                throw new Error('Impossible attachment type');
        }

    };

    /**  *generated by script*
    * @param {glsFboUtil.Attachment} att
    * @param {WebGLRenderingContextBase=} gl
    * @return {number}
    * @throws {Error}
    */
    glsFboUtil.attachmentType = function(att, gl) {
        if (!(gl = gl || window.gl)) throw new Error('Invalid gl object');

        if (att.type & glsFboUtil.Config.s_types.ATT_RENDERBUFFER) {
            return gl.RENDERBUFFER;
        }
        if (att.type & glsFboUtil.Config.s_types.ATT_TEXTURE) {
            return gl.TEXTURE;
        }
        throw new Error('Impossible attachment type.');

    };

    /**
    * @param {glsFboUtil.Attachment} att
    * @return {number}
    * @throws {Error}
    */
    glsFboUtil.textureLayer = function(att) {
        if (att.type & glsFboUtil.Config.s_types.ATT_TEXTURE_FLAT) return 0;
        if (att.type & glsFboUtil.Config.s_types.ATT_TEXTURE_LAYER) return att.layer;
        throw new Error('Impossible attachment type.');
    };

    /**
    * @param {glsFboUtil.Checker} cctx
    * @param {glsFboUtil.Attachment} att
    * @param {number} attPoint
    * @param {glsFboUtil.Image} image
    * @param {glsFboUtil.FormatDB} db
    * @param {WebGLRenderingContextBase=} gl
    * @throws {Error}
    */
    glsFboUtil.checkAttachmentCompleteness = function(cctx, att, attPoint, image, db, gl) {
        if (!(gl = gl || window.gl)) throw new Error('Invalid gl object');

        // GLES2 4.4.5 / GLES3 4.4.4, "glsFboUtil.Framebuffer attachment completeness"
        if (
            (att.type & glsFboUtil.Config.s_types.ATT_TEXTURE) &&
            (image.type & glsFboUtil.Config.s_types.TEXTURE_LAYERED)
        ) {
            // GLES3: "If the value of FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE is
            // TEXTURE and the value of FRAMEBUFFER_ATTACHMENT_OBJECT_NAME names a
            // three-dimensional texture, then the value of
            // FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER must be smaller than the depth
            // of the texture.
            //
            // GLES3: "If the value of FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE is
            // TEXTURE and the value of FRAMEBUFFER_ATTACHMENT_OBJECT_NAME names a
            // two-dimensional array texture, then the value of
            // FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER must be smaller than the
            // number of layers in the texture.
            cctx.addFBOStatus(
                glsFboUtil.textureLayer(att) < image.numLayers,
                gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT
            );
        }

        // "The width and height of image are non-zero."
        cctx.addFBOStatus(
            image.width > 0 && image.height > 0,
            gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT
        );

        // Check for renderability
        var flags = db.getFormatInfo(image.internalFormat, glsFboUtil.FormatFlags.ANY_FORMAT);

        // If the format does not have the proper renderability flag, the
        // completeness check _must_ fail.
        cctx.addFBOStatus(
            (flags & glsFboUtil.formatFlag(attPoint)) != 0,
            gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT
        );

        // If the format is only optionally renderable, the completeness check _can_ fail.
        cctx.addPotentialFBOStatus(
            (flags & glsFboUtil.FormatFlags.REQUIRED_RENDERABLE) != 0,
            gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT
        );

    };

    // replaces GLS_UNSIZED_FORMATKEY

    /**
    * All params and return types for this function are 32 bit
    * @param {number} format
    * @param {number} type
    * @return {number}
    */
    glsFboUtil.formatkey = function(format, type) {
        // The formatkey value should be 32-bit unsigned int.
        return ((type << 16) >>> 0 | format) & 0xFFFFFFFF;
    };

    /**
    * @enum
    */
    glsFboUtil.FormatFlags = {
        ANY_FORMAT: 0x00,
        COLOR_RENDERABLE: 0x01,
        DEPTH_RENDERABLE: 0x02,
        STENCIL_RENDERABLE: 0x04,
        RENDERBUFFER_VALID: 0x08,
        TEXTURE_VALID: 0x10,
        REQUIRED_RENDERABLE: 0x20 //< Without this, renderability is allowed, not required.
    };

    /**
    * A framebuffer configuration
    * @constructor
    * @param {WebGLRenderingContextBase=} gl
    */
    glsFboUtil.Framebuffer = function(gl) {
        this.m_gl = gl || window.gl;
        this.fbid = 0;

        var fbidCompare = function(obj1, obj2) {
            return obj1._fbid < obj2._fbid;
        };

        this.attachments = /** @type {glsFboUtil.Map<number,glsFboUtil.Attachment>} */(
            new glsFboUtil.Map(glsFboUtil.Map.compareNumber)
        );
        this.textures = /** @type {glsFboUtil.Map<Object,glsFboUtil.Texture>} */(
            new glsFboUtil.Map(fbidCompare)
        );
        this.rbos = /** @type {glsFboUtil.Map<Object,glsFboUtil.Renderbuffer>} */(
            new glsFboUtil.Map(fbidCompare)
        );
    };

    /**
    * @param {number} attPoint
    * @param {glsFboUtil.Attachment} att
    */
    glsFboUtil.Framebuffer.prototype.attach = function(attPoint, att) {
        if (!att) {
            this.attachments.remove(attPoint);
        } else {
            this.attachments.set(attPoint, att);
        }
    };

    /**
    * @param {WebGLTexture} texName
    * @param {glsFboUtil.Texture} texCfg
    */
    glsFboUtil.Framebuffer.prototype.setTexture = function(texName, texCfg) {
        texName._fbid = this.fbid++;
        this.textures.set(texName, texCfg);
    };

    /**
    * @param {WebGLRenderbuffer} rbName
    * @param {glsFboUtil.Renderbuffer} rbCfg
    */
    glsFboUtil.Framebuffer.prototype.setRbo = function(rbName, rbCfg) {
        rbName._fbid = this.fbid++;
        this.rbos.set(rbName, rbCfg);
    };

    /**
    * @param {number} type
    * @param {WebGLObject} imgName
    * @return {glsFboUtil.Image}
    * @throws {Error}
    */
    glsFboUtil.Framebuffer.prototype.getImage = function(type, imgName) {
        switch (type) {
            case this.m_gl.TEXTURE: return this.textures.lookupDefault(/** @type {WebGLTexture} */(imgName), null);
            case this.m_gl.RENDERBUFFER: return this.rbos.lookupDefault(/** @type {WebGLTexture} */(imgName), null);
            default: break;
        }
        throw new Error('Bad image type.');
    };

    /**
    * @constructor
    * @extends {glsFboUtil.Framebuffer}
    * @param {WebGLFramebuffer} fbo
    * @param {number} target
    * @param {WebGLRenderingContextBase=} gl
    */
    glsFboUtil.FboBuilder = function(fbo, target, gl) {
        glsFboUtil.Framebuffer.call(this, gl);

        this.m_gl = gl || window.gl;
        this.m_target = target;
        this.m_configs = [];
        this.m_error = this.m_gl.NO_ERROR;

        this.m_gl.bindFramebuffer(this.m_target, fbo);

    };

    glsFboUtil.FboBuilder.prototype = Object.create(glsFboUtil.Framebuffer.prototype);
    glsFboUtil.FboBuilder.prototype.constructor = glsFboUtil.FboBuilder;

    glsFboUtil.FboBuilder.prototype.deinit = function() {

        var pair;
        for (var i = 0; i < this.textures.length; ++i) {
            pair = this.textures.getIndex(i);
            glsFboUtil.glsup.remove(pair.second, pair.first, this.m_gl);
        }
        this.textures.clear();

        for (var i = 0; i < this.rbos.length; ++i) {
            pair = this.rbos.getIndex(i);
            glsFboUtil.glsup.remove(pair.second, pair.first, this.m_gl);
        }
        this.rbos.clear();

        this.m_gl.bindFramebuffer(this.m_target, null);
/*
        for (var i = 0 ; i < this.m_configs.length ; ++i) {
            delete this.m_configs[i];
        }
//*/
    };

    /**
    * @param {number} attPoint
    * @param {glsFboUtil.Attachment} att
    */
    glsFboUtil.FboBuilder.prototype.glAttach = function(attPoint, att) {
        if (!att) {
            this.m_gl.framebufferRenderbuffer(this.m_target, attPoint, this.m_gl.RENDERBUFFER, null);
        } else {
            glsFboUtil.attachAttachment(att, attPoint, this.m_gl);
        }
        this.checkError();
        this.attach(attPoint, att);
    };

    /**
    * @param {glsFboUtil.Texture} texCfg
    * @return {WebGLTexture}
    */
    glsFboUtil.FboBuilder.prototype.glCreateTexture = function(texCfg) {
        var texName = glsFboUtil.glsup.create(texCfg, this.m_gl);
        this.checkError();
        this.setTexture(texName, texCfg);
        return texName;
    };

    /**  *generated by script*
    * @param {glsFboUtil.Renderbuffer} rbCfg
    * @return {WebGLRenderbuffer}
    */
    glsFboUtil.FboBuilder.prototype.glCreateRbo = function(rbCfg) {
        var rbName = glsFboUtil.glsup.create(rbCfg, this.m_gl);
        this.checkError();
        this.setRbo(rbName, rbCfg);
        return rbName;
    };

    /**
    * @param {function(new:glsFboUtil.Config)} Definition
    * @return {glsFboUtil.Config}
    */
    glsFboUtil.FboBuilder.prototype.makeConfig = function(Definition) {
        var cfg = new Definition();
        this.m_configs.push(cfg);
        return cfg;
    };

    /**
    */
    glsFboUtil.FboBuilder.prototype.checkError = function() {
        var error = this.m_gl.getError();
        if (error != this.m_gl.NO_ERROR && this.m_error == this.m_gl.NO_ERROR) {
            this.m_error = error;
        }
    };

    /**  *generated by script*
    * @return {number}
    */
    glsFboUtil.FboBuilder.prototype.getError = function() {
        return this.m_error;
    };

    glsFboUtil.isFramebufferStatus = function(fboStatus) {
        return gluStrUtil.getFramebufferStatusName(fboStatus) != '';
    }

    glsFboUtil.isErrorCode = function(errorCode) {
        return gluStrUtil.getErrorName(errorCode) != '';
    }

    /**
    * @typedef {funcion(): glsFboUtil.ValidStatusCodes}
    */
    glsFboUtil.ValidStatusCodes = function() {
        this.m_errorCodes = [];
        this.m_errorStatusCodes = [];
        this.m_allowComplete = false;
    };

    glsFboUtil.ValidStatusCodes.prototype.isFBOStatusValid = function(fboStatus) {
        if (fboStatus == gl.FRAMEBUFFER_COMPLETE)
            return this.m_allowComplete;
        else {
            for(var ndx = 0; ndx < this.m_errorStatusCodes.length; ++ndx) {
                if (this.m_errorStatusCodes[ndx] == fboStatus)
                    return true;
            }
            return false;
        }
    };

    glsFboUtil.ValidStatusCodes.prototype.isFBOStatusRequired = function(fboStatus) {
        if (fboStatus == gl.FRAMEBUFFER_COMPLETE)
            return this.m_allowComplete && this.m_errorStatusCodes.length == 0;
        else
            // fboStatus is the only allowed error status and succeeding is forbidden
            return !this.m_allowComplete && this.m_errorStatusCodes.length == 1 && this.m_errorStatusCodes[0] == fboStatus;
    };

    glsFboUtil.ValidStatusCodes.prototype.isErrorCodeValid = function(errorCode) {
        if (errorCode == gl.NO_ERROR)
            return this.m_errorCodes.length == 0;
        else {
            // rule violation exists?
            for (var ndx = 0; ndx < this.m_errorCodes.length; ++ndx) {
                if (this.m_errorCodes[ndx] == errorCode)
                    return true;
            }
            return false;
        }
    };

    glsFboUtil.ValidStatusCodes.prototype.isErrorCodeRequired = function(errorCode) {
        if (this.m_errorCodes.length == 0 && errorCode == gl.NO_ERROR)
            return true;
        else
            // only this error code listed
            return this.m_errorCodes.length == 1 && merrorCodes[0] == errorCode;
    };

    glsFboUtil.ValidStatusCodes.prototype.addErrorCode = function(error) {
        DE_ASSERT(glsFboUtil.isErrorCode(error));
        DE_ASSERT(error != gl.NO_ERROR)
        this.m_errorCodes.push(error);
    };

    glsFboUtil.ValidStatusCodes.prototype.addFBOErrorStatus = function(status) {
        DE_ASSERT(glsFboUtil.isFramebufferStatus(status));
        DE_ASSERT(status != gl.FRAMEBUFFER_COMPLETE)
        this.m_errorStatusCodes.push(status);
    };

    glsFboUtil.ValidStatusCodes.prototype.setAllowComplete = function(b) {
        this.m_allowComplete = b;
    };

    /**
    * @typedef {function(): glsFboUtil.Checker}
    */
    glsFboUtil.CheckerFactory;

    /**
    * @constructor
    * @param {WebGLRenderingContextBase=} gl
    * @throws {Error}
    */
    glsFboUtil.Checker = function(gl) {
        if (!(gl = gl || window.gl)) throw new Error('Invalid gl object');

        this.m_statusCodes = new glsFboUtil.ValidStatusCodes();
        this.m_statusCodes.setAllowComplete(true);

        if (typeof(this.check) != 'function')
            throw new Error('Constructor called on virtual class: glsFboUtil.Checker');
    };

    /**
    * @param {boolean} condition
    * @param {number} error
    */
    glsFboUtil.Checker.prototype.addGLError = function(condition, error) {
        if (!condition) {
            this.m_statusCodes.addErrorCode(error);
            this.m_statusCodes.setAllowComplete(false);
        }
    };

    /**
    * @param {boolean} condition
    * @param {number} error
    */
    glsFboUtil.Checker.prototype.addPotentialGLError = function(condition, error) {
        if (!condition) {
            this.m_statusCodes.addErrorCode(error);
        }
    };

    /**
    * @param {boolean} condition
    * @param {number} status
    */
    glsFboUtil.Checker.prototype.addFBOStatus = function(condition, status) {
        if (!condition) {
            this.m_statusCodes.addFBOErrorStatus(status);
            this.m_statusCodes.setAllowComplete(false);
        }
    };

    /**
    * @param {boolean} condition
    * @param {number} status
    */
    glsFboUtil.Checker.prototype.addPotentialFBOStatus = function(condition, status) {
        if (!condition) {
            this.m_statusCodes.addFBOErrorStatus(status);
        }
    };

    /**
    * @return {Array<number>}
    */
    glsFboUtil.Checker.prototype.getStatusCodes = function () {
        return this.m_statusCodes;
    };

    /**
    * @param {glsFboUtil.ImageFormat} imgFormat
    * @param {WebGLRenderingContextBase=} gl
    * @return {gluTextureUtil.TransferFormat}
    * @throws {Error}
    */
    glsFboUtil.transferImageFormat = function(imgFormat, gl) {
        if (!(gl = gl || window.gl)) throw new Error('Invalid gl object');
        if (imgFormat.unsizedType == gl.NONE)
            return gluTextureUtil.getTransferFormat(gluTextureUtil.mapGLInternalFormat(imgFormat.format));
        else
            return new gluTextureUtil.TransferFormat(imgFormat.format, imgFormat.unsizedType);
    };

    // FormatDB, CheckerFactory

    /**
    * @constructor
    * @param {glsFboUtil.FormatDB} formats
    * @param {glsFboUtil.CheckerFactory} factory
    */
    glsFboUtil.FboVerifier = function(formats, factory) {
        this.m_formats = formats;
        this.m_factory = factory;
    };
    // config::Framebuffer
    glsFboUtil.FboVerifier.prototype.validStatusCodes = function(cfg, gl) {
        if (!(gl = gl || window.gl)) throw new Error('Invalid gl object');

        /** @type {glsFboUtil.Checker} */
        var cctx = this.m_factory();

        for (var id = 0; id < cfg.textures.length; ++id) {
            var flags = this.m_formats.getFormatInfo(cfg.textures.getIndex(id).second.internalFormat, glsFboUtil.FormatFlags.ANY_FORMAT);
            var textureIsValid = (flags & glsFboUtil.FormatFlags.TEXTURE_VALID) != 0;
            cctx.addGLError(textureIsValid, gl.INVALID_ENUM);
            cctx.addGLError(textureIsValid, gl.INVALID_OPERATION);
            cctx.addGLError(textureIsValid, gl.INVALID_VALUE);
        }

        for (var id = 0; id < cfg.rbos.length; ++id) {
            var flags = this.m_formats.getFormatInfo(cfg.rbos.getIndex(id).second.internalFormat, glsFboUtil.FormatFlags.ANY_FORMAT);
            var rboIsValid = (flags & glsFboUtil.FormatFlags.RENDERBUFFER_VALID) != 0;
            cctx.addGLError(rboIsValid, gl.INVALID_ENUM);
        }

        var count = 0;
        for (var index = 0; index < cfg.attachments.length; ++index) {
            var attPoint = cfg.attachments.getIndex(index).first;
            var att = cfg.attachments.getIndex(index).second;
            /** @type {glsFboUtil.Image}*/
            var image = cfg.getImage(glsFboUtil.attachmentType(att, gl), att.imageName);
            glsFboUtil.checkAttachmentCompleteness(cctx, att, attPoint, image, this.m_formats, gl);
            cctx.check(attPoint, att, image);
            ++count;
        }

        // "There is at least one image attached to the framebuffer."
        // TODO: support XXX_framebuffer_no_attachments
        cctx.addFBOStatus(count > 0, gl.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT);

        return cctx.getStatusCodes();

    };

});
