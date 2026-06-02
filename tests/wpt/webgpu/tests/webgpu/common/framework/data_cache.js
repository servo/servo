/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/ /**
 * Utilities to improve the performance of the CTS, by caching data that is
 * expensive to build using a two-level cache (in-memory, pre-computed file).
 */import { assert } from '../util/util.js';





/** Logger is a basic debug logger function */


/**
 * DataCacheNode represents a single cache entry in the LRU DataCache.
 * DataCacheNode is a doubly linked list, so that least-recently-used entries can be removed, and
 * cache hits can move the node to the front of the list.
 */
class DataCacheNode {
  constructor(path, data) {
    this.path = path;
    this.data = data;
  }

  /** insertAfter() re-inserts this node in the doubly-linked list after `prev` */
  insertAfter(prev) {
    this.unlink();
    this.next = prev.next;
    this.prev = prev;
    prev.next = this;
    if (this.next) {
      this.next.prev = this;
    }
  }

  /** unlink() removes this node from the doubly-linked list */
  unlink() {
    const prev = this.prev;
    const next = this.next;
    if (prev) {
      prev.next = next;
    }
    if (next) {
      next.prev = prev;
    }
    this.prev = null;
    this.next = null;
  }

  // The file path this node represents
  // The deserialized data for this node
  prev = null; // The previous node in the doubly-linked list
  next = null; // The next node in the doubly-linked list
}

/** DataCache is an interface to a LRU-cached data store used to hold data cached by path */
export class DataCache {
  constructor() {
    this.lruHeadNode.next = this.lruTailNode;
    this.lruTailNode.prev = this.lruHeadNode;
  }

  /** setDataStore() sets the backing data store used by the data cache */
  setStore(dataStore) {
    this.dataStore = dataStore;
  }

  /** setDebugLogger() sets the verbose logger */
  setDebugLogger(logger) {
    this.debugLogger = logger;
  }

  /**
   * fetch() retrieves cacheable data from the data cache, first checking the
   * in-memory cache, then the data store (if specified), then resorting to
   * building the data and storing it in the cache.
   */
  async fetch(cacheable) {
    {
      // First check the in-memory cache
      const node = this.cache.get(cacheable.path);
      if (node !== undefined) {
        this.log('in-memory cache hit');
        node.insertAfter(this.lruHeadNode);
        return Promise.resolve(node.data);
      }
    }
    this.log('in-memory cache miss');
    // In in-memory cache miss.
    // Next, try the data store.
    if (this.dataStore !== null && !this.unavailableFiles.has(cacheable.path)) {
      let serialized;
      try {
        serialized = await this.dataStore.load(cacheable.path);
        this.log('loaded serialized');
      } catch (err) {
        // not found in data store
        this.log(`failed to load (${cacheable.path}): ${err}`);
        this.unavailableFiles.add(cacheable.path);
      }
      if (serialized !== undefined) {
        this.log(`deserializing`);
        const data = cacheable.deserialize(serialized);
        this.addToCache(cacheable.path, data);
        return data;
      }
    }
    // Not found anywhere. Build the data, and cache for future lookup.
    this.log(`cache: building (${cacheable.path})`);
    const data = await cacheable.build();
    this.addToCache(cacheable.path, data);
    return data;
  }

  /**
   * addToCache() creates a new node for `path` and `data`, inserting the new node at the front of
   * the doubly-linked list. If the number of entries in the cache exceeds this.maxCount, then the
   * least recently used entry is evicted
   * @param path the file path for the data
   * @param data the deserialized data
   */
  addToCache(path, data) {
    if (this.cache.size >= this.maxCount) {
      const toEvict = this.lruTailNode.prev;
      assert(toEvict !== null);
      toEvict.unlink();
      this.cache.delete(toEvict.path);
      this.log(`evicting ${toEvict.path}`);
    }
    const node = new DataCacheNode(path, data);
    node.insertAfter(this.lruHeadNode);
    this.cache.set(path, node);
    this.log(`added ${path}. new count: ${this.cache.size}`);
  }

  log(msg) {
    if (this.debugLogger !== null) {
      this.debugLogger(`DataCache: ${msg}`);
    }
  }

  // Max number of entries in the cache before LRU entries are evicted.
  maxCount = 4;

  cache = new Map();
  lruHeadNode = new DataCacheNode('', null); // placeholder node (no path or data)
  lruTailNode = new DataCacheNode('', null); // placeholder node (no path or data)
  unavailableFiles = new Set();
  dataStore = null;
  debugLogger = null;
}

/** The data cache */
export const dataCache = new DataCache();

/** true if the current process is building the cache */
let isBuildingDataCache = false;

/** @returns true if the data cache is currently being built */
export function getIsBuildingDataCache() {
  return isBuildingDataCache;
}

/** Sets whether the data cache is currently being built */
export function setIsBuildingDataCache(value = true) {
  isBuildingDataCache = value;
}

/**
 * Cacheable is the interface to something that can be stored into the
 * DataCache.
 * The 'npm run gen_cache' tool will look for module-scope variables of this
 * interface, with the name `d`.
 */