#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Starting Rust Backend Integration Tests${NC}\n"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker is not running. Please start Docker first.${NC}"
    exit 1
fi

# Load test environment variables
if [ -f .env.test ]; then
    export $(cat .env.test | xargs)
else
    echo -e "${YELLOW}⚠️  .env.test not found, using default values${NC}"
    export TEST_DATABASE_URL="postgresql://postgres:postgres@localhost:5434/test_postgres"
    export TEST_REDIS_URL="redis://localhost:6380"
fi

echo -e "${YELLOW}🐳 Starting test dependencies...${NC}"

# Start the dataset manager
project_dir=$(cargo locate-project | jq -r '.root')
top_dir=$(dirname $(dirname $project_dir))
# HACK: getting the debug binary, but i guess i should use the release one when it exists
dataset_manager="$top_dir/dataset-manager/target/debug/dataset-manager"
eval "$dataset_manager $(dirname $project_dir)/tests/test_datasets &"
dataset_manager_pid=$!

# Start test database and Redis
docker compose up -d db_test redis_test

# Wait for databases to be ready
echo -e "${YELLOW}⏳ Waiting for databases to be ready...${NC}"
sleep 5

# Check if test database is accessible
if ! pg_isready -h localhost -p 5434 -U user > /dev/null 2>&1; then
    echo -e "${RED}❌ Test database is not ready. Waiting a bit more...${NC}"
    sleep 10
    if ! pg_isready -h localhost -p 5434 -U user > /dev/null 2>&1; then
        echo -e "${RED}❌ Test database failed to start${NC}"
        exit 1
    fi
fi

# Check Redis
if ! redis-cli -p 6380 ping > /dev/null 2>&1; then
    echo -e "${RED}❌ Test Redis is not ready${NC}"
    exit 1
fi

DATABASE_URL=$TEST_DATABASE_URL $HOME/.cargo/bin/sqlx database create

echo -e "${GREEN}✅ Dependencies are ready!${NC}\n"

# Run different test suites
echo -e "${BLUE}🏃 Running Integration Tests...${NC}"

# Run all integration tests
if cargo test --test integration_tests -- --test-threads=1 --nocapture; then
    echo -e "\n${GREEN}✅ All integration tests passed!${NC}\n"
    TEST_SUCCESS=true
else
    echo -e "\n${RED}❌ Some integration tests failed${NC}\n"
    TEST_SUCCESS=false
fi

# Run unit tests
echo -e "${BLUE}🏃 Running Unit Tests...${NC}"
if cargo test --lib; then
    echo -e "\n${GREEN}✅ All unit tests passed!${NC}\n"
    UNIT_SUCCESS=true
else
    echo -e "\n${RED}❌ Some unit tests failed${NC}\n"
    UNIT_SUCCESS=false
fi

# Cleanup
echo -e "${YELLOW}🧹 Cleaning up test environment...${NC}"
docker compose stop db_test redis_test
docker compose rm -f db_test redis_test

trap "kill $dataset_manager_pid 2>/dev/null" EXIT

# Final result
echo -e "${BLUE}==================== FINAL RESULTS ====================${NC}"

if [ "$TEST_SUCCESS" = true ] && [ "$UNIT_SUCCESS" = true ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED! 🚀${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Please check the output above.${NC}"
    if [ "$TEST_SUCCESS" = false ]; then
        echo -e "${RED}  - Integration tests failed${NC}"
    fi
    if [ "$UNIT_SUCCESS" = false ]; then
        echo -e "${RED}  - Unit tests failed${NC}"
    fi
    exit 1
fi
