# Infinite Matrix Guide

The Infinite Matrix is a powerful feature of the Unified Database API that provides n-dimensional data storage and manipulation capabilities. This guide explains how to use the Infinite Matrix effectively.

## What is the Infinite Matrix?

The Infinite Matrix is a flexible data structure that lets you:

- Store and retrieve data in n-dimensional space
- Dynamically add or remove dimensions
- Automatically scale to accommodate new data
- Efficiently store sparse data (only non-empty cells use memory)
- Persist data to different storage backends

## Key Concepts

### Dimensions

Each dimension has:

- A name (e.g., "time", "user", "metric")
- A type (currently only "numeric" is supported)
- Minimum and maximum boundaries
- A default value for empty cells

### Coordinates

Values are accessed using coordinates - arrays of values corresponding to positions in each dimension:

```javascript
// 3D coordinates: [x, y, z]
[10, 20, 30][
  // 4D coordinates: [time, user, metric, region]
  (1, 123, 5, 2)
];
```

## Getting Started

### Creating a Matrix

```javascript
// Using the client library
const client = new UnifiedDBClient({
  baseUrl: "http://localhost:8000",
});

// Create a 3D matrix
await client._fetch("/matrix", {
  method: "POST",
  body: JSON.stringify({
    name: "my-matrix",
    dimensions: [
      { name: "x", type: "numeric", min: 0, max: 100 },
      { name: "y", type: "numeric", min: 0, max: 100 },
      { name: "z", type: "numeric", min: 0, max: 100 },
    ],
  }),
});

// Get a matrix interface
const matrix = client.matrix("my-matrix");
```

### Storing Values

```javascript
// Set a value at specific coordinates
await matrix.setValue([10, 20, 30], {
  data: "Hello from the matrix!",
  timestamp: new Date().toISOString(),
});
```

### Retrieving Values

```javascript
// Get a value at specific coordinates
const value = await matrix.getValue([10, 20, 30]);
console.log(value.value.data); // 'Hello from the matrix!'
```

### Querying Values

```javascript
// Query for values in a range
const results = await matrix.query({
  x: { min: 5, max: 15 },
  y: 20,
  z: { min: 25, max: 35 },
});

// Process the results
results.results.forEach((result) => {
  console.log(
    `Value at [${result.coordinates.join(", ")}]: ${result.value.data}`,
  );
});
```

## Advanced Features

### Dimension Management

#### Adding Dimensions

```javascript
// Add a new dimension to the matrix
await matrix.addDimension({
  name: "time",
  type: "numeric",
  min: 0,
  max: 1000,
  default: 0,
});

// Now you can set values with the new dimension
await matrix.setValue([10, 20, 30, 5], {
  data: "Data at time point 5",
});
```

#### Removing Dimensions

```javascript
// Remove a dimension from the matrix
await matrix.removeDimension("time");
```

#### Resizing Dimensions

```javascript
// Resize the matrix dimensions
await matrix.resize([
  { name: "x", min: 0, max: 200 },
  { name: "y", min: 0, max: 200 },
]);
```

### Storage Management

You can change how the matrix data is stored:

```javascript
// Change the matrix storage from memory to database
await matrix.changeStorage("database", {
  collection: "matrix_data",
  dbType: "mongodb",
});

// Or back to memory
await matrix.changeStorage("memory");
```

## Common Use Cases

### Time-Series Data Analysis

Perfect for storing and analyzing time-series data:

```javascript
// Create a matrix for time-series data
await client._fetch("/matrix", {
  method: "POST",
  body: JSON.stringify({
    name: "metrics",
    dimensions: [
      { name: "time", type: "numeric", min: 0, max: 10000 }, // Time points
      { name: "server", type: "numeric", min: 0, max: 100 }, // Server IDs
      { name: "metric", type: "numeric", min: 0, max: 50 }, // Different metrics
    ],
  }),
});

const matrix = client.matrix("metrics");

// Record CPU usage for server 1 at time 100
await matrix.setValue([100, 1, 0], { value: 0.75 }); // Metric 0 = CPU

// Record memory usage for server 1 at time 100
await matrix.setValue([100, 1, 1], { value: 0.5 }); // Metric 1 = Memory

// Query CPU usage for server 1 over time
const cpuUsage = await matrix.query({
  server: 1,
  metric: 0,
  time: { min: 0, max: 1000 },
});
```

### Multi-Dimensional Analytics

Great for complex analytics across multiple dimensions:

```javascript
// Create a matrix for user analytics
await client._fetch("/matrix", {
  method: "POST",
  body: JSON.stringify({
    name: "user-analytics",
    dimensions: [
      { name: "user", type: "numeric", min: 0, max: 10000 }, // User IDs
      { name: "action", type: "numeric", min: 0, max: 100 }, // Action types
      { name: "day", type: "numeric", min: 0, max: 365 }, // Day of year
      { name: "hour", type: "numeric", min: 0, max: 24 }, // Hour of day
    ],
  }),
});

const matrix = client.matrix("user-analytics");

// Record user actions
await matrix.setValue([123, 1, 1, 14], { count: 5 }); // User 123, action 1, day 1, hour 14
await matrix.setValue([123, 2, 1, 15], { count: 3 }); // User 123, action 2, day 1, hour 15

// Get all actions for a specific user on a specific day
const userActions = await matrix.query({
  user: 123,
  day: 1,
});
```

### Feature Comparison Matrix

Perfect for comparing features across products:

```javascript
// Create a feature comparison matrix
await client._fetch("/matrix", {
  method: "POST",
  body: JSON.stringify({
    name: "feature-matrix",
    dimensions: [
      { name: "product", type: "numeric", min: 0, max: 100 }, // Product IDs
      { name: "feature", type: "numeric", min: 0, max: 500 }, // Feature IDs
      { name: "version", type: "numeric", min: 0, max: 100 }, // Version IDs
    ],
  }),
});

const matrix = client.matrix("feature-matrix");

// Record feature support
await matrix.setValue([1, 10, 1], { supported: true, since: "2023-01-01" });
await matrix.setValue([1, 11, 1], { supported: false, planned: "2023-06-01" });
```

## Best Practices

### Performance Optimization

1. **Use Sparse Data**: The matrix is optimized for sparse data (most cells empty)
2. **Limit Query Ranges**: Narrow query ranges to improve performance
3. **Use Appropriate Storage**: Memory storage is faster but limited by RAM
4. **Consider Dimension Order**: Most frequently queried dimensions should come first
5. **Batch Operations**: Group multiple operations together

### Error Handling

```javascript
try {
  await matrix.setValue([10, 20, 30], { data: "Test" });
} catch (error) {
  console.error(`Matrix operation failed: ${error.message}`);

  // Common errors
  if (error.message.includes("dimension")) {
    console.error(
      "Dimension error: Check your coordinates match the dimensions",
    );
  } else if (error.message.includes("storage")) {
    console.error("Storage error: Check your database connection");
  }
}
```

### Debugging Matrix Operations

```javascript
const client = new UnifiedDBClient({
  baseUrl: "http://localhost:8000",
  debug: true, // Enable debug logging
});

// This will log all matrix operations
const matrix = client.matrix("my-matrix");
await matrix.setValue([1, 2, 3], { test: true });
```

## Working with Large Datasets

For large datasets, consider:

1. **Using database storage**: Memory storage is limited by available RAM
2. **Batching operations**: Group operations to reduce network overhead
3. **Using sparse representations**: Only store non-empty cells
4. **Compressing values**: Compress values before storing them
5. **Sharding data**: Split large matrices across multiple instances

## Implementation Details

The Infinite Matrix is implemented in `infinite_matrix.js` and uses a sparse representation to efficiently store data. Each non-empty cell is stored with its coordinates as the key.

Memory constraints only apply to the number of non-empty cells, not the size of the dimensions. You can have dimensions with very large ranges without using excessive memory, as long as most cells remain empty.

## Further Reading

- [Client Library Documentation](Client-Library) - Learn more about the client library
- [API Reference](API-Reference) - Explore the matrix API endpoints
- [Architecture](Architecture) - Understand the system architecture
