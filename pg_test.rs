use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing PostgreSQL connection");
    
    // Check if DATABASE_URL is set
    match env::var("DATABASE_URL") {
        Ok(url) => println!("Found DATABASE_URL: {}", url),
        Err(_) => println!("DATABASE_URL not set"),
    }
    
    // Check other PG environment variables
    for var in &["PGHOST", "PGUSER", "PGPASSWORD", "PGDATABASE", "PGPORT"] {
        match env::var(var) {
            Ok(val) => println!("{} is set to: {}", var, val),
            Err(_) => println!("{} is not set", var),
        }
    }
    
    println!("Test complete");
    Ok(())
}