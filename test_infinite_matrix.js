/**
 * Test client for the Unified Database API's Infinite Matrix capabilities
 * This demonstrates how to use the dynamic customization features
 */

const { UnifiedDBClient } = require("./unified_db_client");

// Create a client with debugging enabled
const client = new UnifiedDBClient({
  baseUrl: "http://localhost:8000",
  debug: true,
});

async function runMatrixDemo() {
  console.log("===== Unified Database API with Infinite Matrix Demo =====");

  try {
    // 1. Get current system configuration
    console.log("Fetching system configuration...");
    const config = await client._fetch("/config");
    console.log("System version:", config.version);
    console.log(
      "Supported databases:",
      JSON.stringify(config.supportedDatabases),
    );

    // 2. Create a new 3D matrix
    console.log("\nCreating a 3D matrix...");
    const matrix3dResponse = await client._fetch("/matrix", {
      method: "POST",
      body: JSON.stringify({
        name: "demo-3d-matrix",
        dimensions: [
          { name: "x", type: "numeric", min: 0, max: 100 },
          { name: "y", type: "numeric", min: 0, max: 100 },
          { name: "z", type: "numeric", min: 0, max: 100 },
        ],
        scaling: { auto: true, growthFactor: 2 },
      }),
    });
    console.log("Matrix created:", matrix3dResponse.success);

    // 3. Set values in the matrix
    console.log("\nSetting values in the matrix...");
    const setValueResponse = await client._fetch(
      "/matrix/demo-3d-matrix/value",
      {
        method: "POST",
        body: JSON.stringify({
          coordinates: [10, 20, 30],
          value: {
            data: "Sample data at point (10,20,30)",
            timestamp: new Date().toISOString(),
            metadata: { type: "demo-point" },
          },
        }),
      },
    );
    console.log("Value set:", setValueResponse.success);

    // 4. Set another value
    await client._fetch("/matrix/demo-3d-matrix/value", {
      method: "POST",
      body: JSON.stringify({
        coordinates: [15, 25, 35],
        value: {
          data: "Another data point at (15,25,35)",
          timestamp: new Date().toISOString(),
          metadata: { type: "demo-point" },
        },
      }),
    });

    // 5. Get matrix info after adding values
    console.log("\nGetting matrix info...");
    const matrixInfo = await client._fetch("/matrix/demo-3d-matrix");
    console.log("Matrix dimensions:", matrixInfo.dimensions.length);
    console.log("Cell count:", matrixInfo.metadata.cellCount);

    // 6. Retrieve a value from specific coordinates
    console.log("\nRetrieving value from coordinates [10, 20, 30]...");
    const value = await client._fetch(
      "/matrix/demo-3d-matrix/value?coordinates=10,20,30",
    );
    console.log("Retrieved value:", value.value.data);

    // 7. Add a new dimension to the matrix (making it 4D)
    console.log("\nAdding a new dimension to create a 4D matrix...");
    const addDimensionResponse = await client._fetch(
      "/matrix/demo-3d-matrix/dimensions",
      {
        method: "POST",
        body: JSON.stringify({
          name: "time",
          type: "numeric",
          min: 0,
          max: 1000,
        }),
      },
    );
    console.log("Dimension added:", addDimensionResponse.success);

    // 8. Set a value in the 4D matrix
    console.log("\nSetting a value in the 4D matrix...");
    await client._fetch("/matrix/demo-3d-matrix/value", {
      method: "POST",
      body: JSON.stringify({
        coordinates: [10, 20, 30, 5],
        value: {
          data: "Time-series data at point (10,20,30) at time 5",
          timestamp: new Date().toISOString(),
          metadata: { type: "time-series-point" },
        },
      }),
    });

    // 9. Query the matrix for values
    console.log("\nQuerying the matrix...");
    const queryResponse = await client._fetch("/matrix/demo-3d-matrix/query", {
      method: "POST",
      body: JSON.stringify({
        query: {
          x: { min: 5, max: 15 },
          y: { min: 15, max: 25 },
          z: { min: 25, max: 35 },
        },
      }),
    });
    console.log("Query results count:", queryResponse.count);
    console.log(
      "First result:",
      queryResponse.results.length > 0
        ? queryResponse.results[0].value.data
        : "No results",
    );

    // 10. Test dynamic configuration update
    console.log("\nUpdating system configuration...");
    const updateConfigResponse = await client._fetch("/config", {
      method: "PATCH",
      body: JSON.stringify({
        section: "performance",
        updates: {
          cacheSize: 2000,
          queryTimeout: 60000,
        },
      }),
    });
    console.log("Configuration updated:", updateConfigResponse.success);

    // 11. Verify configuration changes
    const updatedConfig = await client._fetch("/config");
    console.log("New cache size:", updatedConfig.dynamic.performance.cacheSize);
    console.log(
      "New query timeout:",
      updatedConfig.dynamic.performance.queryTimeout,
    );

    console.log("\n===== Demo Completed Successfully =====");
  } catch (error) {
    console.error("Error in demo:", error.message);
  }
}

// Run the demo
runMatrixDemo();
