const Database = require('better-sqlite3');
const path = require('path');

console.log('Testing SQLite on Windows...\n');

try {
    // Test 1: Create an in-memory database
    console.log('Test 1: Creating in-memory database...');
    const memDb = new Database(':memory:');
    console.log('✓ In-memory database created successfully');
    
    // Test 2: Create a table
    console.log('\nTest 2: Creating table...');
    memDb.exec(`
        CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT UNIQUE
        )
    `);
    console.log('✓ Table created successfully');
    
    // Test 3: Insert data
    console.log('\nTest 3: Inserting data...');
    const insert = memDb.prepare('INSERT INTO users (name, email) VALUES (?, ?)');
    insert.run('John Doe', 'john@example.com');
    insert.run('Jane Smith', 'jane@example.com');
    console.log('✓ Data inserted successfully');
    
    // Test 4: Query data
    console.log('\nTest 4: Querying data...');
    const rows = memDb.prepare('SELECT * FROM users').all();
    console.log('✓ Query executed successfully');
    console.log('Results:', rows);
    
    memDb.close();
    
    // Test 5: Create a file-based database
    console.log('\nTest 5: Creating file-based database...');
    const dbPath = path.join(__dirname, 'test.db');
    const fileDb = new Database(dbPath);
    console.log(`✓ File database created at: ${dbPath}`);
    
    // Test 6: Operations on file database
    console.log('\nTest 6: Testing file database operations...');
    fileDb.exec(`
        CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY,
            name TEXT,
            price REAL
        )
    `);
    
    const productInsert = fileDb.prepare('INSERT OR REPLACE INTO products VALUES (?, ?, ?)');
    productInsert.run(1, 'Laptop', 999.99);
    productInsert.run(2, 'Mouse', 29.99);
    
    const products = fileDb.prepare('SELECT * FROM products').all();
    console.log('✓ File database operations successful');
    console.log('Products:', products);
    
    // Test 7: Transaction test
    console.log('\nTest 7: Testing transactions...');
    const transaction = fileDb.transaction((items) => {
        for (const item of items) {
            productInsert.run(item.id, item.name, item.price);
        }
    });
    
    transaction([
        { id: 3, name: 'Keyboard', price: 79.99 },
        { id: 4, name: 'Monitor', price: 299.99 }
    ]);
    console.log('✓ Transaction completed successfully');
    
    // Test 8: Prepared statements
    console.log('\nTest 8: Testing prepared statements...');
    const stmt = fileDb.prepare('SELECT * FROM products WHERE price < ?');
    const affordableProducts = stmt.all(100);
    console.log('✓ Prepared statement executed');
    console.log('Affordable products:', affordableProducts);
    
    fileDb.close();
    
    console.log('\n' + '='.repeat(50));
    console.log('✅ All SQLite tests passed successfully!');
    console.log('SQLite is working properly on Windows.');
    console.log('='.repeat(50));
    
} catch (error) {
    console.error('\n❌ SQLite test failed:');
    console.error('Error:', error.message);
    console.error('\nFull error details:');
    console.error(error);
    
    console.log('\n' + '='.repeat(50));
    console.log('TROUBLESHOOTING STEPS:');
    console.log('1. Install Visual Studio Build Tools:');
    console.log('   npm install --global windows-build-tools');
    console.log('   OR');
    console.log('   Download from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022');
    console.log('\n2. Install better-sqlite3 with rebuild:');
    console.log('   npm rebuild better-sqlite3');
    console.log('   OR');
    console.log('   npm install better-sqlite3 --build-from-source');
    console.log('\n3. Try using prebuilt binaries:');
    console.log('   npm install better-sqlite3@latest');
    console.log('='.repeat(50));
    
    process.exit(1);
}