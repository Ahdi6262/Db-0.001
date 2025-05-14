/**
 * User-friendly example of how to use the Unified Database API Client
 * This demonstrates the key features that make this library powerful:
 * 1. Multiple database support through a single API
 * 2. Dynamic matrix operations for n-dimensional data
 * 3. Dynamic configuration and customization
 */

const { UnifiedDBClient } = require('./unified_db_client');

// Create a client instance
const client = new UnifiedDBClient({
  baseUrl: 'http://localhost:8000',
  defaultDbType: 'postgresql',
  debug: true
});

async function demoPostgreSQLOperations() {
  console.log('===== PostgreSQL Operations Demo =====');
  
  // Get a PostgreSQL database interface
  const postgres = client.database('postgresql', 'postgres');
  
  try {
    // Create a new record
    const newUser = await postgres.createRecord('users', {
      name: 'John Doe',
      email: 'john@example.com',
      created_at: new Date().toISOString()
    });
    console.log('Created user:', newUser);
    
    // Query records
    const users = await postgres.queryRecords('users', { name: 'John Doe' });
    console.log('Found users:', users);
    
    // Execute raw SQL (for advanced operations)
    const result = await postgres.executeRawQuery(
      'SELECT COUNT(*) as user_count FROM users WHERE name LIKE $1',
      ['John%']
    );
    console.log('User count:', result);
    
  } catch (error) {
    console.error('PostgreSQL operations failed:', error.message);
  }
}

async function demoMatrixOperations() {
  console.log('\n===== Infinite Matrix Operations Demo =====');
  
  try {
    // Create a matrix for financial data with 3 dimensions
    await client._fetch('/matrix', {
      method: 'POST',
      body: JSON.stringify({
        name: 'financial-data',
        dimensions: [
          { name: 'time', type: 'numeric', min: 0, max: 365 }, // Days of year
          { name: 'asset', type: 'numeric', min: 0, max: 1000 }, // Asset ID
          { name: 'metric', type: 'numeric', min: 0, max: 10 }  // Different metrics (price, volume, etc.)
        ]
      })
    });
    
    // Get matrix interface
    const financialMatrix = client.matrix('financial-data');
    
    // Store some financial data
    await financialMatrix.setValue([1, 100, 0], { price: 150.50, currency: 'USD' });
    await financialMatrix.setValue([1, 100, 1], { volume: 10000, currency: 'USD' });
    await financialMatrix.setValue([2, 100, 0], { price: 152.75, currency: 'USD' });
    
    // Query for price changes
    const priceData = await financialMatrix.query({
      asset: 100,
      metric: 0
    });
    
    console.log('Price data for asset 100:', priceData.results.map(r => ({
      day: r.coordinates[0],
      price: r.value.price
    })));
    
    // Add a new dimension for geographic region
    await financialMatrix.addDimension({
      name: 'region',
      type: 'numeric',
      min: 0,
      max: 5
    });
    
    // Store data with region
    await financialMatrix.setValue([3, 100, 0, 1], { 
      price: 153.25, 
      currency: 'USD',
      region: 'North America'
    });
    
    console.log('Financial matrix now has 4 dimensions');
    
  } catch (error) {
    console.error('Matrix operations failed:', error.message);
  }
}

async function demoCustomConfiguration() {
  console.log('\n===== Dynamic Configuration Demo =====');
  
  try {
    // Update performance settings
    const updateResult = await client._fetch('/config', {
      method: 'PATCH',
      body: JSON.stringify({
        section: 'performance',
        updates: {
          cacheSize: 5000,
          queryTimeout: 120000 // 2 minutes
        }
      })
    });
    console.log('Updated configuration:', updateResult.success);
    
    // Register a custom database adapter
    const adapterResult = await client._fetch('/adapters', {
      method: 'POST',
      body: JSON.stringify({
        name: 'custom-time-series',
        type: 'timeseries',
        connectionConfig: {
          url: 'https://custom-time-series-db.example.com',
          authType: 'token'
        }
      })
    });
    console.log('Registered custom adapter:', adapterResult.success);
    
    // Get current system status
    const config = await client._fetch('/config');
    console.log('System configuration:', {
      version: config.version,
      cacheSize: config.dynamic.performance.cacheSize,
      customAdapters: Object.keys(config.dynamic.activeAdapters)
    });
    
  } catch (error) {
    console.error('Configuration operations failed:', error.message);
  }
}

// Run the demos
async function runAllDemos() {
  try {
    // These demos would work if the PostgreSQL DB is properly configured
    // await demoPostgreSQLOperations();
    
    // These demos work with our InfiniteMatrix implementation
    await demoMatrixOperations();
    
    // These demos showcase dynamic configuration
    await demoCustomConfiguration();
    
    console.log('\nAll demos completed successfully!');
  } catch (error) {
    console.error('Demo failed:', error);
  }
}

runAllDemos();