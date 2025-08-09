# The Quickest Fix for NPX Claude-Flow on Windows

## The Problem
`npx claude-flow@alpha init` fails with SQLite binding errors.

## The Solution
Pre-compile the bindings and put them where npx expects them.

## Quick Fix (3 Steps)

### 1. Clone and Build (One Time)
```batch
git clone https://github.com/ruvnet/claude-flow.git C:\code\claude-flow
cd C:\code\claude-flow
npm install
cd node_modules\better-sqlite3
npm run build-release
```

### 2. Find NPX Cache
```batch
# Run npx once to create cache
npx claude-flow@alpha --version

# Find the cache hash (16 character hex string)
dir %LOCALAPPDATA%\npm-cache\_npx /b
```

You'll see something like: `7cfa166e65244432`

### 3. Copy Binding to Cache
```batch
# Replace {HASH} with your actual hash from step 2
set HASH=7cfa166e65244432
set CACHE=%LOCALAPPDATA%\npm-cache\_npx\%HASH%

# Create directories and copy binding
mkdir "%CACHE%\node_modules\better-sqlite3\build\Release"
copy "C:\code\claude-flow\node_modules\better-sqlite3\build\Release\better_sqlite3.node" ^
     "%CACHE%\node_modules\better-sqlite3\build\Release\"
```

## Done! 
Now `npx claude-flow@alpha init` works from anywhere!

## One-Liner After Building
If you've already built the bindings in C:\code\claude-flow:

```batch
for /f %i in ('dir %LOCALAPPDATA%\npm-cache\_npx /b /ad-h /od ^| findstr /r "^[a-f0-9]*$"') do @(mkdir "%LOCALAPPDATA%\npm-cache\_npx\%i\node_modules\better-sqlite3\build\Release" 2>nul & copy "C:\code\claude-flow\node_modules\better-sqlite3\build\Release\better_sqlite3.node" "%LOCALAPPDATA%\npm-cache\_npx\%i\node_modules\better-sqlite3\build\Release\" >nul & echo Fixed cache: %i)
```

## Why This Works
- NPX downloads packages to a cache directory
- It expects to find `better_sqlite3.node` but can't build it
- We build it separately and place it there
- NPX finds it and everything works!

## When to Re-run
- After `npm cache clean`
- After Node.js version update
- If npx starts failing again

## Alternative: Use the Script
Run `scripts\populate-npx-cache.bat` from this repo - it does everything automatically.