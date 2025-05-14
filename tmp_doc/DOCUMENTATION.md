# Unified Database API with Infinite Matrix - Technical Documentation

This comprehensive documentation covers the architecture, components, functionality, and usage of the Unified Database API system. This API provides a unified interface for multiple database types while supporting n-dimensional data through the Infinite Matrix implementation.

## Table of Contents

1. [System Architecture](#system-architecture)
2. [Core Components](#core-components)
3. [API Endpoints Reference](#api-endpoints-reference)
4. [Client Library Reference](#client-library-reference)
5. [Infinite Matrix Implementation](#infinite-matrix-implementation)
6. [Dynamic Configuration System](#dynamic-configuration-system)
7. [Database Connectors](#database-connectors)
8. [Deployment and Scaling](#deployment-and-scaling)
9. [Security Considerations](#security-considerations)
10. [Examples and Use Cases](#examples-and-use-cases)

## System Architecture

The Unified Database API is built on a modular architecture designed for flexibility, extensibility, and performance:

### High-Level Architecture

```
┌────────────────────┐     ┌─────────────────┐     ┌─────────────────────┐
│                    │     │                 │     │                     │
│    Client Apps     │────▶│  Unified API    │────▶│  Database Adapters  │
│                    │     │      Layer      │     │                     │
└────────────────────┘     └─────────────────┘     └─────────────────────┘
                                   │                          │
                                   ▼                          ▼
                           ┌─────────────────┐     ┌─────────────────────┐
                           │                 │     │                     │
                           │ Infinite Matrix │     │ Database Connectors │
                           │     Engine      │     │                     │
                           └─────────────────┘     └─────────────────────┘
                                                             │
                                                             ▼
                                                   ┌─────────────────────┐
                                                   │                     │
                                                   │  Database Systems   │
                                                   │                     │
                                                   └─────────────────────┘
```

### Core Design Principles

1. **Unified Interface**: All database types are accessed through a consistent API
2. **Extensibility**: New database types can be added without modifying existing code
3. **Dynamic Configuration**: All aspects of the system can be configured at runtime
4. **Scalability**: Components can scale independently to handle increased load
5. **Abstraction**: Complex database operations are abstracted behind a simple interface

## Core Components

### Unified API Server

The API server (unified_db_api_server.js) serves as the central entry point for all client interactions. It provides:

- Request routing and handling
- Authentication and authorization
- Database connection management
- Dynamic configuration
- Error handling and logging

### Infinite Matrix Engine

The Infinite Matrix (infinite_matrix.js) provides n-dimensional data storage and manipulation:

- Dynamic dimension management
- Automatic scaling of dimensions
- Efficient storage and retrieval of data
- Query capabilities across dimensions
- Support for different storage backends

### Client Library

The client library (unified_db_client.js) provides a convenient interface for applications:

- Type-safe interfaces for different database operations
- Simplified error handling
- Connection pooling and management
- Automatic retries and timeouts
- Support for all server-side features

### Database Adapters

Database adapters provide the translation layer between the unified API and specific database systems:

- SQL databases (PostgreSQL, MySQL)
- NoSQL databases (MongoDB, Redis, Cassandra)
- Graph databases (Neo4j)
- Time-series databases (InfluxDB)
- Vector databases (Qdrant)
- Custom database types

## API Endpoints Reference

### Configuration Endpoints

#### GET /config

Retrieves the current system configuration.

**Response**:
```json
{
  "version": "1.0.0",
  "scalingFactor": 1,
  "maxConnections": 100,
  "enableDynamicConfiguration": true,
  "dynamicAdapters": true,
  "supportedDatabases": {...},
  "dynamic": {
    "activeAdapters": {...},
    "customConnections": {...},
    "scalingRules": {...},
    "securityRules": {...},
    "performance": {...}
  }
}
```

#### PATCH /config

Updates a section of the system configuration.

**Request**:
```json
{
  "section": "performance",
  "updates": {
    "cacheSize": 5000,
    "queryTimeout": 60000
  }
}
```

**Response**:
```json
{
  "success": true,
  "message": "Configuration updated successfully"
}
```

### Adapter Management

#### POST /adapters

Registers a custom database adapter.

**Request**:
```json
{
  "name": "custom-db",
  "type": "nosql",
  "connectionConfig": {
    "url": "https://custom-db.example.com"
  }
}
```

**Response**:
```json
{
  "success": true,
  "message": "Custom adapter 'custom-db' registered successfully",
  "adapter": {...}
}
```

### Connection Management

#### POST /connections

Creates a new database connection.

**Request**:
```json
{
  "name": "my-postgres",
  "type": "postgresql",
  "connectionString": "postgresql://user:password@host:port/database"
}
```

**Response**:
```json
{
  "success": true,
  "message": "Connection 'my-postgres' created successfully",
  "connection": {...}
}
```

#### GET /connections

Lists all registered connections.

**Response**:
```json
{
  "connections": [
    {
      "name": "my-postgres",
      "type": "postgresql",
      "status": "connected",
      "created": "2023-05-14T12:34:56.789Z"
    }
  ]
}
```

### Infinite Matrix Endpoints

#### POST /matrix

Creates a new matrix.

**Request**:
```json
{
  "name": "my-matrix",
  "dimensions": [
    { "name": "x", "type": "numeric", "min": 0, "max": 100 },
    { "name": "y", "type": "numeric", "min": 0, "max": 100 },
    { "name": "z", "type": "numeric", "min": 0, "max": 100 }
  ],
  "storage": { "type": "memory" },
  "scaling": { "auto": true }
}
```

**Response**:
```json
{
  "success": true,
  "message": "Matrix 'my-matrix' created successfully",
  "matrix": {...}
}
```

#### GET /matrix/:name

Gets information about a matrix.

**Response**:
```json
{
  "dimensions": [...],
  "storage": {...},
  "scaling": {...},
  "metadata": {
    "created": "2023-05-14T12:34:56.789Z",
    "lastModified": "2023-05-14T12:34:56.789Z",
    "cellCount": 10
  }
}
```

#### POST /matrix/:name/value

Sets a value in a matrix.

**Request**:
```json
{
  "coordinates": [10, 20, 30],
  "value": { "data": "Hello, Matrix!" }
}
```

**Response**:
```json
{
  "success": true,
  "message": "Value set at coordinates [10, 20, 30]"
}
```

#### GET /matrix/:name/value

Gets a value from a matrix.

**Query Parameters**:
- `coordinates`: Comma-separated list of coordinates (e.g., `10,20,30`)

**Response**:
```json
{
  "coordinates": [10, 20, 30],
  "value": { "data": "Hello, Matrix!" }
}
```

#### POST /matrix/:name/query

Queries values in a matrix.

**Request**:
```json
{
  "query": {
    "x": { "min": 5, "max": 15 },
    "y": 20,
    "z": { "min": 25, "max": 35 }
  }
}
```

**Response**:
```json
{
  "results": [
    {
      "coordinates": [10, 20, 30],
      "value": { "data": "Hello, Matrix!" }
    }
  ],
  "count": 1
}
```

#### POST /matrix/:name/dimensions

Adds a dimension to a matrix.

**Request**:
```json
{
  "name": "time",
  "type": "numeric",
  "min": 0,
  "max": 1000
}
```

**Response**:
```json
{
  "success": true,
  "message": "Dimension 'time' added at index 3",
  "dimension": {...}
}
```

#### DELETE /matrix/:name/dimensions/:dimensionName

Removes a dimension from a matrix.

**Response**:
```json
{
  "success": true,
  "message": "Dimension 'time' removed successfully"
}
```

#### PATCH /matrix/:name/resize

Resizes a matrix.

**Request**:
```json
{
  "dimensions": [
    { "name": "x", "min": 0, "max": 200 },
    { "name": "y", "min": 0, "max": 200 }
  ]
}
```

**Response**:
```json
{
  "success": true,
  "message": "Matrix resized successfully",
  "dimensions": [...]
}
```

#### PATCH /matrix/:name/storage

Changes the storage type for a matrix.

**Request**:
```json
{
  "storageType": "database",
  "options": {
    "collection": "matrix_data",
    "dbType": "mongodb"
  }
}
```

**Response**:
```json
{
  "success": true,
  "message": "Matrix storage changed to database successfully"
}
```

### Database Operations

#### POST /databases/:dbType/:dbName/records

Creates a record in a collection.

**Request**:
```json
{
  "collection": "users",
  "data": {
    "name": "John Doe",
    "email": "john@example.com"
  }
}
```

**Response**:
```json
{
  "id": "123",
  "name": "John Doe",
  "email": "john@example.com",
  "created": "2023-05-14T12:34:56.789Z"
}
```

#### GET /databases/:dbType/:dbName/records/:id

Gets a record by ID.

**Query Parameters**:
- `collection`: The collection/table name

**Response**:
```json
{
  "id": "123",
  "name": "John Doe",
  "email": "john@example.com",
  "created": "2023-05-14T12:34:56.789Z"
}
```

#### PUT /databases/:dbType/:dbName/records/:id

Updates a record.

**Request**:
```json
{
  "collection": "users",
  "data": {
    "name": "John Doe",
    "email": "john.updated@example.com"
  }
}
```

**Response**:
```json
{
  "id": "123",
  "name": "John Doe",
  "email": "john.updated@example.com",
  "created": "2023-05-14T12:34:56.789Z",
  "updated": "2023-05-14T13:45:67.890Z"
}
```

#### DELETE /databases/:dbType/:dbName/records/:id

Deletes a record.

**Query Parameters**:
- `collection`: The collection/table name

**Response**:
```json
{
  "success": true,
  "message": "Record deleted successfully"
}
```

#### POST /databases/:dbType/:dbName/query

Queries records in a collection.

**Request**:
```json
{
  "collection": "users",
  "query": { "name": "John" },
  "limit": 10,
  "offset": 0,
  "sortBy": "created",
  "sortOrder": "desc"
}
```

**Response**:
```json
{
  "results": [...],
  "total": 1,
  "limit": 10,
  "offset": 0
}
```

### Database-Specific Endpoints

#### POST /databases/:dbType/:dbName/aggregate (MongoDB)

Executes an aggregation pipeline.

**Request**:
```json
{
  "collection": "orders",
  "pipeline": [
    { "$match": { "status": "completed" } },
    { "$group": { "_id": "$customer", "total": { "$sum": "$amount" } } }
  ]
}
```

**Response**:
```json
{
  "results": [...]
}
```

#### POST /databases/:dbType/:dbName/execute (SQL)

Executes a raw SQL query.

**Request**:
```json
{
  "sql": "SELECT * FROM users WHERE name LIKE $1",
  "params": ["John%"]
}
```

**Response**:
```json
{
  "results": [...],
  "rowCount": 1
}
```

#### POST /databases/:dbType/:dbName/cypher (Neo4j)

Executes a Cypher query.

**Request**:
```json
{
  "query": "MATCH (user:User)-[:PURCHASED]->(product:Product) RETURN user, product",
  "params": {}
}
```

**Response**:
```json
{
  "results": [...]
}
```

## Client Library Reference

### UnifiedDBClient

The main client class for interacting with the API.

```javascript
const client = new UnifiedDBClient({
  baseUrl: 'http://localhost:8000',
  defaultDbType: 'postgresql',
  defaultDbName: 'postgres',
  auth: {
    type: 'bearer',
    token: 'your-auth-token'
  },
  debug: true
});
```

#### Methods

##### database(dbType, dbName)

Gets a database interface for the specified database type.

```javascript
const postgresql = client.database('postgresql', 'postgres');
const mongodb = client.database('mongodb', 'mydb');
```

##### matrix(name)

Gets a matrix interface.

```javascript
const matrix = client.matrix('my-matrix');
```

##### async getDatabases()

Gets available databases.

```javascript
const databases = await client.getDatabases();
```

##### async getHealth()

Gets system health status.

```javascript
const health = await client.getHealth();
```

### Database Interfaces

#### SQL Database Interface

```javascript
const postgres = client.database('postgresql', 'postgres');

// Create a record
const newUser = await postgres.createRecord('users', {
  name: 'John Doe',
  email: 'john@example.com'
});

// Get a record
const user = await postgres.getRecord('users', 123);

// Update a record
await postgres.updateRecord('users', 123, {
  name: 'John Doe',
  email: 'john.updated@example.com'
});

// Delete a record
await postgres.deleteRecord('users', 123);

// Query records
const users = await postgres.queryRecords('users', 
  { name: 'John' }, 
  { limit: 10, offset: 0, sortBy: 'created', sortOrder: 'desc' }
);

// Execute raw SQL
const results = await postgres.executeRawQuery(
  'SELECT * FROM users WHERE name LIKE $1',
  ['John%']
);
```

#### Document Database Interface

```javascript
const mongodb = client.database('mongodb', 'mydb');

// Document databases have all SQL methods plus:
const results = await mongodb.aggregate('orders', [
  { $match: { status: 'completed' } },
  { $group: { _id: '$customer', total: { $sum: '$amount' } } }
]);

// Upsert a record
await mongodb.upsertRecord('users', 
  { email: 'john@example.com' }, 
  { name: 'John Doe', email: 'john@example.com' }
);
```

#### Key-Value Database Interface

```javascript
const redis = client.database('redis', 'cache');

// Set a value
await redis.set('user:123', { name: 'John Doe' }, { expiry: 3600 });

// Get a value
const user = await redis.get('user:123');

// Delete a value
await redis.delete('user:123');

// Increment a value
const newCount = await redis.increment('visitor_count', 1);
```

#### Graph Database Interface

```javascript
const neo4j = client.database('neo4j', 'graph');

// Create a node
const user = await neo4j.createNode('User', {
  name: 'John Doe',
  email: 'john@example.com'
});

// Create a relationship
await neo4j.createRelationship(
  user.id,
  product.id,
  'PURCHASED',
  { date: '2023-05-14' }
);

// Execute a Cypher query
const results = await neo4j.query(
  'MATCH (user:User)-[:PURCHASED]->(product:Product) RETURN user, product'
);
```

#### Time Series Database Interface

```javascript
const influxdb = client.database('influxdb', 'metrics');

// Write points
await influxdb.writePoints('cpu_usage', [
  { time: '2023-05-14T12:34:56Z', value: 0.75, host: 'server1' },
  { time: '2023-05-14T12:34:57Z', value: 0.80, host: 'server1' }
]);

// Query data
const results = await influxdb.query(
  'from(bucket:"metrics") |> range(start: -1h) |> filter(fn: (r) => r._measurement == "cpu_usage")'
);
```

#### Vector Database Interface

```javascript
const qdrant = client.database('qdrant', 'vectors');

// Create a collection
await qdrant.createCollection('embeddings', {
  vectorSize: 384,
  distance: 'Cosine'
});

// Add vectors
await qdrant.addVectors('embeddings', [
  { id: '1', vector: [...], payload: { text: 'Hello world' } },
  { id: '2', vector: [...], payload: { text: 'How are you?' } }
]);

// Search for similar vectors
const results = await qdrant.search('embeddings', [...], 10);
```

### Matrix Interface

```javascript
const matrix = client.matrix('my-matrix');

// Get matrix info
const info = await matrix.getInfo();

// Set a value
await matrix.setValue([10, 20, 30], { data: 'Hello, Matrix!' });

// Get a value
const value = await matrix.getValue([10, 20, 30]);

// Query values
const results = await matrix.query({
  x: { min: 5, max: 15 },
  y: 20,
  z: { min: 25, max: 35 }
});

// Add a dimension
await matrix.addDimension({
  name: 'time',
  type: 'numeric',
  min: 0,
  max: 1000
});

// Remove a dimension
await matrix.removeDimension('time');

// Resize the matrix
await matrix.resize([
  { name: 'x', min: 0, max: 200 },
  { name: 'y', min: 0, max: 200 }
]);

// Change storage type
await matrix.changeStorage('database', {
  collection: 'matrix_data',
  dbType: 'mongodb'
});
```

## Infinite Matrix Implementation

The Infinite Matrix is implemented in `infinite_matrix.js` and provides a powerful system for working with n-dimensional data.

### Key Features

- **Dynamic Dimensions**: Dimensions can be added or removed at runtime
- **Automatic Scaling**: Matrix automatically expands to accommodate new data
- **Efficient Storage**: Data is stored sparsely, only for non-empty cells
- **Multiple Storage Backends**: In-memory or database storage
- **Query Capabilities**: Query data across any combination of dimensions

### Implementation Details

#### Data Structure

The matrix uses a sparse representation where only non-empty cells are stored. This allows for efficient storage of high-dimensional data where most cells are empty.

```javascript
// Example internal structure for a 3D matrix
{
  "10:20:30": { data: "Value at coordinates [10,20,30]" },
  "15:25:35": { data: "Another value" }
}
```

#### Dimension Management

Each dimension has metadata including name, type, minimum and maximum values:

```javascript
{
  name: "x",
  type: "numeric",
  min: 0,
  max: 100,
  default: 0
}
```

#### Coordinate System

Coordinates are represented as arrays of numbers, with each element corresponding to a dimension:

```javascript
[10, 20, 30] // 3D coordinates
[10, 20, 30, 40] // 4D coordinates
```

#### Storage Types

The matrix supports multiple storage types:

- **Memory**: Data is stored in-memory
- **Database**: Data is stored in a database (PostgreSQL, MongoDB, etc.)

## Dynamic Configuration System

The dynamic configuration system allows every aspect of the API to be configured at runtime.

### Configuration Sections

#### Global Configuration

```javascript
{
  "version": "1.0.0",
  "scalingFactor": 1,
  "maxConnections": 100,
  "enableDynamicConfiguration": true,
  "dynamicAdapters": true
}
```

#### Scaling Rules

```javascript
{
  "autoScale": true,
  "minInstances": 1,
  "maxInstances": 10,
  "scaleUpThreshold": 0.8,
  "scaleDownThreshold": 0.2
}
```

#### Security Rules

```javascript
{
  "rateLimiting": true,
  "maxRequestsPerMinute": 100,
  "authRequired": false,
  "allowedOrigins": ["*"]
}
```

#### Performance

```javascript
{
  "connectionPooling": true,
  "cacheEnabled": true,
  "cacheSize": 1000,
  "cacheExpirySeconds": 300,
  "queryTimeout": 30000
}
```

## Database Connectors

The system includes connectors for various database types. Each connector implements a common interface for consistent interaction.

### PostgreSQL Connector

The PostgreSQL connector uses the `pg` package and connection pooling for efficient database operations.

**Features**:
- Connection pooling
- Parameterized queries
- Transaction support
- Error handling and retries

### MongoDB Connector

The MongoDB connector uses the official MongoDB Node.js driver.

**Features**:
- Collection management
- CRUD operations
- Aggregation pipeline support
- Indexing

## Deployment and Scaling

### Deployment Options

The Unified Database API can be deployed in various environments:

- **Standalone**: Run as a standalone Node.js application
- **Docker**: Run in a Docker container
- **Kubernetes**: Deploy in a Kubernetes cluster for horizontal scaling
- **Serverless**: Deploy as a serverless function (with limitations)

### Scaling Strategies

#### Vertical Scaling

Adjust the `scalingFactor` configuration to allocate more resources to the API server.

#### Horizontal Scaling

Run multiple instances of the API server behind a load balancer. Each instance can handle a portion of the total load.

#### Database Scaling

Configure each database connector with its own scaling parameters. For example:

- PostgreSQL: Connection pool size
- MongoDB: Read preference and write concern

## Security Considerations

### Authentication and Authorization

The API supports multiple authentication methods:

- **Bearer Token**: JWT-based authentication
- **Basic Auth**: Username/password authentication
- **API Key**: Simple API key authentication

### Data Protection

Sensitive data should be protected using:

- **TLS/SSL**: All connections should use HTTPS
- **Encryption**: Sensitive data can be encrypted at rest
- **Access Control**: Fine-grained access control for database operations

### Rate Limiting

Configure rate limiting to prevent abuse:

```javascript
{
  "rateLimiting": true,
  "maxRequestsPerMinute": 100
}
```

## Examples and Use Cases

### Financial Time-Series Data

Store and analyze financial time-series data using the Infinite Matrix:

```javascript
// Create a matrix for financial data
await client._fetch('/matrix', {
  method: 'POST',
  body: JSON.stringify({
    name: 'financial-data',
    dimensions: [
      { name: 'time', type: 'numeric', min: 0, max: 1000 },
      { name: 'symbol', type: 'numeric', min: 0, max: 1000 },
      { name: 'metric', type: 'numeric', min: 0, max: 10 }
    ]
  })
});

const matrix = client.matrix('financial-data');

// Store price data
await matrix.setValue([1, 100, 0], { price: 150.50 });
await matrix.setValue([2, 100, 0], { price: 152.75 });

// Store volume data
await matrix.setValue([1, 100, 1], { volume: 10000 });
await matrix.setValue([2, 100, 1], { volume: 12000 });

// Query price data for a symbol over time
const priceData = await matrix.query({
  symbol: 100,
  metric: 0,
  time: { min: 0, max: 10 }
});
```

### User Activity Tracking

Track user activity across multiple dimensions:

```javascript
// Create a matrix for user activity
await client._fetch('/matrix', {
  method: 'POST',
  body: JSON.stringify({
    name: 'user-activity',
    dimensions: [
      { name: 'user', type: 'numeric', min: 0, max: 10000 },
      { name: 'action', type: 'numeric', min: 0, max: 100 },
      { name: 'day', type: 'numeric', min: 0, max: 365 }
    ]
  })
});

const matrix = client.matrix('user-activity');

// Record user actions
await matrix.setValue([123, 1, 1], { count: 5 }); // User 123, action 1 (login), day 1
await matrix.setValue([123, 2, 1], { count: 10 }); // User 123, action 2 (page view), day 1

// Add a new dimension for location
await matrix.addDimension({
  name: 'location',
  type: 'numeric',
  min: 0,
  max: 1000
});

// Record user actions with location
await matrix.setValue([123, 1, 2, 42], { count: 3 }); // User 123, action 1, day 2, location 42

// Query user activity by location
const locationActivity = await matrix.query({
  location: 42,
  day: { min: 0, max: 10 }
});
```

### Multi-Database Application

Use multiple database types in a single application:

```javascript
// Use PostgreSQL for relational data
const postgres = client.database('postgresql', 'postgres');
const users = await postgres.queryRecords('users', { active: true });

// Use MongoDB for document data
const mongodb = client.database('mongodb', 'mydb');
const orders = await mongodb.queryRecords('orders', { status: 'completed' });

// Use Redis for caching
const redis = client.database('redis', 'cache');
await redis.set('user:123:cart', { items: [...] }, { expiry: 3600 });

// Use Neo4j for graph data
const neo4j = client.database('neo4j', 'graph');
const recommendations = await neo4j.query(
  'MATCH (user:User {id: $userId})-[:PURCHASED]->(product:Product)<-[:PURCHASED]-(otherUser:User)-[:PURCHASED]->(otherProduct:Product) WHERE NOT (user)-[:PURCHASED]->(otherProduct) RETURN otherProduct, count(*) as score ORDER BY score DESC LIMIT 5',
  { userId: 123 }
);
```

---

This documentation provides a comprehensive overview of the Unified Database API system. For more specific use cases or technical details, refer to the inline code documentation or example files.