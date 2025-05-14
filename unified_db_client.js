/**
 * Unified Database Client Library
 * A customizable client for interacting with the Unified Database API
 */

class UnifiedDBClient {
  /**
   * Create a new UnifiedDBClient
   * @param {Object} config - Configuration object
   * @param {string} config.baseUrl - Base URL of the unified database API server
   * @param {string} config.defaultDbType - Default database type to use (postgresql, mongodb, etc.)
   * @param {string} config.defaultDbName - Default database name to use
   * @param {Object} config.auth - Authentication details (if required)
   * @param {boolean} config.debug - Enable debug logging
   */
  constructor(config = {}) {
    this.baseUrl = config.baseUrl || 'http://localhost:8000';
    this.defaultDbType = config.defaultDbType || 'postgresql';
    this.defaultDbName = config.defaultDbName || 'postgres';
    this.auth = config.auth || null;
    this.debug = config.debug || false;
    
    // Initialize adapter mapping for different database types
    this.adapters = {
      postgresql: this._createSqlAdapter(),
      mongodb: this._createDocumentAdapter(),
      redis: this._createKeyValueAdapter(),
      neo4j: this._createGraphAdapter(),
      influxdb: this._createTimeSeriesAdapter(),
      qdrant: this._createVectorAdapter()
    };
    
    this.log(`Initialized UnifiedDBClient for ${this.baseUrl}`);
  }
  
  /**
   * Set a custom adapter for a database type
   * @param {string} dbType - Database type
   * @param {Object} adapter - Custom adapter implementation
   */
  setAdapter(dbType, adapter) {
    this.adapters[dbType] = adapter;
    this.log(`Set custom adapter for ${dbType}`);
    return this;
  }
  
  /**
   * Get available databases
   * @returns {Promise<Array>} - List of available databases
   */
  async getDatabases() {
    try {
      const response = await this._fetch('/databases');
      return response.databases || [];
    } catch (err) {
      this.logError('Failed to get databases', err);
      throw err;
    }
  }
  
  /**
   * Get database health
   * @returns {Promise<Object>} - Health status information
   */
  async getHealth() {
    try {
      return await this._fetch('/health');
    } catch (err) {
      this.logError('Failed to get health', err);
      throw err;
    }
  }
  
  /**
   * Get a database interface for the specified database type
   * @param {string} dbType - Database type
   * @param {string} dbName - Database name
   * @returns {Object} - Database interface
   */
  database(dbType = this.defaultDbType, dbName = this.defaultDbName) {
    if (!this.adapters[dbType]) {
      throw new Error(`Unsupported database type: ${dbType}`);
    }
    
    return this.adapters[dbType](dbType, dbName);
  }
  
  /**
   * Create a matrix interface for working with n-dimensional data
   * @param {string} name - Matrix name
   * @returns {Object} - Matrix interface
   */
  matrix(name) {
    const self = this;
    return {
      /**
       * Get matrix information
       * @returns {Promise<Object>} - Matrix information
       */
      async getInfo() {
        try {
          return await self._fetch(`/matrix/${name}`);
        } catch (err) {
          self.logError(`Failed to get matrix info for ${name}`, err);
          throw err;
        }
      },
      
      /**
       * Set a value at specified coordinates
       * @param {Array} coordinates - Coordinates in n-dimensional space
       * @param {*} value - Value to store
       * @returns {Promise<Object>} - Result
       */
      async setValue(coordinates, value) {
        try {
          return await self._fetch(`/matrix/${name}/value`, {
            method: 'POST',
            body: JSON.stringify({ coordinates, value })
          });
        } catch (err) {
          self.logError(`Failed to set value in matrix ${name}`, err);
          throw err;
        }
      },
      
      /**
       * Get a value at specified coordinates
       * @param {Array} coordinates - Coordinates in n-dimensional space
       * @returns {Promise<Object>} - Retrieved value
       */
      async getValue(coordinates) {
        try {
          return await self._fetch(`/matrix/${name}/value?coordinates=${coordinates.join(',')}`);
        } catch (err) {
          self.logError(`Failed to get value from matrix ${name}`, err);
          throw err;
        }
      },
      
      /**
       * Query the matrix for values matching the pattern
       * @param {Object} query - Query object specifying constraints on dimensions
       * @returns {Promise<Array>} - Query results
       */
      async query(query) {
        try {
          return await self._fetch(`/matrix/${name}/query`, {
            method: 'POST',
            body: JSON.stringify({ query })
          });
        } catch (err) {
          self.logError(`Failed to query matrix ${name}`, err);
          throw err;
        }
      },
      
      /**
       * Add a dimension to the matrix
       * @param {Object} dimension - Dimension specification
       * @returns {Promise<Object>} - Result
       */
      async addDimension(dimension) {
        try {
          return await self._fetch(`/matrix/${name}/dimensions`, {
            method: 'POST',
            body: JSON.stringify(dimension)
          });
        } catch (err) {
          self.logError(`Failed to add dimension to matrix ${name}`, err);
          throw err;
        }
      },
      
      /**
       * Remove a dimension from the matrix
       * @param {string} dimensionName - Name of the dimension to remove
       * @returns {Promise<Object>} - Result
       */
      async removeDimension(dimensionName) {
        try {
          return await self._fetch(`/matrix/${name}/dimensions/${dimensionName}`, {
            method: 'DELETE'
          });
        } catch (err) {
          self.logError(`Failed to remove dimension from matrix ${name}`, err);
          throw err;
        }
      },
      
      /**
       * Resize the matrix
       * @param {Array} dimensions - New dimension specifications
       * @returns {Promise<Object>} - Result
       */
      async resize(dimensions) {
        try {
          return await self._fetch(`/matrix/${name}/resize`, {
            method: 'PATCH',
            body: JSON.stringify({ dimensions })
          });
        } catch (err) {
          self.logError(`Failed to resize matrix ${name}`, err);
          throw err;
        }
      },
      
      /**
       * Change the storage type for the matrix
       * @param {string} storageType - New storage type ('memory' or 'database')
       * @param {Object} options - Storage options
       * @returns {Promise<Object>} - Result
       */
      async changeStorage(storageType, options = {}) {
        try {
          return await self._fetch(`/matrix/${name}/storage`, {
            method: 'PATCH',
            body: JSON.stringify({ storageType, options })
          });
        } catch (err) {
          self.logError(`Failed to change storage for matrix ${name}`, err);
          throw err;
        }
      }
    };
  }
  
  /**
   * Create a SQL adapter (PostgreSQL, MySQL, etc.)
   * @returns {Function} - Adapter function
   * @private
   */
  _createSqlAdapter() {
    const self = this;
    return function(dbType, dbName) {
      return {
        /**
         * Create a record in a collection
         * @param {string} collection - Collection/table name
         * @param {Object} data - Record data
         * @returns {Promise<Object>} - Created record
         */
        async createRecord(collection, data) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/records`, {
              method: 'POST',
              body: JSON.stringify({ collection, data })
            });
          } catch (err) {
            self.logError(`Failed to create record in ${collection}`, err);
            throw err;
          }
        },
        
        /**
         * Get a record from a collection
         * @param {string} collection - Collection/table name
         * @param {string} id - Record ID
         * @returns {Promise<Object>} - Retrieved record
         */
        async getRecord(collection, id) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/records/${id}?collection=${collection}`);
          } catch (err) {
            self.logError(`Failed to get record ${id} from ${collection}`, err);
            throw err;
          }
        },
        
        /**
         * Update a record in a collection
         * @param {string} collection - Collection/table name
         * @param {string} id - Record ID
         * @param {Object} data - Updated record data
         * @returns {Promise<Object>} - Updated record
         */
        async updateRecord(collection, id, data) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/records/${id}`, {
              method: 'PUT',
              body: JSON.stringify({ collection, data })
            });
          } catch (err) {
            self.logError(`Failed to update record ${id} in ${collection}`, err);
            throw err;
          }
        },
        
        /**
         * Delete a record from a collection
         * @param {string} collection - Collection/table name
         * @param {string} id - Record ID
         * @returns {Promise<boolean>} - Success status
         */
        async deleteRecord(collection, id) {
          try {
            await self._fetch(`/databases/${dbType}/${dbName}/records/${id}?collection=${collection}`, {
              method: 'DELETE'
            });
            return true;
          } catch (err) {
            self.logError(`Failed to delete record ${id} from ${collection}`, err);
            throw err;
          }
        },
        
        /**
         * Query records in a collection
         * @param {string} collection - Collection/table name
         * @param {Object} query - Query criteria
         * @param {Object} options - Query options (limit, offset, sortBy, sortOrder)
         * @returns {Promise<Array>} - Query results
         */
        async queryRecords(collection, query = {}, options = {}) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/query`, {
              method: 'POST',
              body: JSON.stringify({
                collection,
                query,
                limit: options.limit,
                offset: options.offset,
                sortBy: options.sortBy,
                sortOrder: options.sortOrder
              })
            });
          } catch (err) {
            self.logError(`Failed to query records in ${collection}`, err);
            throw err;
          }
        },
        
        /**
         * Execute a raw SQL query (specific to SQL databases)
         * @param {string} sql - SQL query
         * @param {Array} params - Query parameters
         * @returns {Promise<Object>} - Query results
         */
        async executeRawQuery(sql, params = []) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/execute`, {
              method: 'POST',
              body: JSON.stringify({ sql, params })
            });
          } catch (err) {
            self.logError('Failed to execute raw query', err);
            throw err;
          }
        }
      };
    };
  }
  
  /**
   * Create a document adapter (MongoDB, CouchDB, etc.)
   * @returns {Function} - Adapter function
   * @private
   */
  _createDocumentAdapter() {
    // The basic CRUD operations are the same as SQL
    const sqlAdapter = this._createSqlAdapter();
    const self = this;
    
    return function(dbType, dbName) {
      const adapter = sqlAdapter(dbType, dbName);
      
      // Add document-specific methods
      return Object.assign({}, adapter, {
        /**
         * Execute an aggregation pipeline (specific to document databases)
         * @param {string} collection - Collection name
         * @param {Array} pipeline - Aggregation pipeline
         * @returns {Promise<Array>} - Aggregation results
         */
        async aggregate(collection, pipeline) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/aggregate`, {
              method: 'POST',
              body: JSON.stringify({ collection, pipeline })
            });
          } catch (err) {
            self.logError(`Failed to execute aggregate on ${collection}`, err);
            throw err;
          }
        },
        
        /**
         * Create or update a document (upsert)
         * @param {string} collection - Collection name
         * @param {Object} filter - Filter to find existing document
         * @param {Object} data - Document data
         * @returns {Promise<Object>} - Upserted document
         */
        async upsertRecord(collection, filter, data) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/upsert`, {
              method: 'POST',
              body: JSON.stringify({ collection, filter, data })
            });
          } catch (err) {
            self.logError(`Failed to upsert record in ${collection}`, err);
            throw err;
          }
        }
      });
    };
  }
  
  /**
   * Create a key-value adapter (Redis, etc.)
   * @returns {Function} - Adapter function
   * @private
   */
  _createKeyValueAdapter() {
    const self = this;
    return function(dbType, dbName) {
      return {
        /**
         * Set a key-value pair
         * @param {string} key - Key
         * @param {*} value - Value
         * @param {Object} options - Options (expiry, etc.)
         * @returns {Promise<boolean>} - Success status
         */
        async set(key, value, options = {}) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/kv`, {
              method: 'POST',
              body: JSON.stringify({ key, value, options })
            });
          } catch (err) {
            self.logError(`Failed to set key ${key}`, err);
            throw err;
          }
        },
        
        /**
         * Get a value by key
         * @param {string} key - Key
         * @returns {Promise<*>} - Retrieved value
         */
        async get(key) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/kv/${key}`);
          } catch (err) {
            self.logError(`Failed to get key ${key}`, err);
            throw err;
          }
        },
        
        /**
         * Delete a key
         * @param {string} key - Key
         * @returns {Promise<boolean>} - Success status
         */
        async delete(key) {
          try {
            await self._fetch(`/databases/${dbType}/${dbName}/kv/${key}`, {
              method: 'DELETE'
            });
            return true;
          } catch (err) {
            self.logError(`Failed to delete key ${key}`, err);
            throw err;
          }
        },
        
        /**
         * Increment a numeric value
         * @param {string} key - Key
         * @param {number} by - Increment amount
         * @returns {Promise<number>} - Updated value
         */
        async increment(key, by = 1) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/kv/increment`, {
              method: 'POST',
              body: JSON.stringify({ key, by })
            });
          } catch (err) {
            self.logError(`Failed to increment key ${key}`, err);
            throw err;
          }
        }
      };
    };
  }
  
  /**
   * Create a graph adapter (Neo4j, etc.)
   * @returns {Function} - Adapter function
   * @private
   */
  _createGraphAdapter() {
    const self = this;
    return function(dbType, dbName) {
      return {
        /**
         * Create a node
         * @param {string} label - Node label
         * @param {Object} properties - Node properties
         * @returns {Promise<Object>} - Created node
         */
        async createNode(label, properties) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/nodes`, {
              method: 'POST',
              body: JSON.stringify({ label, properties })
            });
          } catch (err) {
            self.logError(`Failed to create node with label ${label}`, err);
            throw err;
          }
        },
        
        /**
         * Create a relationship between nodes
         * @param {string} startNodeId - Start node ID
         * @param {string} endNodeId - End node ID
         * @param {string} type - Relationship type
         * @param {Object} properties - Relationship properties
         * @returns {Promise<Object>} - Created relationship
         */
        async createRelationship(startNodeId, endNodeId, type, properties = {}) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/relationships`, {
              method: 'POST',
              body: JSON.stringify({ startNodeId, endNodeId, type, properties })
            });
          } catch (err) {
            self.logError(`Failed to create relationship between ${startNodeId} and ${endNodeId}`, err);
            throw err;
          }
        },
        
        /**
         * Execute a Cypher query
         * @param {string} query - Cypher query
         * @param {Object} params - Query parameters
         * @returns {Promise<Object>} - Query results
         */
        async query(query, params = {}) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/cypher`, {
              method: 'POST',
              body: JSON.stringify({ query, params })
            });
          } catch (err) {
            self.logError('Failed to execute Cypher query', err);
            throw err;
          }
        }
      };
    };
  }
  
  /**
   * Create a time series adapter (InfluxDB, etc.)
   * @returns {Function} - Adapter function
   * @private
   */
  _createTimeSeriesAdapter() {
    const self = this;
    return function(dbType, dbName) {
      return {
        /**
         * Write time series data points
         * @param {string} measurement - Measurement name
         * @param {Array} points - Data points
         * @param {Object} options - Write options
         * @returns {Promise<boolean>} - Success status
         */
        async writePoints(measurement, points, options = {}) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/points`, {
              method: 'POST',
              body: JSON.stringify({ measurement, points, options })
            });
          } catch (err) {
            self.logError(`Failed to write points to ${measurement}`, err);
            throw err;
          }
        },
        
        /**
         * Query time series data
         * @param {string} query - Query string (InfluxQL, Flux, etc.)
         * @param {Object} options - Query options
         * @returns {Promise<Array>} - Query results
         */
        async query(query, options = {}) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/query`, {
              method: 'POST',
              body: JSON.stringify({ query, options })
            });
          } catch (err) {
            self.logError('Failed to query time series data', err);
            throw err;
          }
        }
      };
    };
  }
  
  /**
   * Create a vector adapter (Qdrant, etc.)
   * @returns {Function} - Adapter function
   * @private
   */
  _createVectorAdapter() {
    const self = this;
    return function(dbType, dbName) {
      return {
        /**
         * Create a collection
         * @param {string} collection - Collection name
         * @param {Object} config - Collection configuration
         * @returns {Promise<Object>} - Created collection
         */
        async createCollection(collection, config) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/collections`, {
              method: 'POST',
              body: JSON.stringify({ collection, config })
            });
          } catch (err) {
            self.logError(`Failed to create collection ${collection}`, err);
            throw err;
          }
        },
        
        /**
         * Add vectors to a collection
         * @param {string} collection - Collection name
         * @param {Array} vectors - Vectors to add
         * @returns {Promise<Object>} - Result
         */
        async addVectors(collection, vectors) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/collections/${collection}/vectors`, {
              method: 'POST',
              body: JSON.stringify({ vectors })
            });
          } catch (err) {
            self.logError(`Failed to add vectors to ${collection}`, err);
            throw err;
          }
        },
        
        /**
         * Search for similar vectors
         * @param {string} collection - Collection name
         * @param {Array} vector - Query vector
         * @param {number} limit - Maximum number of results
         * @returns {Promise<Array>} - Search results
         */
        async search(collection, vector, limit = 10) {
          try {
            return await self._fetch(`/databases/${dbType}/${dbName}/collections/${collection}/search`, {
              method: 'POST',
              body: JSON.stringify({ vector, limit })
            });
          } catch (err) {
            self.logError(`Failed to search vectors in ${collection}`, err);
            throw err;
          }
        }
      };
    };
  }
  
  /**
   * Make a fetch request to the API
   * @param {string} path - API path
   * @param {Object} options - Fetch options
   * @returns {Promise<*>} - Response data
   * @private
   */
  async _fetch(path, options = {}) {
    const url = `${this.baseUrl}${path}`;
    
    // Set default headers
    const headers = {
      'Content-Type': 'application/json',
      ...options.headers
    };
    
    // Add authentication if provided
    if (this.auth) {
      if (this.auth.type === 'bearer') {
        headers['Authorization'] = `Bearer ${this.auth.token}`;
      } else if (this.auth.type === 'basic') {
        const credentials = Buffer.from(`${this.auth.username}:${this.auth.password}`).toString('base64');
        headers['Authorization'] = `Basic ${credentials}`;
      }
    }
    
    this.log(`Fetching: ${options.method || 'GET'} ${url}`);
    
    try {
      const response = await fetch(url, {
        ...options,
        headers
      });
      
      // Handle non-2xx responses
      if (!response.ok) {
        const error = await response.json().catch(() => ({ message: 'Unknown error' }));
        throw new Error(error.message || `Request failed with status ${response.status}`);
      }
      
      // Handle empty response
      if (response.status === 204) {
        return null;
      }
      
      return await response.json();
    } catch (err) {
      this.logError(`Fetch error: ${err.message}`, err);
      throw err;
    }
  }
  
  /**
   * Log a message if debug is enabled
   * @param {string} message - Message to log
   * @private
   */
  log(message) {
    if (this.debug) {
      console.log(`[UnifiedDBClient] ${message}`);
    }
  }
  
  /**
   * Log an error
   * @param {string} message - Error message
   * @param {Error} error - Error object
   * @private
   */
  logError(message, error) {
    if (this.debug) {
      console.error(`[UnifiedDBClient] ERROR: ${message}`, error);
    }
  }
}

// Export for Node.js and browser environments
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { UnifiedDBClient };
} else {
  // Browser export
  window.UnifiedDBClient = UnifiedDBClient;
}