# Installation Guide

This guide will walk you through the installation and setup process for the Unified Database API.

## Prerequisites

Before you begin, ensure you have the following installed:

- Node.js (v14.0.0 or later)
- npm (v6.0.0 or later)
- PostgreSQL (for SQL database support)
- MongoDB (optional, for document database support)
- Neo4j (optional, for graph database support)
- InfluxDB (optional, for time-series database support)
- Qdrant (optional, for vector database support)

## Installation Steps

### 1. Clone the Repository

```bash
git clone https://github.com/Ahdi6262/Db-0.001.git
cd Db-0.001
```

### 2. Install Dependencies

```bash
npm install
```

This will install all required Node.js dependencies, including:

- express
- cors
- pg (PostgreSQL client)
- mongodb
- and other required libraries

### 3. Configure Environment Variables

Create a `.env` file in the root directory and configure the following variables:

```
# PostgreSQL Configuration
DATABASE_URL=postgresql://username:password@localhost:5432/database

# MongoDB Configuration (optional)
MONGODB_URI=mongodb://username:password@localhost:27017/database

# Neo4j Configuration (optional)
NEO4J_URI=neo4j://localhost:7687
NEO4J_USERNAME=neo4j
NEO4J_PASSWORD=password

# InfluxDB Configuration (optional)
INFLUXDB_URL=http://localhost:8086
INFLUXDB_TOKEN=your-influxdb-token
INFLUXDB_ORG=your-organization
INFLUXDB_BUCKET=your-bucket

# Server Configuration
PORT=8000
```

Adjust the values according to your database configurations.

## Database Setup

### PostgreSQL Setup

1. Install PostgreSQL on your system (if not already installed)
2. Create a new database:
   ```bash
   createdb unified_db_api
   ```
3. The system will automatically create required tables on first run

### MongoDB Setup (Optional)

1. Install MongoDB on your system (if not already installed)
2. Create a new database:
   ```bash
   mongosh
   > use unified_db_api
   ```
3. No additional setup required as MongoDB creates collections dynamically

## Starting the Server

Once you've completed the installation and configuration, you can start the server:

```bash
node unified_db_api_server.js
```

By default, the server will listen on port 8000. You can access the API at http://localhost:8000.

## Verifying the Installation

To verify that the installation was successful, you can test the health endpoint:

```bash
curl http://localhost:8000/health
```

You should see a response indicating the health status of the server and connected databases:

```json
{
  "status": "ok",
  "version": "1.0.0",
  "databases": {
    "postgresql": {
      "status": "connected"
    },
    "mongodb": {
      "status": "connected"
    }
  }
}
```

## Installing the Client Library

The client library is included in the repository. To use it in your application:

```javascript
const { UnifiedDBClient } = require("./unified_db_client");

// Create a client instance
const client = new UnifiedDBClient({
  baseUrl: "http://localhost:8000",
  debug: true,
});

// Test the connection
async function testConnection() {
  try {
    const health = await client.getHealth();
    console.log("Connected to Unified Database API:", health);
  } catch (error) {
    console.error("Failed to connect:", error);
  }
}

testConnection();
```

## Troubleshooting

### Common Installation Issues

#### PostgreSQL Connection Issues

If you see an error like:

```
Failed to connect to PostgreSQL: ECONNREFUSED
```

Check that:

1. PostgreSQL is running
2. Your DATABASE_URL is correct
3. Your PostgreSQL user has appropriate permissions

#### MongoDB Connection Issues

If you see an error like:

```
Failed to connect to MongoDB: MongoError: failed to connect to server
```

Check that:

1. MongoDB is running
2. Your MONGODB_URI is correct
3. Authentication details are correct if MongoDB requires authentication

### Getting Help

If you encounter any issues during installation, please:

1. Check the [Troubleshooting](Troubleshooting) page for common solutions
2. Open an issue on the GitHub repository with details about your problem
3. Join our community Discord server for real-time help

## Next Steps

Now that you have installed the Unified Database API, you can:

- Follow the [Quick Start Tutorial](Quick-Start) to learn how to use the API
- Explore the [API Reference](API-Reference) for details on all available endpoints
- Learn about the [Infinite Matrix](Infinite-Matrix) feature for n-dimensional data storage
- Read the [Architecture](Architecture) documentation to understand the system design
