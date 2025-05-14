# Infinite Matrix Guide

The Infinite Matrix is a powerful feature of the Unified Database API that provides n-dimensional data storage and manipulation capabilities. This guide explains how to use the Infinite Matrix effectively.

## Concepts

### What is the Infinite Matrix?

The Infinite Matrix is a data structure that allows you to store and retrieve values in an n-dimensional space. Unlike traditional arrays or matrices, the Infinite Matrix:

- Supports any number of dimensions
- Dimensions can be added or removed dynamically
- Automatically scales to accommodate new data
- Efficiently stores sparse data (only non-empty cells consume memory)
- Can be persisted to different storage backends

### Dimensions

Each dimension in the matrix has:

- A name (e.g., "time", "user", "metric")
- A type (currently "numeric" is supported)
- A minimum value
- A maximum value
- A default value

### Coordinates

Values in the matrix are accessed using coordinates, which are arrays of values corresponding to positions along each dimension:

```javascript
// 3D coordinates: [x, y, z]
[10, 20, 30][
  // 4D coordinates: [time, user, metric, region]
  (1, 123, 5, 2)
];
```

## Creating a Matrix

### Basic Creation

```javascript
// Using the API directly
await fetch("http://localhost:8000/matrix", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    name: "my-matrix",
    dimensions: [
      { name: "x", type: "numeric", min: 0, max: 100 },
      { name: "y", type: "numeric", min: 0, max: 100 },
      { name: "z", type: "numeric", min: 0, max: 100 },
    ],
  }),
});

// Using the client library
const client = new UnifiedDBClient({
  baseUrl: "http://localhost:8000",
});

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
```

### Advanced Creation Options

You can customize the matrix behavior with additional options:

```javascript
await client._fetch("/matrix", {
  method: "POST",
  body: JSON.stringify({
    name: "custom-matrix",
    dimensions: [
      { name: "time", type: "numeric", min: 0, max: 1000 },
      { name: "user", type: "numeric", min: 0, max: 10000 },
      { name: "metric", type: "numeric", min: 0, max: 50 },
    ],
    storage: {
      type: "memory", // or 'database'
    },
    scaling: {
      auto: true,
      growthFactor: 2,
      maxSize: Number.MAX_SAFE_INTEGER,
    },
  }),
});
```

## Working with Matrix Data

### Setting Values

```javascript
const matrix = client.matrix("my-matrix");

// Set a value at specific coordinates
await matrix.setValue([10, 20, 30], {
  data: "Hello from the matrix!",
  timestamp: new Date().toISOString(),
});
```

### Getting Values

```javascript
// Get a value at specific coordinates
const value = await matrix.getValue([10, 20, 30]);
console.log(value.value.data); // 'Hello from the matrix!'
```

### Querying Values

You can query the matrix for values matching certain criteria:

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

### Retrieving Matrix Information

```javascript
// Get information about the matrix
const info = await matrix.getInfo();
console.log(`Matrix has ${info.dimensions.length} dimensions`);
console.log(`Cell count: ${info.metadata.cellCount}`);
```

## Manipulating Dimensions

### Adding a Dimension

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

### Removing a Dimension

```javascript
// Remove a dimension from the matrix
await matrix.removeDimension("time");
```

### Resizing Dimensions

```javascript
// Resize the matrix dimensions
await matrix.resize([
  { name: "x", min: 0, max: 200 },
  { name: "y", min: 0, max: 200 },
]);
```

## Storage Management

### Changing Storage Type

```javascript
// Change the matrix storage from memory to database
await matrix.changeStorage("database", {
  collection: "matrix_data",
  dbType: "mongodb",
});

// Or back to memory
await matrix.changeStorage("memory");
```

## Use Cases

### Time-Series Data

The Infinite Matrix is excellent for time-series data:

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

// Plot the CPU usage over time
console.log(
  cpuUsage.results.map((r) => ({
    time: r.coordinates[0],
    value: r.value.value,
  })),
);
```

### Multi-Dimensional Analytics

Perfect for complex analytics across multiple dimensions:

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

// Get all actions of a specific type across all users
const actionStats = await matrix.query({
  action: 1,
  day: { min: 1, max: 7 }, // First week
});

// Analyze usage by hour of day
const hourlyUsage = await matrix.query({
  day: 1,
  hour: 14,
});
```

### Feature Matrix Comparisons

Use the Infinite Matrix to compare features across products:

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
await matrix.setValue([2, 10, 1], { supported: true, since: "2022-11-15" });

// Compare feature support across products
const featureSupport = await matrix.query({
  feature: 10,
  version: 1,
});

// Check which features are supported in a specific product version
const productFeatures = await matrix.query({
  product: 1,
  version: 1,
});

// Filter for only supported features
const supportedFeatures = productFeatures.results.filter(
  (r) => r.value.supported,
);
```

## Advanced Usage

### Dynamic Dimension Mapping

You can create a mapping layer to translate between human-readable identifiers and numeric coordinates:

```javascript
class DimensionMapper {
  constructor() {
    this.maps = {};
    this.reverseMaps = {};
  }

  createMap(dimension, values) {
    this.maps[dimension] = {};
    this.reverseMaps[dimension] = {};

    values.forEach((value, index) => {
      this.maps[dimension][value] = index;
      this.reverseMaps[dimension][index] = value;
    });
  }

  toCoordinate(dimension, value) {
    return this.maps[dimension][value];
  }

  fromCoordinate(dimension, coordinate) {
    return this.reverseMaps[dimension][coordinate];
  }

  translateCoordinates(namedCoordinates) {
    const result = [];

    for (const dimension in namedCoordinates) {
      const dimensionIndex = Object.keys(this.maps).indexOf(dimension);
      result[dimensionIndex] = this.toCoordinate(
        dimension,
        namedCoordinates[dimension],
      );
    }

    return result;
  }

  translateResults(results) {
    return results.map((result) => {
      const namedCoordinates = {};

      Object.keys(this.maps).forEach((dimension, index) => {
        namedCoordinates[dimension] = this.fromCoordinate(
          dimension,
          result.coordinates[index],
        );
      });

      return {
        namedCoordinates,
        value: result.value,
      };
    });
  }
}

// Usage
const mapper = new DimensionMapper();

// Create mappings
mapper.createMap("product", ["ProductA", "ProductB", "ProductC"]);
mapper.createMap("feature", ["Authentication", "Authorization", "Reporting"]);
mapper.createMap("version", ["v1.0", "v1.1", "v2.0"]);

// Set a value using human-readable identifiers
const coordinates = mapper.translateCoordinates({
  product: "ProductA",
  feature: "Authentication",
  version: "v1.0",
});

await matrix.setValue(coordinates, { supported: true });

// Query and translate results
const results = await matrix.query({
  product: mapper.toCoordinate("product", "ProductA"),
});

const translatedResults = mapper.translateResults(results.results);
console.log(translatedResults);
```

### Batch Operations

For efficiency, you can batch operations:

```javascript
// Batch set multiple values
async function batchSetValues(matrix, valuesList) {
  const promises = valuesList.map((item) =>
    matrix.setValue(item.coordinates, item.value),
  );

  return Promise.all(promises);
}

// Example usage
await batchSetValues(matrix, [
  { coordinates: [1, 1, 1], value: { data: "Value 1" } },
  { coordinates: [1, 1, 2], value: { data: "Value 2" } },
  { coordinates: [1, 2, 1], value: { data: "Value 3" } },
]);
```

## Performance Considerations

1. **Use Sparse Data**: The matrix is optimized for sparse data (most cells empty)
2. **Limit Query Ranges**: Narrow query ranges to improve performance
3. **Use Appropriate Storage**: Memory storage is faster but limited by available RAM
4. **Consider Dimension Order**: Most frequently queried dimensions should come first
5. **Batch Operations**: Group multiple operations together

## Matrix Implementations

### Memory-Based Matrix

Best for:

- Temporary data
- Small to medium datasets
- High performance requirements

### Database-Based Matrix

Best for:

- Persistent data
- Large datasets
- Distributed access

## Error Handling

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

## Debugging Matrix Operations

The client library supports debugging:

```javascript
const client = new UnifiedDBClient({
  baseUrl: "http://localhost:8000",
  debug: true, // Enable debug logging
});

// This will log all matrix operations
const matrix = client.matrix("my-matrix");
await matrix.setValue([1, 2, 3], { test: true });
```

## Best Practices

1. **Plan Your Dimensions**: Carefully consider which dimensions you need
2. **Use Meaningful Dimension Names**: Choose clear, descriptive names
3. **Set Reasonable Bounds**: Set min/max values that cover your expected data range
4. **Consider Storage Requirements**: Choose the appropriate storage backend for your data volume
5. **Index Hot Dimensions**: Place frequently accessed dimensions first in your coordinate arrays
6. **Use Query Caching**: Cache frequent queries for better performance
7. **Monitor Cell Count**: Keep track of cell count to avoid excessive memory usage
