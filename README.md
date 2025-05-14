# Unified Database API with Infinite Matrix

A powerful and flexible API system that provides a unified interface for interacting with multiple database types while supporting n-dimensional data through an Infinite Matrix implementation.

## Features

- **Multiple Database Support**: SQL, NoSQL, graph, time-series, and vector databases through a single API
- **Infinite Matrix**: N-dimensional data storage and querying with automatic scaling
- **Dynamic Configuration**: Customize every aspect of the system at runtime
- **Client Library**: Easy-to-use client library for seamless integration

## Architecture

The system consists of three main components:

1. **Core API Server**: Handles database connections, request routing, and dynamic configuration
2. **Infinite Matrix Engine**: Manages n-dimensional data with automatic scaling
3. **Client Library**: Provides a convenient interface for applications

## Getting Started

### Start the Server

```bash
node unified_db_api_server.js
```

### Use the Client Library

```javascript
const { UnifiedDBClient } = require('./unified_db_client');

// Create a client
const client = new UnifiedDBClient({
  baseUrl: 'http://localhost:8000',
  defaultDbType: 'postgresql',
  debug: true
});

// Work with PostgreSQL
const postgres = client.database('postgresql', 'postgres');
const users = await postgres.queryRecords('users', { active: true });

// Work with Infinite Matrix
// Create a 3D matrix
await client._fetch('/matrix', {
  method: 'POST',
  body: JSON.stringify({
    name: 'my-matrix',
    dimensions: [
      { name: 'x', type: 'numeric', min: 0, max: 100 },
      { name: 'y', type: 'numeric', min: 0, max: 100 },
      { name: 'z', type: 'numeric', min: 0, max: 100 }
    ]
  })
});

// Get a matrix interface
const matrix = client.matrix('my-matrix');

// Set values
await matrix.setValue([10, 20, 30], { data: 'Hello, Matrix!' });

// Query values
const results = await matrix.query({ x: { min: 5, max: 15 } });
```

## Dynamic Configuration

The API allows you to configure almost every aspect of the system at runtime:

```javascript
// Update performance settings
await client._fetch('/config', {
  method: 'PATCH',
  body: JSON.stringify({
    section: 'performance',
    updates: {
      cacheSize: 5000,
      queryTimeout: 120000
    }
  })
});

// Register a custom adapter
await client._fetch('/adapters', {
  method: 'POST',
  body: JSON.stringify({
    name: 'custom-db',
    type: 'nosql',
    connectionConfig: {
      url: 'https://custom-db.example.com'
    }
  })
});
```

## Infinite Matrix Operations

The Infinite Matrix provides powerful capabilities for working with n-dimensional data:

```javascript
// Create a matrix
await client._fetch('/matrix', {
  method: 'POST',
  body: JSON.stringify({
    name: 'metrics',
    dimensions: [
      { name: 'time', min: 0, max: 1000 },
      { name: 'user', min: 0, max: 10000 },
      { name: 'metric', min: 0, max: 50 }
    ]
  })
});

const matrix = client.matrix('metrics');

// Store time-series data for users
await matrix.setValue([1, 123, 0], { value: 42 });  // User 123, metric 0, time 1
await matrix.setValue([2, 123, 0], { value: 43 });  // User 123, metric 0, time 2

// Query by time range for a specific user and metric
const results = await matrix.query({
  time: { min: 0, max: 10 },
  user: 123,
  metric: 0
});

// Add a new dimension
await matrix.addDimension({
  name: 'region',
  min: 0,
  max: 5
});

// Now the matrix is 4D, and we can store region-specific data
await matrix.setValue([3, 123, 0, 2], { value: 44, region: 'Europe' });
```

## Supported Database Types

- **SQL**: PostgreSQL, MySQL
- **NoSQL**: MongoDB, Redis, Cassandra
- **Graph**: Neo4j
- **Time Series**: InfluxDB
- **Vector**: Qdrant
- **Custom**: Add your own database types via the API

## Examples

Check out these examples to see the API in action:

- `client_example.js`: Basic client usage
- `test_infinite_matrix.js`: In-depth Infinite Matrix operations

## Scaling

The system supports dynamic scaling:

1. Configure `maxConnections` to control connection pooling
2. Set `scalingFactor` to adjust the resource allocation
3. Use the matrix's `scaling` options to control n-dimensional scaling behavior

## Customization

Almost every aspect of the system can be customized through the API:

- Database adapters and connections
- Performance settings
- Security rules
- Matrix dimensions and storage