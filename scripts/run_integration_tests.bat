@echo off
echo Running Integration Tests with Full System Features
echo ====================================================

echo.
echo STEP 1: Testing with test-integration features
echo ----------------------------------------------
cargo test integration_test --features test-integration -- --nocapture

echo.
echo STEP 2: Testing with full-system features (if above fails)
echo ----------------------------------------------------------
cargo test integration_test --features full-system -- --nocapture

echo.
echo STEP 3: List all integration tests (to verify discovery)
echo --------------------------------------------------------
cargo test integration_test --features full-system -- --list

echo.
echo STEP 4: Run all integration test files
echo --------------------------------------
cargo test bm25_integration_tests --features full-system -- --nocapture
cargo test chunker_integration_tests --features full-system -- --nocapture  
cargo test config_integration_verification --features full-system -- --nocapture
cargo test working_integration_test --features full-system -- --nocapture

echo.
echo Integration test execution completed!
pause