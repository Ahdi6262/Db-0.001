# Quick Start Guide

This guide will help you get started with the Unified Database API quickly. You'll learn how to install the system, connect to databases, and perform basic operations.

## Prerequisites

Before you begin, ensure you have:

- Node.js (v14.0.0 or later)
- npm (v6.0.0 or later)
- Access to at least one database (PostgreSQL, MongoDB, etc.)

## Installation

First, install the Unified Database API:

```bash
# Clone the repository
git clone https://github.com/Ahdi6262/Db-0.001.git
cd Db-0.001

# Install dependencies
npm install
```

## Configuration

Create a `.env` file in the root directory with your database connection information:

```
# PostgreSQL Configuration
DATABASE_URL=postgresql://username:password@localhost:5432/database

# MongoDB Configuration (optional)
MONGODB_URI=mongodb://username:password@localhost:27017/database

# Server Configuration
PORT=8000
```

## Start the Server

Start the API server:

```bash
node unified_db_api_server.js
```

The server will start on port 8000 (or the port specified in your `.env` file).

## Basic Usage

### Creating a Client

First, create a client to interact with the API:

```javascript
const { UnifiedDBClient } = require("./unified_db_client");

const client = new UnifiedDBClient({
  baseUrl: "http://localhost:8000",
  defaultDbType: "postgresql",
  defaultDbName: "postgres",
  debug: true,
});
```

### Check API Health

Verify that the API is running and connected to your databases:

```javascript
async function checkHealth() {
  try {
    const health = await client.getHealth();
    console.log("API Health:", health);
  } catch (error) {
    console.error("Failed to check health:", error);
  }
}

checkHealth();
```

### Working with SQL Databases

```javascript
async function workWithPostgres() {
  // Get a PostgreSQL database interface
  const postgres = client.database("postgresql", "postgres");

  try {
    // Create a new record
    const newUser = await postgres.createRecord("users", {
      name: "John Doe",
      email: "john@example.com",
    });
    console.log("Created user:", newUser);

    // Query records
    const users = await postgres.queryRecords("users", { name: "John Doe" });
    console.log("Found users:", users);

    // Update a record
    await postgres.updateRecord("users", newUser.id, {
      name: "John Doe",
      email: "john.updated@example.com",
    });

    // Get a record by ID
    const user = await postgres.getRecord("users", newUser.id);
    console.log("Updated user:", user);

    // Delete a record
    await postgres.deleteRecord("users", newUser.id);
    console.log("User deleted");
  } catch (error) {
    console.error("PostgreSQL operations failed:", error);
  }
}

workWithPostgres();
```

### Working with MongoDB

```javascript
async function workWithMongoDB() {
  // Get a MongoDB database interface
  const mongodb = client.database("mongodb", "mydb");

  try {
    // Create a new document
    const newProduct = await mongodb.createRecord("products", {
      name: "Awesome Product",
      price: 99.99,
      tags: ["electronics", "gadgets"],
    });
    console.log("Created product:", newProduct);

    // Query documents
    const products = await mongodb.queryRecords("products", {
      price: { $gt: 50 },
    });
    console.log("Found products:", products);

    // Aggregate documents
    const results = await mongodb.aggregate("products", [
      { $match: { price: { $gt: 50 } } },
      { $group: { _id: null, avgPrice: { $avg: "$price" } } },
    ]);
    console.log("Average price:", results[0].avgPrice);

    // Upsert a document
    await mongodb.upsertRecord(
      "products",
      { name: "Awesome Product" },
      { name: "Awesome Product", price: 89.99 },
    );
    console.log("Product upserted");
  } catch (error) {
    console.error("MongoDB operations failed:", error);
  }
}

workWithMongoDB();
```

### Working with the Infinite Matrix

```javascript
async function workWithMatrix() {
  try {
    // Create a matrix
    await client._fetch("/matrix", {
      method: "POST",
      body: JSON.stringify({
        name: "demo-matrix",
        dimensions: [
          { name: "x", type: "numeric", min: 0, max: 100 },
          { name: "y", type: "numeric", min: 0, max: 100 },
          { name: "z", type: "numeric", min: 0, max: 100 },
        ],
      }),
    });
    console.log("Matrix created");

    // Get a matrix interface
    const matrix = client.matrix("demo-matrix");

    // Set values
    await matrix.setValue([10, 20, 30], { data: "Hello, Matrix!" });
    await matrix.setValue([15, 25, 35], { data: "Another point" });

    // Get a value
    const value = await matrix.getValue([10, 20, 30]);
    console.log("Value at [10, 20, 30]:", value.value.data);

    // Query values
    const results = await matrix.query({
      x: { min: 5, max: 20 },
      y: { min: 15, max: 30 },
    });
    console.log("Query results:", results.count);
    results.results.forEach((result) => {
      console.log(
        `Value at [${result.coordinates.join(", ")}]: ${result.value.data}`,
      );
    });

    // Add a dimension
    await matrix.addDimension({
      name: "time",
      type: "numeric",
      min: 0,
      max: 1000,
    });
    console.log("Added time dimension");

    // Set values in the 4D matrix
    await matrix.setValue([10, 20, 30, 5], { data: "Time point 5" });
  } catch (error) {
    console.error("Matrix operations failed:", error);
  }
}

workWithMatrix();
```

## Dynamic Configuration

You can configure the API dynamically at runtime:

```javascript
async function updateConfiguration() {
  try {
    // Update performance settings
    await client._fetch("/config", {
      method: "PATCH",
      body: JSON.stringify({
        section: "performance",
        updates: {
          cacheSize: 5000,
          queryTimeout: 60000, // 1 minute
        },
      }),
    });
    console.log("Configuration updated");

    // Get current configuration
    const config = await client._fetch("/config");
    console.log("New cache size:", config.dynamic.performance.cacheSize);
  } catch (error) {
    console.error("Configuration update failed:", error);
  }
}

updateConfiguration();
```

## Creating a Custom Adapter

You can register custom database adapters:

```javascript
async function registerCustomAdapter() {
  try {
    await client._fetch("/adapters", {
      method: "POST",
      body: JSON.stringify({
        name: "custom-db",
        type: "nosql",
        connectionConfig: {
          url: "https://custom-db.example.com",
          options: {
            timeout: 5000,
          },
        },
      }),
    });
    console.log("Custom adapter registered");
  } catch (error) {
    console.error("Adapter registration failed:", error);
  }
}

registerCustomAdapter();
```

## Running Examples

The repository includes several example files that demonstrate the API's capabilities:

- `client_example.js`: Basic client usage examples
- `test_infinite_matrix.js`: Infinite Matrix examples

Run these examples to see the API in action:

```bash
node client_example.js
node test_infinite_matrix.js
```

## Next Steps

Now that you have the basics, you can:

- Explore the [API Reference](API-Reference) for details on all available endpoints
- Learn more about the [Infinite Matrix](Infinite-Matrix) feature
- Study the [Architecture](Architecture) to understand the system design
- Read the [Database Adapters](Database-Adapters) documentation to learn how to extend the system

Happy coding!
