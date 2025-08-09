# Run These Commands To Fix NPX Claude-Flow

## Commands to Run Right Now (Copy & Paste)

### Step 1: Build the Bindings
```batch
git clone https://github.com/ruvnet/claude-flow.git C:\code\claude-flow
cd C:\code\claude-flow
npm install
cd node_modules\better-sqlite3
npm run build-release
cd C:\code\claude-flow
```

### Step 2: Find Your NPX Cache
```batch
npx claude-flow@alpha --version
```

Now look for the cache directory:
```batch
dir %LOCALAPPDATA%\npm-cache\_npx /b /ad
```

You'll see something like `7cfa166e65244432` - that's your cache hash.

### Step 3: Copy Bindings to Cache (Replace YOUR_HASH with actual hash)
```batch
set HASH=YOUR_HASH
set CACHE=%LOCALAPPDATA%\npm-cache\_npx\%HASH%
set BINDING=C:\code\claude-flow\node_modules\better-sqlite3\build\Release\better_sqlite3.node

mkdir "%CACHE%\node_modules\better-sqlite3\build\Release"
copy "%BINDING%" "%CACHE%\node_modules\better-sqlite3\build\Release\"

mkdir "%CACHE%\node_modules\better-sqlite3\lib\binding\node-v127-win32-x64"
copy "%BINDING%" "%CACHE%\node_modules\better-sqlite3\lib\binding\node-v127-win32-x64\"

mkdir "%CACHE%\node_modules\better-sqlite3\compiled\22.15.0\win32\x64"
copy "%BINDING%" "%CACHE%\node_modules\better-sqlite3\compiled\22.15.0\win32\x64\"
```

### Step 4: Fix ruv-swarm's better-sqlite3 too
```batch
mkdir "%CACHE%\node_modules\ruv-swarm\node_modules\better-sqlite3\build\Release"
copy "%BINDING%" "%CACHE%\node_modules\ruv-swarm\node_modules\better-sqlite3\build\Release\"
```

### Step 5: Test It Works
```batch
cd C:\temp
npx claude-flow@alpha init
```

## One-Command Fix (After Building)

If you've already built the bindings in Step 1, here's a one-liner that finds and fixes all caches:

```batch
for /d %d in (%LOCALAPPDATA%\npm-cache\_npx\*) do @if exist "%d\node_modules\better-sqlite3" (mkdir "%d\node_modules\better-sqlite3\build\Release" 2>nul & copy "C:\code\claude-flow\node_modules\better-sqlite3\build\Release\better_sqlite3.node" "%d\node_modules\better-sqlite3\build\Release\" >nul & echo Fixed: %~nd)
```

## Automated Script

Save this as `fix-npx.bat` and run it:

```batch
@echo off
echo Fixing NPX Claude-Flow...

:: Build if needed
if not exist "C:\code\claude-flow\node_modules\better-sqlite3\build\Release\better_sqlite3.node" (
    echo Building bindings...
    cd C:\code\claude-flow
    call npm install
    cd node_modules\better-sqlite3
    call npm run build-release
    cd C:\code\claude-flow
)

:: Find and fix all caches
for /d %%d in (%LOCALAPPDATA%\npm-cache\_npx\*) do (
    if exist "%%d\node_modules\better-sqlite3" (
        echo Fixing cache: %%~nd
        
        mkdir "%%d\node_modules\better-sqlite3\build\Release" 2>nul
        copy "C:\code\claude-flow\node_modules\better-sqlite3\build\Release\better_sqlite3.node" ^
             "%%d\node_modules\better-sqlite3\build\Release\" >nul
        
        mkdir "%%d\node_modules\better-sqlite3\lib\binding\node-v127-win32-x64" 2>nul
        copy "C:\code\claude-flow\node_modules\better-sqlite3\build\Release\better_sqlite3.node" ^
             "%%d\node_modules\better-sqlite3\lib\binding\node-v127-win32-x64\" >nul
    )
    
    if exist "%%d\node_modules\ruv-swarm\node_modules\better-sqlite3" (
        mkdir "%%d\node_modules\ruv-swarm\node_modules\better-sqlite3\build\Release" 2>nul
        copy "C:\code\claude-flow\node_modules\better-sqlite3\build\Release\better_sqlite3.node" ^
             "%%d\node_modules\ruv-swarm\node_modules\better-sqlite3\build\Release\" >nul
    )
)

echo Done! Testing...
cd %TEMP%
npx claude-flow@alpha --version
pause
```

## Quick Verification

Run these to verify the fix worked:

```batch
# Should work without errors
npx claude-flow@alpha --version

# Should store and retrieve data
npx claude-flow@alpha memory store test "Hello"
npx claude-flow@alpha memory get test

# Should initialize a project
cd C:\temp\test
npx claude-flow@alpha init
```

## If It Still Doesn't Work

1. Check Node version matches:
```batch
node -v
# Should be v22.x.x
```

2. Verify binding was built:
```batch
dir C:\code\claude-flow\node_modules\better-sqlite3\build\Release\better_sqlite3.node
# Should show a ~1.8MB file
```

3. Clear cache and try again:
```batch
npm cache clean --force
# Then repeat from Step 2
```