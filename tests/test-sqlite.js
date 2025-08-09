const Database = require('better-sqlite3');
const sqlite3 = require('sqlite3').verbose();

console.log('Testing SQLite bindings on Windows...\n');

// Test better-sqlite3
try {
    const db = new Database(':memory:');
    db.exec('CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)');
    const insert = db.prepare('INSERT INTO test (name) VALUES (?)');
    insert.run('Windows Test');
    const row = db.prepare('SELECT * FROM test').get();
    console.log('✓ better-sqlite3 works:', row);
    db.close();
} catch (error) {
    console.error('✗ better-sqlite3 failed:', error.message);
}

// Test sqlite3
const db2 = new sqlite3.Database(':memory:', (err) => {
    if (err) {
        console.error('✗ sqlite3 failed to open:', err.message);
        return;
    }
    
    db2.serialize(() => {
        db2.run('CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)', (err) => {
            if (err) {
                console.error('✗ sqlite3 failed to create table:', err.message);
                return;
            }
            
            db2.run('INSERT INTO test (name) VALUES (?)', ['Windows Test'], (err) => {
                if (err) {
                    console.error('✗ sqlite3 failed to insert:', err.message);
                    return;
                }
                
                db2.get('SELECT * FROM test', (err, row) => {
                    if (err) {
                        console.error('✗ sqlite3 failed to select:', err.message);
                        return;
                    }
                    console.log('✓ sqlite3 works:', row);
                    db2.close();
                });
            });
        });
    });
});