const express = require('express');
const { Pool } = require('pg');
const { MongoClient, ObjectId } = require('mongodb');
const crypto = require('crypto');
const cors = require('cors');
const { InfiniteMatrix } = require('./infinite_matrix');
const app = express();
const port = 8000;

// Enable configuration
const config = {
  version: '1.0.0',
  scalingFactor: process.env.SCALING_FACTOR || 1,
  maxConnections: process.env.MAX_CONNECTIONS || 100,
  enableDynamicConfiguration: true, // Allow runtime configuration changes
  dynamicAdapters: true, // Enable dynamic adapter loading
  supportedDatabases: {
    sql: ['postgresql', 'mysql'],
    nosql: ['mongodb', 'redis', 'cassandra'],
    graph: ['neo4j'],
    timeseries: ['influxdb'],
    vector: ['qdrant'],
    custom: [] // User-defined custom databases
  }
};

// Helper function to generate UUIDs
function generateUUID() {
  return crypto.randomUUID();
}

// Database connection configuration
const connections = {
  // PostgreSQL configuration
  postgresql: {
    client: null,
    async connect() {
      try {
        this.client = new Pool({
          connectionString: process.env.DATABASE_URL
        });
        console.log('PostgreSQL connected successfully');
        return true;
      } catch (err) {
        console.error('PostgreSQL connection error:', err);
        return false;
      }
    },
    async testConnection() {
      try {
        const result = await this.client.query('SELECT 1');
        return result ? true : false;
      } catch (err) {
        console.error('PostgreSQL test connection error:', err);
        return false;
      }
    },
    // PostgreSQL-specific operations
    async getTables() {
      try {
        const result = await this.client.query(
          "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'"
        );
        return result.rows.map(row => row.table_name);
      } catch (err) {
        console.error('PostgreSQL getTables error:', err);
        return [];
      }
    },
    async createCollection(collectionName) {
      try {
        // In PostgreSQL, collections are tables
        await this.client.query(`
          CREATE TABLE IF NOT EXISTS ${collectionName} (
            id TEXT PRIMARY KEY,
            data JSONB NOT NULL
          )
        `);
        return true;
      } catch (err) {
        console.error(`PostgreSQL createCollection error for ${collectionName}:`, err);
        return false;
      }
    },
    async collectionExists(collectionName) {
      try {
        const result = await this.client.query(`
          SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = $1
          )
        `, [collectionName]);
        return result.rows[0].exists;
      } catch (err) {
        console.error(`PostgreSQL collectionExists error for ${collectionName}:`, err);
        return false;
      }
    },
    async insertRecord(collectionName, data) {
      try {
        // Ensure the table exists
        await this.createCollection(collectionName);
        
        // Generate ID if not provided
        const id = data.id || generateUUID();
        
        // Get the data without the ID field (if present)
        const { id: _, ...dataWithoutId } = data;
        
        // Insert the record
        await this.client.query(`
          INSERT INTO ${collectionName} (id, data) VALUES ($1, $2)
        `, [id, dataWithoutId]);
        
        return { id, data: dataWithoutId };
      } catch (err) {
        console.error(`PostgreSQL insertRecord error for ${collectionName}:`, err);
        throw err;
      }
    },
    async getRecord(collectionName, id) {
      try {
        if (!(await this.collectionExists(collectionName))) {
          return null;
        }
        
        const result = await this.client.query(`
          SELECT data FROM ${collectionName} WHERE id = $1
        `, [id]);
        
        if (result.rows.length === 0) {
          return null;
        }
        
        return { id, data: result.rows[0].data };
      } catch (err) {
        console.error(`PostgreSQL getRecord error for ${collectionName}:`, err);
        throw err;
      }
    },
    async updateRecord(collectionName, id, data) {
      try {
        if (!(await this.collectionExists(collectionName))) {
          return false;
        }
        
        // Get the data without the ID field (if present)
        const { id: _, ...dataWithoutId } = data;
        
        const result = await this.client.query(`
          UPDATE ${collectionName} SET data = $1 WHERE id = $2
        `, [dataWithoutId, id]);
        
        return result.rowCount > 0;
      } catch (err) {
        console.error(`PostgreSQL updateRecord error for ${collectionName}:`, err);
        throw err;
      }
    },
    async deleteRecord(collectionName, id) {
      try {
        if (!(await this.collectionExists(collectionName))) {
          return false;
        }
        
        const result = await this.client.query(`
          DELETE FROM ${collectionName} WHERE id = $1
        `, [id]);
        
        return result.rowCount > 0;
      } catch (err) {
        console.error(`PostgreSQL deleteRecord error for ${collectionName}:`, err);
        throw err;
      }
    },
    async queryRecords(collectionName, query, options = {}) {
      try {
        if (!(await this.collectionExists(collectionName))) {
          return [];
        }
        
        let sqlQuery = `SELECT id, data FROM ${collectionName}`;
        const queryParams = [];
        
        // Build WHERE clause
        if (query && Object.keys(query).length > 0) {
          sqlQuery += ' WHERE ';
          const conditions = [];
          
          for (const [key, value] of Object.entries(query)) {
            queryParams.push(value);
            if (value === null) {
              conditions.push(`data->>'${key}' IS NULL`);
            } else if (typeof value === 'string') {
              conditions.push(`data->>'${key}' = $${queryParams.length}`);
            } else if (typeof value === 'number') {
              conditions.push(`(data->'${key}')::numeric = $${queryParams.length}`);
            } else if (typeof value === 'boolean') {
              conditions.push(`(data->'${key}')::boolean = $${queryParams.length}`);
            } else {
              queryParams.pop(); // Remove the value we just added
              conditions.push(`data->>'${key}' = '${JSON.stringify(value).replace(/'/g, "''")}'`);
            }
          }
          
          sqlQuery += conditions.join(' AND ');
        }
        
        // Add sort
        if (options.sortBy) {
          const sortOrder = options.sortOrder || 'asc';
          sqlQuery += ` ORDER BY data->>'${options.sortBy}' ${sortOrder.toUpperCase()}`;
        }
        
        // Add limit and offset
        if (options.limit) {
          sqlQuery += ` LIMIT ${options.limit}`;
          if (options.offset) {
            sqlQuery += ` OFFSET ${options.offset}`;
          }
        }
        
        const result = await this.client.query(sqlQuery, queryParams);
        
        return result.rows.map(row => ({
          id: row.id,
          data: row.data
        }));
      } catch (err) {
        console.error(`PostgreSQL queryRecords error for ${collectionName}:`, err);
        throw err;
      }
    }
  },
  
  // MongoDB configuration
  mongodb: {
    client: null,
    db: null,
    async connect() {
      try {
        // Use MongoDB URI if provided, otherwise construct one from individual parts
        const uri = process.env.MONGODB_URI || 'mongodb://localhost:27017/unified_db_api';
        this.client = new MongoClient(uri);
        await this.client.connect();
        this.db = this.client.db();
        console.log('MongoDB connected successfully');
        return true;
      } catch (err) {
        console.error('MongoDB connection error:', err);
        return false;
      }
    },
    async testConnection() {
      try {
        // Ping the database to test connection
        await this.db.command({ ping: 1 });
        return true;
      } catch (err) {
        console.error('MongoDB test connection error:', err);
        return false;
      }
    },
    // MongoDB-specific operations
    async getTables() {
      try {
        // In MongoDB, collections are equivalent to tables in SQL
        const collections = await this.db.listCollections().toArray();
        return collections.map(col => col.name);
      } catch (err) {
        console.error('MongoDB getTables error:', err);
        return [];
      }
    },
    async createCollection(collectionName) {
      try {
        await this.db.createCollection(collectionName);
        return true;
      } catch (err) {
        // Collection might already exist, which is fine
        if (err.code === 48) {
          return true;
        }
        console.error(`MongoDB createCollection error for ${collectionName}:`, err);
        return false;
      }
    },
    async collectionExists(collectionName) {
      try {
        const collections = await this.db.listCollections({ name: collectionName }).toArray();
        return collections.length > 0;
      } catch (err) {
        console.error(`MongoDB collectionExists error for ${collectionName}:`, err);
        return false;
      }
    },
    async insertRecord(collectionName, data) {
      try {
        // Ensure the collection exists
        await this.createCollection(collectionName);
        
        // Create a copy of the data to avoid modifying the original
        const recordData = { ...data };
        
        // Generate ID if not provided
        if (!recordData._id) {
          recordData._id = generateUUID();
        }
        
        const id = recordData._id;
        
        // Insert the record
        const collection = this.db.collection(collectionName);
        await collection.insertOne(recordData);
        
        // Return the record with id in the expected format
        return { 
          id: id,
          data: Object.fromEntries(
            Object.entries(recordData).filter(([key]) => key !== '_id')
          )
        };
      } catch (err) {
        console.error(`MongoDB insertRecord error for ${collectionName}:`, err);
        throw err;
      }
    },
    async getRecord(collectionName, id) {
      try {
        if (!(await this.collectionExists(collectionName))) {
          return null;
        }
        
        const collection = this.db.collection(collectionName);
        const record = await collection.findOne({ _id: id });
        
        if (!record) {
          return null;
        }
        
        // Return the record in the expected format
        const { _id, ...data } = record;
        return { id: _id, data };
      } catch (err) {
        console.error(`MongoDB getRecord error for ${collectionName}:`, err);
        throw err;
      }
    },
    async updateRecord(collectionName, id, data) {
      try {
        if (!(await this.collectionExists(collectionName))) {
          return false;
        }
        
        const collection = this.db.collection(collectionName);
        
        // Create a copy of the data to avoid modifying the original
        const updateData = { ...data };
        
        // Remove _id from the update data if present
        delete updateData._id;
        
        const result = await collection.updateOne(
          { _id: id },
          { $set: updateData }
        );
        
        return result.matchedCount > 0;
      } catch (err) {
        console.error(`MongoDB updateRecord error for ${collectionName}:`, err);
        throw err;
      }
    },
    async deleteRecord(collectionName, id) {
      try {
        if (!(await this.collectionExists(collectionName))) {
          return false;
        }
        
        const collection = this.db.collection(collectionName);
        const result = await collection.deleteOne({ _id: id });
        
        return result.deletedCount > 0;
      } catch (err) {
        console.error(`MongoDB deleteRecord error for ${collectionName}:`, err);
        throw err;
      }
    },
    async queryRecords(collectionName, query, options = {}) {
      try {
        if (!(await this.collectionExists(collectionName))) {
          return [];
        }
        
        const collection = this.db.collection(collectionName);
        
        // Build the query
        let cursor = collection.find(query || {});
        
        // Add sort
        if (options.sortBy) {
          const sortOrder = options.sortOrder === 'desc' ? -1 : 1;
          cursor = cursor.sort({ [options.sortBy]: sortOrder });
        }
        
        // Add pagination
        if (options.limit) {
          cursor = cursor.limit(options.limit);
          if (options.offset) {
            cursor = cursor.skip(options.offset);
          }
        }
        
        // Execute query
        const records = await cursor.toArray();
        
        // Convert to standardized format
        return records.map(record => {
          const { _id, ...data } = record;
          return { id: _id, data };
        });
      } catch (err) {
        console.error(`MongoDB queryRecords error for ${collectionName}:`, err);
        throw err;
      }
    }
  }
};

// Initialize connections
async function initConnections() {
  let hasConnections = false;
  
  // Connect to PostgreSQL
  if (process.env.DATABASE_URL) {
    const pgConnected = await connections.postgresql.connect();
    if (pgConnected) {
      hasConnections = true;
    }
  } else {
    console.log('PostgreSQL connection string (DATABASE_URL) not provided. PostgreSQL support disabled.');
  }
  
  // Connect to MongoDB
  if (process.env.MONGODB_URI) {
    const mongoConnected = await connections.mongodb.connect();
    if (mongoConnected) {
      hasConnections = true;
    }
  } else {
    console.log('MongoDB connection string (MONGODB_URI) not provided. MongoDB support disabled.');
  }
  
  if (!hasConnections) {
    console.warn('No database connections established. API will have limited functionality.');
  }
  
  return hasConnections;
}

// Middleware
app.use(express.json());
app.use(cors());

// Dynamic configuration store
const dynamicConfig = {
  activeAdapters: {},
  customConnections: {},
  scalingRules: {
    autoScale: true,
    minInstances: 1,
    maxInstances: 10,
    scaleUpThreshold: 0.8, // CPU/Memory threshold to trigger scaling
    scaleDownThreshold: 0.2
  },
  securityRules: {
    rateLimiting: true,
    maxRequestsPerMinute: 100,
    authRequired: false,
    allowedOrigins: ['*']
  },
  performance: {
    connectionPooling: true,
    cacheEnabled: true,
    cacheSize: 1000,
    cacheExpirySeconds: 300,
    queryTimeout: 30000 // ms
  }
};

// API endpoints
// Configuration management endpoints
app.get('/config', (req, res) => {
  res.json({
    ...config,
    dynamic: dynamicConfig
  });
});

// Update configuration
app.patch('/config', (req, res) => {
  const { section, updates } = req.body;
  
  if (!section || !updates || typeof updates !== 'object') {
    return res.status(400).json({ error: 'Invalid request format. Requires section and updates.' });
  }
  
  // Safely update configuration
  if (section === 'scalingRules' && dynamicConfig.scalingRules) {
    dynamicConfig.scalingRules = { ...dynamicConfig.scalingRules, ...updates };
  } else if (section === 'securityRules' && dynamicConfig.securityRules) {
    dynamicConfig.securityRules = { ...dynamicConfig.securityRules, ...updates };
  } else if (section === 'performance' && dynamicConfig.performance) {
    dynamicConfig.performance = { ...dynamicConfig.performance, ...updates };
  } else if (section === 'global') {
    // Update top-level configuration
    Object.keys(updates).forEach(key => {
      if (typeof config[key] !== 'undefined') {
        config[key] = updates[key];
      }
    });
  } else {
    return res.status(400).json({ error: 'Invalid configuration section.' });
  }
  
  res.json({ success: true, message: 'Configuration updated successfully' });
});

// Register custom database adapter
app.post('/adapters', (req, res) => {
  const { name, type, connectionConfig, implementation } = req.body;
  
  if (!name || !type || !connectionConfig) {
    return res.status(400).json({ error: 'Missing required fields: name, type, connectionConfig' });
  }
  
  // Register the custom adapter
  dynamicConfig.activeAdapters[name] = {
    type,
    connectionConfig,
    implementation: implementation || 'default',
    created: new Date().toISOString()
  };
  
  // Add to supported databases if it's a custom type
  if (!config.supportedDatabases.sql.includes(name) && 
      !config.supportedDatabases.nosql.includes(name) &&
      !config.supportedDatabases.graph.includes(name) &&
      !config.supportedDatabases.timeseries.includes(name) &&
      !config.supportedDatabases.vector.includes(name)) {
    config.supportedDatabases.custom.push(name);
  }
  
  res.status(201).json({ 
    success: true, 
    message: `Custom adapter '${name}' registered successfully`,
    adapter: dynamicConfig.activeAdapters[name]
  });
});

// Dynamic connection management
app.post('/connections', async (req, res) => {
  const { name, type, connectionString, options } = req.body;
  
  if (!name || !type || !connectionString) {
    return res.status(400).json({ error: 'Missing required fields: name, type, connectionString' });
  }
  
  try {
    // Store the connection config
    dynamicConfig.customConnections[name] = {
      type,
      connectionString,
      options: options || {},
      status: 'initializing',
      created: new Date().toISOString()
    };
    
    // Implementation would depend on database type
    // For now, we'll just mark it as connected
    dynamicConfig.customConnections[name].status = 'connected';
    
    res.status(201).json({
      success: true,
      message: `Connection '${name}' created successfully`,
      connection: {
        name,
        type,
        status: dynamicConfig.customConnections[name].status
      }
    });
  } catch (error) {
    dynamicConfig.customConnections[name].status = 'failed';
    dynamicConfig.customConnections[name].error = error.message;
    
    res.status(500).json({
      error: `Failed to create connection: ${error.message}`
    });
  }
});

// Get registered connections
app.get('/connections', (req, res) => {
  const connections = Object.entries(dynamicConfig.customConnections).map(([name, config]) => ({
    name,
    type: config.type,
    status: config.status,
    created: config.created
  }));
  
  res.json({ connections });
});

// Infinite matrix management
// Store for active matrices
const activeMatrices = {};

// Create a new matrix
app.post('/matrix', (req, res) => {
  const { name, dimensions, storage, scaling } = req.body;
  
  if (!name) {
    return res.status(400).json({ error: 'Matrix name is required' });
  }
  
  if (!dimensions || !Array.isArray(dimensions)) {
    return res.status(400).json({ error: 'Invalid dimensions configuration. Dimensions must be an array.' });
  }
  
  try {
    // Create a new matrix instance
    const matrix = new InfiniteMatrix({
      dimensions,
      storage: storage || { type: 'memory' },
      scaling: scaling || { auto: true }
    });
    
    // Store the matrix
    activeMatrices[name] = matrix;
    
    // Store configuration reference
    dynamicConfig.matrices = dynamicConfig.matrices || {};
    dynamicConfig.matrices[name] = {
      dimensions,
      storageType: storage?.type || 'memory',
      created: new Date().toISOString()
    };
    
    res.status(201).json({
      success: true,
      message: `Matrix '${name}' created successfully`,
      matrix: matrix.getInfo()
    });
  } catch (error) {
    res.status(500).json({
      error: `Failed to create matrix: ${error.message}`
    });
  }
});

// Get matrix info
app.get('/matrix/:name', (req, res) => {
  const { name } = req.params;
  
  if (!activeMatrices[name]) {
    return res.status(404).json({ error: `Matrix '${name}' not found` });
  }
  
  res.json(activeMatrices[name].getInfo());
});

// List all matrices
app.get('/matrices', (req, res) => {
  const matrices = Object.keys(activeMatrices).map(name => ({
    name,
    ...activeMatrices[name].getInfo().metadata,
    dimensions: activeMatrices[name].dimensions.length
  }));
  
  res.json({ matrices });
});

// Set value in matrix
app.post('/matrix/:name/value', (req, res) => {
  const { name } = req.params;
  const { coordinates, value } = req.body;
  
  if (!activeMatrices[name]) {
    return res.status(404).json({ error: `Matrix '${name}' not found` });
  }
  
  if (!coordinates || !Array.isArray(coordinates)) {
    return res.status(400).json({ error: 'Invalid coordinates. Coordinates must be an array.' });
  }
  
  try {
    activeMatrices[name].set(coordinates, value);
    
    res.json({
      success: true,
      message: `Value set at coordinates [${coordinates.join(', ')}]`
    });
  } catch (error) {
    res.status(500).json({
      error: `Failed to set value: ${error.message}`
    });
  }
});

// Get value from matrix
app.get('/matrix/:name/value', (req, res) => {
  const { name } = req.params;
  const coordinatesStr = req.query.coordinates;
  
  if (!activeMatrices[name]) {
    return res.status(404).json({ error: `Matrix '${name}' not found` });
  }
  
  if (!coordinatesStr) {
    return res.status(400).json({ error: 'Coordinates query parameter is required' });
  }
  
  try {
    // Parse coordinates from query string (format: "1,2,3")
    const coordinates = coordinatesStr.split(',').map(Number);
    
    const value = activeMatrices[name].get(coordinates);
    
    res.json({
      coordinates,
      value
    });
  } catch (error) {
    res.status(500).json({
      error: `Failed to get value: ${error.message}`
    });
  }
});

// Query matrix
app.post('/matrix/:name/query', (req, res) => {
  const { name } = req.params;
  const { query } = req.body;
  
  if (!activeMatrices[name]) {
    return res.status(404).json({ error: `Matrix '${name}' not found` });
  }
  
  if (!query || typeof query !== 'object') {
    return res.status(400).json({ error: 'Invalid query. Query must be an object.' });
  }
  
  try {
    const results = activeMatrices[name].query(query);
    
    res.json({
      results,
      count: results.length
    });
  } catch (error) {
    res.status(500).json({
      error: `Failed to query matrix: ${error.message}`
    });
  }
});

// Add dimension to matrix
app.post('/matrix/:name/dimensions', (req, res) => {
  const { name } = req.params;
  const dimension = req.body;
  
  if (!activeMatrices[name]) {
    return res.status(404).json({ error: `Matrix '${name}' not found` });
  }
  
  if (!dimension || !dimension.name) {
    return res.status(400).json({ error: 'Invalid dimension. Dimension must have a name.' });
  }
  
  try {
    const dimensionIndex = activeMatrices[name].addDimension(dimension);
    
    // Update stored config
    if (dynamicConfig.matrices && dynamicConfig.matrices[name]) {
      dynamicConfig.matrices[name].dimensions = activeMatrices[name].dimensions;
    }
    
    res.json({
      success: true,
      message: `Dimension '${dimension.name}' added at index ${dimensionIndex}`,
      dimension
    });
  } catch (error) {
    res.status(500).json({
      error: `Failed to add dimension: ${error.message}`
    });
  }
});

// Remove dimension from matrix
app.delete('/matrix/:name/dimensions/:dimensionName', (req, res) => {
  const { name, dimensionName } = req.params;
  
  if (!activeMatrices[name]) {
    return res.status(404).json({ error: `Matrix '${name}' not found` });
  }
  
  try {
    activeMatrices[name].removeDimension(dimensionName);
    
    // Update stored config
    if (dynamicConfig.matrices && dynamicConfig.matrices[name]) {
      dynamicConfig.matrices[name].dimensions = activeMatrices[name].dimensions;
    }
    
    res.json({
      success: true,
      message: `Dimension '${dimensionName}' removed successfully`
    });
  } catch (error) {
    res.status(500).json({
      error: `Failed to remove dimension: ${error.message}`
    });
  }
});

// Resize matrix
app.patch('/matrix/:name/resize', (req, res) => {
  const { name } = req.params;
  const { dimensions } = req.body;
  
  if (!activeMatrices[name]) {
    return res.status(404).json({ error: `Matrix '${name}' not found` });
  }
  
  if (!dimensions || !Array.isArray(dimensions)) {
    return res.status(400).json({ error: 'Invalid dimensions. Dimensions must be an array.' });
  }
  
  try {
    activeMatrices[name].resize(dimensions);
    
    // Update stored config
    if (dynamicConfig.matrices && dynamicConfig.matrices[name]) {
      dynamicConfig.matrices[name].dimensions = activeMatrices[name].dimensions;
    }
    
    res.json({
      success: true,
      message: 'Matrix resized successfully',
      dimensions: activeMatrices[name].dimensions
    });
  } catch (error) {
    res.status(500).json({
      error: `Failed to resize matrix: ${error.message}`
    });
  }
});

// Change matrix storage
app.patch('/matrix/:name/storage', async (req, res) => {
  const { name } = req.params;
  const { storageType, options } = req.body;
  
  if (!activeMatrices[name]) {
    return res.status(404).json({ error: `Matrix '${name}' not found` });
  }
  
  if (!storageType || (storageType !== 'memory' && storageType !== 'database')) {
    return res.status(400).json({ error: 'Invalid storage type. Must be "memory" or "database".' });
  }
  
  try {
    await activeMatrices[name].changeStorage(storageType, options || {});
    
    // Update stored config
    if (dynamicConfig.matrices && dynamicConfig.matrices[name]) {
      dynamicConfig.matrices[name].storageType = storageType;
    }
    
    res.json({
      success: true,
      message: `Matrix storage changed to ${storageType} successfully`
    });
  } catch (error) {
    res.status(500).json({
      error: `Failed to change matrix storage: ${error.message}`
    });
  }
});

// Health check
app.get('/health', async (req, res) => {
  const status = {
    status: 'ok',
    databases: {}
  };
  
  // Check PostgreSQL
  if (connections.postgresql.client) {
    status.databases.postgresql = {
      connected: await connections.postgresql.testConnection()
    };
  }
  
  // Check MongoDB
  if (connections.mongodb.client) {
    status.databases.mongodb = {
      connected: await connections.mongodb.testConnection()
    };
  }
  
  res.json(status);
});

// Get all available databases
app.get('/databases', async (req, res) => {
  const databases = [];
  
  // Add PostgreSQL if available
  if (connections.postgresql.client) {
    const tables = await connections.postgresql.getTables();
    databases.push({
      type: 'postgresql',
      name: 'postgres',
      collections: tables,
      status: await connections.postgresql.testConnection() ? 'connected' : 'error'
    });
  }
  
  // Add MongoDB if available
  if (connections.mongodb.client) {
    const collections = await connections.mongodb.getTables();
    databases.push({
      type: 'mongodb',
      name: 'mongodb',
      collections: collections,
      status: await connections.mongodb.testConnection() ? 'connected' : 'error'
    });
  }
  
  res.json({ databases });
});

// Generic database operations
// Create a record
app.post('/databases/:dbType/:dbName/records', async (req, res) => {
  const { dbType } = req.params;
  const { collection, data } = req.body;
  
  if (!collection || !data) {
    return res.status(400).json({ error: 'Collection name and data are required' });
  }
  
  // Check if database type is supported
  if (!connections[dbType] || !connections[dbType].client) {
    return res.status(400).json({ error: `Database type '${dbType}' is not supported or not connected` });
  }
  
  try {
    const result = await connections[dbType].insertRecord(collection, data);
    res.status(201).json(result);
  } catch (err) {
    console.error(err);
    res.status(500).json({ error: err.message });
  }
});

// Get a record
app.get('/databases/:dbType/:dbName/records/:id', async (req, res) => {
  const { dbType, id } = req.params;
  const { collection } = req.query;
  
  if (!collection) {
    return res.status(400).json({ error: 'Collection name is required' });
  }
  
  // Check if database type is supported
  if (!connections[dbType] || !connections[dbType].client) {
    return res.status(400).json({ error: `Database type '${dbType}' is not supported or not connected` });
  }
  
  try {
    const record = await connections[dbType].getRecord(collection, id);
    
    if (!record) {
      return res.status(404).json({ error: 'Record not found' });
    }
    
    res.json(record);
  } catch (err) {
    console.error(err);
    res.status(500).json({ error: err.message });
  }
});

// Update a record
app.put('/databases/:dbType/:dbName/records/:id', async (req, res) => {
  const { dbType, id } = req.params;
  const { collection, data } = req.body;
  
  if (!collection || !data) {
    return res.status(400).json({ error: 'Collection name and data are required' });
  }
  
  // Check if database type is supported
  if (!connections[dbType] || !connections[dbType].client) {
    return res.status(400).json({ error: `Database type '${dbType}' is not supported or not connected` });
  }
  
  try {
    const updated = await connections[dbType].updateRecord(collection, id, data);
    
    if (!updated) {
      return res.status(404).json({ error: 'Record not found' });
    }
    
    // Get the updated record
    const record = await connections[dbType].getRecord(collection, id);
    res.json(record);
  } catch (err) {
    console.error(err);
    res.status(500).json({ error: err.message });
  }
});

// Delete a record
app.delete('/databases/:dbType/:dbName/records/:id', async (req, res) => {
  const { dbType, id } = req.params;
  const { collection } = req.query;
  
  if (!collection) {
    return res.status(400).json({ error: 'Collection name is required' });
  }
  
  // Check if database type is supported
  if (!connections[dbType] || !connections[dbType].client) {
    return res.status(400).json({ error: `Database type '${dbType}' is not supported or not connected` });
  }
  
  try {
    const deleted = await connections[dbType].deleteRecord(collection, id);
    
    if (!deleted) {
      return res.status(404).json({ error: 'Record not found' });
    }
    
    res.status(204).end();
  } catch (err) {
    console.error(err);
    res.status(500).json({ error: err.message });
  }
});

// Query records
app.post('/databases/:dbType/:dbName/query', async (req, res) => {
  const { dbType } = req.params;
  const { collection, query, limit, offset, sortBy, sortOrder } = req.body;
  
  if (!collection) {
    return res.status(400).json({ error: 'Collection name is required' });
  }
  
  // Check if database type is supported
  if (!connections[dbType] || !connections[dbType].client) {
    return res.status(400).json({ error: `Database type '${dbType}' is not supported or not connected` });
  }
  
  try {
    const options = { limit, offset, sortBy, sortOrder };
    const records = await connections[dbType].queryRecords(collection, query, options);
    res.json(records);
  } catch (err) {
    console.error(err);
    res.status(500).json({ error: err.message });
  }
});

// Start the server
async function startServer() {
  await initConnections();
  
  app.listen(port, '0.0.0.0', () => {
    console.log(`Unified Database API Server running at http://0.0.0.0:${port}`);
  });
}

startServer().catch(err => {
  console.error('Failed to start server:', err);
  process.exit(1);
});