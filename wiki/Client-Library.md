# Client Library Documentation

The Unified Database API provides a powerful client library (`unified_db_client.js`) that simplifies interaction with the API. This page documents the client library's features and usage.

## Installation

The client library is included in the repository. No separate installation is required.

```javascript
const { UnifiedDBClient } = require('./unified_db_client');
```

## Creating a Client

```javascript
const client = new UnifiedDBClient({
  baseUrl: 'http://localhost:8000',
  defaultDbType: 'postgresql',
  defaultDbName: 'postgres',
  auth: {
    type: 'bearer', // or 'basic'
    token: 'your-auth-token' // or username/password for basic auth
  },
  debug: true // Enable debug logging
});
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `baseUrl` | Base URL of the API server | `'http://localhost:8000'` |
| `defaultDbType` | Default database type | `'postgresql'` |
| `defaultDbName` | Default database name | `'postgres'` |
| `auth` | Authentication configuration | `null` |
| `debug` | Enable debug logging | `false` |

## Core Methods

### Getting System Information

```javascript
// Get system health
const health = await client.getHealth();

// Get available databases
const databases = await client.getDatabases();
```

### Working with Databases

Get a database interface for a specific database type:

```javascript
const postgres = client.database('postgresql', 'postgres');
const mongodb = client.database('mongodb', 'mydb');
const redis = client.database('redis', 'cache');
const neo4j = client.database('neo4j', 'graph');
const influxdb = client.database('influxdb', 'metrics');
const qdrant = client.database('qdrant', 'vectors');
```

### Working with Matrices

Get a matrix interface for a specific matrix:

```javascript
const matrix = client.matrix('my-matrix');
```

### Direct API Access

For operations not covered by the higher-level interfaces, you can access the API directly:

```javascript
const response = await client._fetch('/path/to/resource', {
  method: 'POST',
  body: JSON.stringify({ /* request body */ })
});
```

## Database Interfaces

Each database type has its own interface with type-specific methods.

### SQL Database Interface

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

### Document Database Interface

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

### Key-Value Database Interface

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

### Graph Database Interface

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

### Time Series Database Interface

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

### Vector Database Interface

```javascript
const qdrant = client.database('qdrant', 'vectors');

// Create a collection
await qdrant.createCollection('embeddings', {
  vectorSize: 384,
  distance: 'Cosine'
});

// Add vectors
await qdrant.addVectors('embeddings', [
  { id: '1', vector: [0.1, 0.2, 0.3], payload: { text: 'Hello world' } },
  { id: '2', vector: [0.2, 0.3, 0.4], payload: { text: 'How are you?' } }
]);

// Search for similar vectors
const results = await qdrant.search('embeddings', [0.1, 0.2, 0.3], 10);
```

## Matrix Interface

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

## Authentication

The client supports multiple authentication methods:

### Bearer Token Authentication

```javascript
const client = new UnifiedDBClient({
  baseUrl: 'http://localhost:8000',
  auth: {
    type: 'bearer',
    token: 'your-jwt-token'
  }
});
```

### Basic Authentication

```javascript
const client = new UnifiedDBClient({
  baseUrl: 'http://localhost:8000',
  auth: {
    type: 'basic',
    username: 'your-username',
    password: 'your-password'
  }
});
```

## Debugging

Enable debug logging to see detailed information about requests and responses:

```javascript
const client = new UnifiedDBClient({
  baseUrl: 'http://localhost:8000',
  debug: true
});
```

This will log:
- All API requests with method and URL
- Response status and timing information
- Error details

## Error Handling

The client provides detailed error information when operations fail. Errors include:

- HTTP status code
- Error message
- Additional context (when available)

Example error handling:

```javascript
try {
  const user = await postgres.getRecord('users', 123);
  console.log('User found:', user);
} catch (error) {
  console.error('Failed to get user:', error.message);
  
  // Check for specific error types
  if (error.message.includes('not found')) {
    console.error('User does not exist');
  } else if (error.message.includes('unauthorized')) {
    console.error('Authentication required');
  }
}
```

## Custom Adapters

You can register custom adapters for database types not included in the default set:

```javascript
client.setAdapter('custom-db', (dbType, dbName) => ({
  async query(params) {
    // Custom implementation
    return client._fetch(`/databases/${dbType}/${dbName}/custom-query`, {
      method: 'POST',
      body: JSON.stringify(params)
    });
  },
  // Other custom methods
}));

// Use the custom adapter
const customDb = client.database('custom-db', 'mydb');
const results = await customDb.query({ /* query params */ });
```

## Best Practices

1. **Reuse client instances**: Create a single client instance and reuse it throughout your application
2. **Handle errors properly**: Always wrap API calls in try/catch blocks
3. **Use the appropriate database interface**: Choose the right database type for your needs
4. **Enable debug in development**: Debug logging helps identify issues
5. **Use query parameters**: Specify limit and offset for queries that might return large result sets
6. **Cache responses**: Consider caching responses for frequently accessed data
7. **Use transactions**: For SQL operations that need to be atomic

## Examples

See the example files in the repository for practical usage examples:

- `client_example.js`: Basic client usage
- `test_infinite_matrix.js`: Matrix operations

## Further Reading

- [API Reference](API-Reference) - Complete API documentation
- [Infinite Matrix Guide](Infinite-Matrix) - Detailed information about the Infinite Matrix
- [Architecture](Architecture) - System architecture overview