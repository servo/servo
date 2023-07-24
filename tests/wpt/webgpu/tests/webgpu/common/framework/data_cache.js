/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/

/** DataCache is an interface to a data store used to hold cached data */
export class DataCache {
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
    // First check the in-memory cache
    let data = this.cache.get(cacheable.path);
    if (data !== undefined) {
      this.log('in-memory cache hit');
      return Promise.resolve(data);
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
        data = cacheable.deserialize(serialized);
        this.cache.set(cacheable.path, data);
        return data;
      }
    }
    // Not found anywhere. Build the data, and cache for future lookup.
    this.log(`cache: building (${cacheable.path})`);
    data = await cacheable.build();
    this.cache.set(cacheable.path, data);
    return data;
  }

  log(msg) {
    if (this.debugLogger !== null) {
      this.debugLogger(`DataCache: ${msg}`);
    }
  }

  cache = new Map();
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
