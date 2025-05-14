# Unified Database API Documentation

Welcome to the comprehensive documentation for the Unified Database API with Infinite Matrix capabilities. This documentation will help you understand, install, configure, and use all features of the system.

## Documentation Overview

| Document | Description |
|----------|-------------|
| [README.md](README.md) | Project overview, installation instructions, and basic usage examples |
| [DOCUMENTATION.md](DOCUMENTATION.md) | Comprehensive technical documentation covering all aspects of the system |
| [API_REFERENCE.md](API_REFERENCE.md) | Detailed reference of all API endpoints with request/response examples |
| [ARCHITECTURE.md](ARCHITECTURE.md) | System architecture design, components, and data flow |
| [INFINITE_MATRIX_GUIDE.md](INFINITE_MATRIX_GUIDE.md) | Complete guide to using the n-dimensional Infinite Matrix |
| [client_example.js](client_example.js) | Example client application showing database and matrix operations |
| [test_infinite_matrix.js](test_infinite_matrix.js) | Test script demonstrating Infinite Matrix capabilities |

## Quick Start

1. **Installation**: 
   ```bash
   # Clone the repository
   git clone https://github.com/Ahdi6262/Db-0.001.git
   cd Db-0.001
   
   # Install dependencies
   npm install
   ```

2. **Start the Server**:
   ```bash
   node unified_db_api_server.js
   ```

3. **Use the Client Library**:
   ```javascript
   const { UnifiedDBClient } = require('./unified_db_client');
   
   const client = new UnifiedDBClient({
     baseUrl: 'http://localhost:8000',
     debug: true
   });
   
   // Create a matrix
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
   
   // Set a value
   const matrix = client.matrix('my-matrix');
   await matrix.setValue([10, 20, 30], { data: 'Hello, Matrix!' });
   ```

## Core Features

- **Unified Database Interface**: Work with multiple database types through a consistent API
- **Infinite Matrix**: Store and query n-dimensional data with dynamic dimensions
- **Dynamic Configuration**: Customize the system behavior at runtime
- **Type-Safe Client**: Use the client library for type-safe database interactions
- **Flexible Storage**: Choose from memory or database storage for different needs

## Supported Database Types

- **SQL**: PostgreSQL, MySQL
- **NoSQL**: MongoDB, Redis, Cassandra
- **Graph**: Neo4j
- **Time Series**: InfluxDB
- **Vector**: Qdrant

## Advanced Usage

For advanced usage examples and detailed documentation, see:

- [DOCUMENTATION.md](DOCUMENTATION.md) for complete technical details
- [API_REFERENCE.md](API_REFERENCE.md) for API endpoint specifications
- [INFINITE_MATRIX_GUIDE.md](INFINITE_MATRIX_GUIDE.md) for matrix operations
- [client_example.js](client_example.js) for practical examples

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.