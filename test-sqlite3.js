const sqlite3 = require('sqlite3').verbose();
const path = require('path');
const fs = require('fs');

console.log('Testing SQLite3 on Windows...\n');

// Test 1: Create an in-memory database
console.log('Test 1: Creating in-memory database...');
const memDb = new sqlite3.Database(':memory:', (err) => {
    if (err) {
        console.error('❌ Failed to create in-memory database:', err);
        return;
    }
    console.log('✓ In-memory database created successfully');
    
    // Test 2: Create a table
    console.log('\nTest 2: Creating table...');
    memDb.run(`
        CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT UNIQUE
        )
    `, (err) => {
        if (err) {
            console.error('❌ Failed to create table:', err);
            return;
        }
        console.log('✓ Table created successfully');
        
        // Test 3: Insert data
        console.log('\nTest 3: Inserting data...');
        const stmt = memDb.prepare('INSERT INTO users (name, email) VALUES (?, ?)');
        stmt.run('John Doe', 'john@example.com');
        stmt.run('Jane Smith', 'jane@example.com');
        stmt.finalize((err) => {
            if (err) {
                console.error('❌ Failed to insert data:', err);
                return;
            }
            console.log('✓ Data inserted successfully');
            
            // Test 4: Query data
            console.log('\nTest 4: Querying data...');
            memDb.all('SELECT * FROM users', [], (err, rows) => {
                if (err) {
                    console.error('❌ Failed to query data:', err);
                    return;
                }
                console.log('✓ Query executed successfully');
                console.log('Results:', rows);
                
                memDb.close((err) => {
                    if (err) {
                        console.error('❌ Failed to close in-memory database:', err);
                    }
                    
                    // Test 5: Create a file-based database
                    console.log('\nTest 5: Creating file-based database...');
                    const dbPath = path.join(__dirname, 'test.db');
                    
                    // Remove existing database if it exists
                    if (fs.existsSync(dbPath)) {
                        fs.unlinkSync(dbPath);
                    }
                    
                    const fileDb = new sqlite3.Database(dbPath, (err) => {
                        if (err) {
                            console.error('❌ Failed to create file database:', err);
                            return;
                        }
                        console.log(`✓ File database created at: ${dbPath}`);
                        
                        // Test 6: Operations on file database
                        console.log('\nTest 6: Testing file database operations...');
                        fileDb.serialize(() => {
                            fileDb.run(`
                                CREATE TABLE IF NOT EXISTS products (
                                    id INTEGER PRIMARY KEY,
                                    name TEXT,
                                    price REAL
                                )
                            `);
                            
                            const productStmt = fileDb.prepare('INSERT OR REPLACE INTO products VALUES (?, ?, ?)');
                            productStmt.run(1, 'Laptop', 999.99);
                            productStmt.run(2, 'Mouse', 29.99);
                            productStmt.run(3, 'Keyboard', 79.99);
                            productStmt.run(4, 'Monitor', 299.99);
                            productStmt.finalize();
                            
                            // Test 7: Prepared statements
                            console.log('\nTest 7: Testing prepared statements...');
                            fileDb.all('SELECT * FROM products WHERE price < ?', [100], (err, products) => {
                                if (err) {
                                    console.error('❌ Failed to execute prepared statement:', err);
                                    return;
                                }
                                console.log('✓ Prepared statement executed');
                                console.log('Affordable products:', products);
                                
                                // Test 8: Get all products
                                fileDb.all('SELECT * FROM products', [], (err, allProducts) => {
                                    if (err) {
                                        console.error('❌ Failed to get all products:', err);
                                        return;
                                    }
                                    console.log('\n✓ All products retrieved:');
                                    console.log('Products:', allProducts);
                                    
                                    fileDb.close((err) => {
                                        if (err) {
                                            console.error('❌ Failed to close file database:', err);
                                            return;
                                        }
                                        
                                        console.log('\n' + '='.repeat(50));
                                        console.log('✅ All SQLite3 tests passed successfully!');
                                        console.log('SQLite3 is working properly on Windows.');
                                        console.log('='.repeat(50));
                                    });
                                });
                            });
                        });
                    });
                });
            });
        });
    });
});