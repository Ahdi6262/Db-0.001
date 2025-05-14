/**
 * InfiniteMatrix
 * A dynamically customizable matrix system for working with n-dimensional data
 * that seamlessly integrates with the unified database API
 */

class InfiniteMatrix {
  /**
   * Create a new InfiniteMatrix with specified dimensions
   * @param {Object} config - Matrix configuration
   * @param {Array} config.dimensions - Array of dimension specifications
   * @param {Object} config.storage - Storage configuration
   * @param {Object} config.scaling - Scaling behavior configuration
   */
  constructor(config = {}) {
    this.dimensions = config.dimensions || [];
    this.storage = config.storage || {
      type: 'memory',
      options: {}
    };
    this.scaling = config.scaling || {
      auto: true,
      maxSize: Number.MAX_SAFE_INTEGER,
      growthFactor: 2
    };
    this.metadata = {
      created: new Date().toISOString(),
      lastModified: new Date().toISOString(),
      cellCount: 0
    };
    
    // Initialize storage based on type
    this._initializeStorage();
  }
  
  /**
   * Initialize the storage system
   * @private
   */
  _initializeStorage() {
    if (this.storage.type === 'memory') {
      this.data = {};
    } else if (this.storage.type === 'database') {
      // Connect to database specified in storage.options
      this.dbConnection = {
        type: this.storage.options.dbType || 'postgresql',
        collection: this.storage.options.collection || 'infinite_matrix',
        connected: false
      };
      
      // Database connection would be established here
    }
  }
  
  /**
   * Set a value at specified coordinates
   * @param {Array} coordinates - Coordinates in n-dimensional space
   * @param {*} value - Value to store
   * @returns {boolean} - Success status
   */
  set(coordinates, value) {
    if (!Array.isArray(coordinates)) {
      throw new Error('Coordinates must be an array');
    }
    
    if (coordinates.length !== this.dimensions.length) {
      throw new Error(`Expected ${this.dimensions.length} coordinates, got ${coordinates.length}`);
    }
    
    // Check if coordinates are within bounds and auto-expand if needed
    if (this._checkAndExpandDimensions(coordinates)) {
      this.metadata.lastModified = new Date().toISOString();
    }
    
    const key = this._coordinatesToKey(coordinates);
    
    if (this.storage.type === 'memory') {
      // Store in memory
      const valueExists = !!this.data[key];
      this.data[key] = value;
      
      if (!valueExists) {
        this.metadata.cellCount++;
      }
      
      return true;
    } else if (this.storage.type === 'database') {
      // Store in database (implementation would depend on the database adapter)
      // This is a placeholder for actual database storage
      console.log(`Storing ${JSON.stringify(value)} at ${key} in database`);
      return true;
    }
    
    return false;
  }
  
  /**
   * Get a value at specified coordinates
   * @param {Array} coordinates - Coordinates in n-dimensional space
   * @returns {*} - Stored value or undefined if not found
   */
  get(coordinates) {
    if (!Array.isArray(coordinates)) {
      throw new Error('Coordinates must be an array');
    }
    
    if (coordinates.length !== this.dimensions.length) {
      throw new Error(`Expected ${this.dimensions.length} coordinates, got ${coordinates.length}`);
    }
    
    const key = this._coordinatesToKey(coordinates);
    
    if (this.storage.type === 'memory') {
      return this.data[key];
    } else if (this.storage.type === 'database') {
      // Retrieve from database (implementation would depend on the database adapter)
      // This is a placeholder for actual database retrieval
      console.log(`Retrieving value at ${key} from database`);
      return null;
    }
    
    return undefined;
  }
  
  /**
   * Delete a value at specified coordinates
   * @param {Array} coordinates - Coordinates in n-dimensional space
   * @returns {boolean} - Success status
   */
  delete(coordinates) {
    if (!Array.isArray(coordinates)) {
      throw new Error('Coordinates must be an array');
    }
    
    if (coordinates.length !== this.dimensions.length) {
      throw new Error(`Expected ${this.dimensions.length} coordinates, got ${coordinates.length}`);
    }
    
    const key = this._coordinatesToKey(coordinates);
    
    if (this.storage.type === 'memory') {
      const valueExists = !!this.data[key];
      
      if (valueExists) {
        delete this.data[key];
        this.metadata.cellCount--;
        this.metadata.lastModified = new Date().toISOString();
        return true;
      }
      
      return false;
    } else if (this.storage.type === 'database') {
      // Delete from database (implementation would depend on the database adapter)
      // This is a placeholder for actual database deletion
      console.log(`Deleting value at ${key} from database`);
      return true;
    }
    
    return false;
  }
  
  /**
   * Query the matrix for values matching the pattern
   * @param {Object} query - Query object specifying constraints on dimensions
   * @returns {Array} - Array of results with coordinates and values
   */
  query(query) {
    const results = [];
    
    if (this.storage.type === 'memory') {
      // For in-memory storage, we'll just iterate through all values
      for (const key in this.data) {
        const coordinates = this._keyToCoordinates(key);
        let matches = true;
        
        // Check if coordinates match the query constraints
        for (let i = 0; i < this.dimensions.length; i++) {
          const dimension = this.dimensions[i];
          const coord = coordinates[i];
          
          if (query[dimension.name]) {
            const constraint = query[dimension.name];
            
            if (typeof constraint === 'object') {
              // Handle range queries like {min: 0, max: 10}
              if (constraint.min !== undefined && coord < constraint.min) {
                matches = false;
                break;
              }
              
              if (constraint.max !== undefined && coord > constraint.max) {
                matches = false;
                break;
              }
            } else if (constraint !== coord) {
              // Handle exact match
              matches = false;
              break;
            }
          }
        }
        
        if (matches) {
          results.push({
            coordinates,
            value: this.data[key]
          });
        }
      }
    } else if (this.storage.type === 'database') {
      // Query from database (implementation would depend on the database adapter)
      // This is a placeholder for actual database querying
      console.log(`Querying database with ${JSON.stringify(query)}`);
    }
    
    return results;
  }
  
  /**
   * Add a new dimension to the matrix
   * @param {Object} dimension - Dimension specification
   * @returns {number} - Index of the new dimension
   */
  addDimension(dimension) {
    if (!dimension || !dimension.name) {
      throw new Error('Dimension must have a name');
    }
    
    // Check if a dimension with this name already exists
    const existingIndex = this.dimensions.findIndex(d => d.name === dimension.name);
    if (existingIndex >= 0) {
      throw new Error(`Dimension with name '${dimension.name}' already exists`);
    }
    
    // Add the new dimension
    this.dimensions.push({
      name: dimension.name,
      type: dimension.type || 'numeric',
      min: dimension.min !== undefined ? dimension.min : 0,
      max: dimension.max !== undefined ? dimension.max : 100,
      default: dimension.default !== undefined ? dimension.default : 0
    });
    
    this.metadata.lastModified = new Date().toISOString();
    
    // If we have data, we need to update all keys to include the new dimension
    if (this.storage.type === 'memory' && Object.keys(this.data).length > 0) {
      const newData = {};
      const defaultValue = this.dimensions[this.dimensions.length - 1].default;
      
      for (const key in this.data) {
        const coordinates = this._keyToCoordinates(key);
        coordinates.push(defaultValue);
        const newKey = this._coordinatesToKey(coordinates);
        newData[newKey] = this.data[key];
      }
      
      this.data = newData;
    }
    
    return this.dimensions.length - 1;
  }
  
  /**
   * Remove a dimension from the matrix
   * @param {string} dimensionName - Name of the dimension to remove
   * @returns {boolean} - Success status
   */
  removeDimension(dimensionName) {
    const dimensionIndex = this.dimensions.findIndex(d => d.name === dimensionName);
    
    if (dimensionIndex < 0) {
      throw new Error(`Dimension '${dimensionName}' not found`);
    }
    
    // Remove the dimension
    this.dimensions.splice(dimensionIndex, 1);
    
    this.metadata.lastModified = new Date().toISOString();
    
    // If we have data, we need to update all keys to exclude the removed dimension
    if (this.storage.type === 'memory' && Object.keys(this.data).length > 0) {
      const newData = {};
      
      for (const key in this.data) {
        const coordinates = this._keyToCoordinates(key);
        coordinates.splice(dimensionIndex, 1);
        
        if (coordinates.length === 0) {
          // If we've removed the last dimension, there's nowhere to store the data
          continue;
        }
        
        const newKey = this._coordinatesToKey(coordinates);
        newData[newKey] = this.data[key];
      }
      
      this.data = newData;
      this.metadata.cellCount = Object.keys(this.data).length;
    }
    
    return true;
  }
  
  /**
   * Get information about the matrix
   * @returns {Object} - Matrix information
   */
  getInfo() {
    return {
      dimensions: this.dimensions,
      storage: this.storage,
      scaling: this.scaling,
      metadata: this.metadata
    };
  }
  
  /**
   * Resize the matrix to accommodate new dimensions
   * @param {Array} dimensions - New dimension specifications
   * @returns {boolean} - Success status
   */
  resize(dimensions) {
    if (!Array.isArray(dimensions)) {
      throw new Error('Dimensions must be an array');
    }
    
    // Update each dimension
    dimensions.forEach((dimension, index) => {
      if (index < this.dimensions.length) {
        // Update existing dimension
        if (dimension.min !== undefined) {
          this.dimensions[index].min = dimension.min;
        }
        
        if (dimension.max !== undefined) {
          this.dimensions[index].max = dimension.max;
        }
      } else {
        // Add new dimension
        this.addDimension(dimension);
      }
    });
    
    this.metadata.lastModified = new Date().toISOString();
    
    return true;
  }
  
  /**
   * Change the storage type for the matrix
   * @param {string} storageType - New storage type ('memory' or 'database')
   * @param {Object} options - Storage options
   * @returns {Promise<boolean>} - Success status
   */
  async changeStorage(storageType, options = {}) {
    if (storageType === this.storage.type) {
      // Just update options
      this.storage.options = { ...this.storage.options, ...options };
      return true;
    }
    
    // Save current data
    const currentData = this.storage.type === 'memory' ? { ...this.data } : null;
    
    // Update storage configuration
    this.storage = {
      type: storageType,
      options
    };
    
    // Initialize new storage
    this._initializeStorage();
    
    // Migrate data if we have it
    if (currentData) {
      for (const key in currentData) {
        const coordinates = this._keyToCoordinates(key);
        await this.set(coordinates, currentData[key]);
      }
    }
    
    this.metadata.lastModified = new Date().toISOString();
    
    return true;
  }
  
  /**
   * Convert coordinates to a string key
   * @param {Array} coordinates - Coordinates in n-dimensional space
   * @returns {string} - String key
   * @private
   */
  _coordinatesToKey(coordinates) {
    return coordinates.join(':');
  }
  
  /**
   * Convert a string key to coordinates
   * @param {string} key - String key
   * @returns {Array} - Coordinates in n-dimensional space
   * @private
   */
  _keyToCoordinates(key) {
    return key.split(':').map(Number);
  }
  
  /**
   * Check if coordinates are within bounds and expand dimensions if needed
   * @param {Array} coordinates - Coordinates to check
   * @returns {boolean} - Whether dimensions were expanded
   * @private
   */
  _checkAndExpandDimensions(coordinates) {
    let expanded = false;
    
    if (!this.scaling.auto) {
      // Don't auto-expand if not enabled
      return false;
    }
    
    coordinates.forEach((coord, index) => {
      const dimension = this.dimensions[index];
      
      if (coord < dimension.min) {
        dimension.min = coord;
        expanded = true;
      }
      
      if (coord > dimension.max) {
        dimension.max = coord;
        expanded = true;
      }
    });
    
    return expanded;
  }
}

// Export for Node.js and browser environments
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { InfiniteMatrix };
} else {
  // Browser export
  window.InfiniteMatrix = InfiniteMatrix;
}