# API Reference

This page provides a detailed reference of all endpoints available in the Unified Database API.

## Table of Contents

1. [Configuration Endpoints](#configuration-endpoints)
2. [Matrix Operations](#matrix-operations)
3. [Database Operations](#database-operations)
4. [Health and Diagnostics](#health-and-diagnostics)
5. [Adapter Management](#adapter-management)
6. [Connection Management](#connection-management)

## Configuration Endpoints

### GET `/config`

Retrieves the current system configuration.

**Response:**
```json
{
  "version": "1.0.0",
  "scalingFactor": 1,
  "maxConnections": 100,
  "enableDynamicConfiguration": true,
  "dynamicAdapters": true,
  "supportedDatabases": {
    "sql": ["postgresql", "mysql"],
    "nosql": ["mongodb", "redis", "cassandra"],
    "graph": ["neo4j"],
    "timeseries": ["influxdb"],
    "vector": ["qdrant"],
    "custom": []
  },
  "dynamic": {
    "activeAdapters": {},
    "customConnections": {},
    "scalingRules": {
      "autoScale": true,
      "minInstances": 1,
      "maxInstances": 10,
      "scaleUpThreshold": 0.8,
      "scaleDownThreshold": 0.2
    },
    "securityRules": {
      "rateLimiting": true,
      "maxRequestsPerMinute": 100,
      "authRequired": false,
      "allowedOrigins": ["*"]
    },
    "performance": {
      "connectionPooling": true,
      "cacheEnabled": true,
      "cacheSize": 1000,
      "cacheExpirySeconds": 300,
      "queryTimeout": 30000
    }
  }
}
```

### PATCH `/config`

Updates a section of the system configuration.

**Request:**
```json
{
  "section": "performance",
  "updates": {
    "cacheSize": 5000,
    "queryTimeout": 60000
  }
}
```

**Response:**
```json
{
  "success": true,
  "message": "Configuration updated successfully"
}
```

## Matrix Operations

### POST `/matrix`

Creates a new matrix.

**Request:**
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

**Response:**
```json
{
  "success": true,
  "message": "Matrix 'my-matrix' created successfully",
  "matrix": {
    "dimensions": [...],
    "storage": {...},
    "scaling": {...},
    "metadata": {...}
  }
}
```

### GET `/matrix/:name`

Gets information about a matrix.

**Response:**
```json
{
  "dimensions": [
    { "name": "x", "type": "numeric", "min": 0, "max": 100, "default": 0 },
    { "name": "y", "type": "numeric", "min": 0, "max": 100, "default": 0 },
    { "name": "z", "type": "numeric", "min": 0, "max": 100, "default": 0 }
  ],
  "storage": {
    "type": "memory",
    "options": {}
  },
  "scaling": {
    "auto": true,
    "maxSize": 9007199254740991,
    "growthFactor": 2
  },
  "metadata": {
    "created": "2023-05-14T12:34:56.789Z",
    "lastModified": "2023-05-14T12:34:56.789Z",
    "cellCount": 10
  }
}
```

### POST `/matrix/:name/value`

Sets a value in a matrix.

**Request:**
```json
{
  "coordinates": [10, 20, 30],
  "value": { "data": "Hello, Matrix!" }
}
```

**Response:**
```json
{
  "success": true,
  "message": "Value set at coordinates [10, 20, 30]"
}
```

### GET `/matrix/:name/value`

Gets a value from a matrix.

**Query Parameters:**
- `coordinates`: Comma-separated list of coordinates (e.g., `10,20,30`)

**Response:**
```json
{
  "coordinates": [10, 20, 30],
  "value": { "data": "Hello, Matrix!" }
}
```

### POST `/matrix/:name/query`

Queries values in a matrix.

**Request:**
```json
{
  "query": {
    "x": { "min": 5, "max": 15 },
    "y": 20,
    "z": { "min": 25, "max": 35 }
  }
}
```

**Response:**
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

### POST `/matrix/:name/dimensions`

Adds a dimension to a matrix.

**Request:**
```json
{
  "name": "time",
  "type": "numeric",
  "min": 0,
  "max": 1000,
  "default": 0
}
```

**Response:**
```json
{
  "success": true,
  "message": "Dimension 'time' added at index 3",
  "dimension": {
    "name": "time",
    "type": "numeric",
    "min": 0,
    "max": 1000,
    "default": 0
  }
}
```

### DELETE `/matrix/:name/dimensions/:dimensionName`

Removes a dimension from a matrix.

**Response:**
```json
{
  "success": true,
  "message": "Dimension 'time' removed successfully"
}
```

### PATCH `/matrix/:name/resize`

Resizes a matrix.

**Request:**
```json
{
  "dimensions": [
    { "name": "x", "min": 0, "max": 200 },
    { "name": "y", "min": 0, "max": 200 }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "message": "Matrix resized successfully",
  "dimensions": [
    { "name": "x", "type": "numeric", "min": 0, "max": 200, "default": 0 },
    { "name": "y", "type": "numeric", "min": 0, "max": 200, "default": 0 }
  ]
}
```

### PATCH `/matrix/:name/storage`

Changes the storage type for a matrix.

**Request:**
```json
{
  "storageType": "database",
  "options": {
    "collection": "matrix_data",
    "dbType": "mongodb"
  }
}
```

**Response:**
```json
{
  "success": true,
  "message": "Matrix storage changed to database successfully"
}
```

## Database Operations

### POST `/databases/:dbType/:dbName/records`

Creates a record in a collection.

**Request:**
```json
{
  "collection": "users",
  "data": {
    "name": "John Doe",
    "email": "john@example.com"
  }
}
```

**Response:**
```json
{
  "id": "123",
  "name": "John Doe",
  "email": "john@example.com",
  "created": "2023-05-14T12:34:56.789Z"
}
```

### GET `/databases/:dbType/:dbName/records/:id`

Gets a record by ID.

**Query Parameters:**
- `collection`: The collection/table name

**Response:**
```json
{
  "id": "123",
  "name": "John Doe",
  "email": "john@example.com",
  "created": "2023-05-14T12:34:56.789Z"
}
```

### PUT `/databases/:dbType/:dbName/records/:id`

Updates a record.

**Request:**
```json
{
  "collection": "users",
  "data": {
    "name": "John Doe",
    "email": "john.updated@example.com"
  }
}
```

**Response:**
```json
{
  "id": "123",
  "name": "John Doe",
  "email": "john.updated@example.com",
  "created": "2023-05-14T12:34:56.789Z",
  "updated": "2023-05-14T13:45:67.890Z"
}
```

### DELETE `/databases/:dbType/:dbName/records/:id`

Deletes a record.

**Query Parameters:**
- `collection`: The collection/table name

**Response:**
```json
{
  "success": true,
  "message": "Record deleted successfully"
}
```

### POST `/databases/:dbType/:dbName/query`

Queries records in a collection.

**Request:**
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

**Response:**
```json
{
  "results": [
    {
      "id": "123",
      "name": "John Doe",
      "email": "john@example.com",
      "created": "2023-05-14T12:34:56.789Z"
    }
  ],
  "total": 1,
  "limit": 10,
  "offset": 0
}
```

## Health and Diagnostics

### GET `/health`

Gets the health status of the API and connected databases.

**Response:**
```json
{
  "status": "ok",
  "version": "1.0.0",
  "uptime": 3600,
  "databases": {
    "postgresql": {
      "status": "connected",
      "latency": 5
    },
    "mongodb": {
      "status": "disconnected",
      "error": "Connection string not provided"
    }
  }
}
```

## Adapter Management

### POST `/adapters`

Registers a custom database adapter.

**Request:**
```json
{
  "name": "custom-db",
  "type": "nosql",
  "connectionConfig": {
    "url": "https://custom-db.example.com"
  }
}
```

**Response:**
```json
{
  "success": true,
  "message": "Custom adapter 'custom-db' registered successfully",
  "adapter": {
    "type": "nosql",
    "connectionConfig": {
      "url": "https://custom-db.example.com"
    },
    "implementation": "default",
    "created": "2023-05-14T12:34:56.789Z"
  }
}
```

## Connection Management

### POST `/connections`

Creates a new database connection.

**Request:**
```json
{
  "name": "my-postgres",
  "type": "postgresql",
  "connectionString": "postgresql://user:password@host:port/database"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Connection 'my-postgres' created successfully",
  "connection": {
    "name": "my-postgres",
    "type": "postgresql",
    "status": "connected"
  }
}
```

### GET `/connections`

Lists all registered connections.

**Response:**
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