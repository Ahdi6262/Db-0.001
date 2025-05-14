const express = require("express");
const { Pool } = require("pg");
const app = express();
const port = 8000;

// Initialize PostgreSQL client
const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
});

// Test database connection
pool.query("SELECT NOW()", (err, res) => {
  if (err) {
    console.error("Error connecting to the database:", err);
  } else {
    console.log("Database connected successfully:", res.rows[0]);
  }
});

// Middleware
app.use(express.json());

// Health check endpoint
app.get("/health", async (req, res) => {
  try {
    const result = await pool.query("SELECT 1");
    res.json({
      status: "ok",
      database_connected: result ? true : false,
    });
  } catch (err) {
    console.error(err);
    res.status(500).json({
      status: "error",
      database_connected: false,
      error: err.message,
    });
  }
});

// Get tables endpoint
app.get("/tables", async (req, res) => {
  try {
    const result = await pool.query(
      "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'",
    );
    const tables = result.rows.map((row) => row.table_name);
    res.json({ tables });
  } catch (err) {
    console.error(err);
    res.status(500).json({ error: err.message });
  }
});

// Create a record (if table doesn't exist, create it)
app.post("/records", async (req, res) => {
  const { collection, data } = req.body;
  if (!collection || !data) {
    return res
      .status(400)
      .json({ error: "Collection name and data are required" });
  }

  try {
    // Create table if it doesn't exist
    await pool.query(`
      CREATE TABLE IF NOT EXISTS ${collection} (
        id TEXT PRIMARY KEY,
        data JSONB NOT NULL
      )
    `);

    // Generate a random ID
    const id = require("crypto").randomUUID();

    // Insert the record
    await pool.query(
      `
      INSERT INTO ${collection} (id, data) VALUES ($1, $2)
    `,
      [id, data],
    );

    res.status(201).json({ id, data });
  } catch (err) {
    console.error(err);
    res.status(500).json({ error: err.message });
  }
});

// Get a record
app.get("/records/:id", async (req, res) => {
  const { id } = req.params;
  const { collection } = req.query;

  if (!collection) {
    return res.status(400).json({ error: "Collection name is required" });
  }

  try {
    // Check if table exists
    const tableCheck = await pool.query(
      `
      SELECT EXISTS (
        SELECT FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name = $1
      )
    `,
      [collection],
    );

    if (!tableCheck.rows[0].exists) {
      return res.status(404).json({ error: "Collection not found" });
    }

    // Get the record
    const result = await pool.query(
      `
      SELECT data FROM ${collection} WHERE id = $1
    `,
      [id],
    );

    if (result.rows.length === 0) {
      return res.status(404).json({ error: "Record not found" });
    }

    res.json({ id, data: result.rows[0].data });
  } catch (err) {
    console.error(err);
    res.status(500).json({ error: err.message });
  }
});

// Start the server
app.listen(port, "0.0.0.0", () => {
  console.log(`Unified DB API server running at http://0.0.0.0:${port}`);
});
