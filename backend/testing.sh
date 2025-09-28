#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function to check response
check_response() {
    local test_name="$1"
    local response="$2"
    local expected_status="$3"
    local expected_content="$4"
    
    echo -e "\n${YELLOW}=== $test_name ===${NC}"
    echo "Response: $response"
    
    # Check if response contains expected content
    if echo "$response" | grep -q "$expected_content"; then
        echo -e "${GREEN}✓ PASS: $test_name${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAIL: $test_name${NC}"
        echo -e "${RED}Expected to find: '$expected_content'${NC}"
        ((TESTS_FAILED++))
    fi
}

# Helper function to check HTTP status
check_status() {
    local test_name="$1"
    local status="$2"
    local expected_status="$3"
    
    if [ "$status" = "$expected_status" ]; then
        echo -e "${GREEN}✓ Status $status (expected $expected_status)${NC}"
    else
        echo -e "${RED}✗ Status $status (expected $expected_status)${NC}"
        ((TESTS_FAILED++))
    fi
}

echo -e "${YELLOW}Starting API Test Suite...${NC}\n"

# Test 1: Registration
echo -e "${YELLOW}=== Test 1: User Registration ===${NC}"
REGISTER_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST http://localhost:3000/api/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "cheezy",
    "email": "test@example.com",
    "password": "testpass123"
  }')

STATUS=$(echo "$REGISTER_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$REGISTER_RESPONSE" | head -n -1)

echo "Status: $STATUS"
echo "Response: $RESPONSE_BODY" | jq 2>/dev/null || echo "Response: $RESPONSE_BODY"

check_status "Registration Status" "$STATUS" "200"
if echo "$RESPONSE_BODY" | jq -e '.message == "User registered successfully" and .user.email == "test@example.com"' >/dev/null 2>&1; then
    echo -e "${GREEN}✓ PASS: Registration response contains correct data${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ FAIL: Registration response validation${NC}"
    ((TESTS_FAILED++))
fi

# Test 2: Login
echo -e "\n${YELLOW}=== Test 2: User Login ===${NC}"
LOGIN_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST http://localhost:3000/api/login \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{
    "email": "test@example.com",
    "password": "testpass123"
  }')

STATUS=$(echo "$LOGIN_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$LOGIN_RESPONSE" | head -n -1)

echo "Status: $STATUS"
echo "Response: $RESPONSE_BODY" | jq 2>/dev/null || echo "Response: $RESPONSE_BODY"

check_status "Login Status" "$STATUS" "200"
if echo "$RESPONSE_BODY" | jq -e '.message == "Login successful" and .user.email == "test@example.com"' >/dev/null 2>&1; then
    echo -e "${GREEN}✓ PASS: Login response contains correct data${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ FAIL: Login response validation${NC}"
    ((TESTS_FAILED++))
fi

# Check if session cookie was set
if [ -f cookies.txt ] && grep -q "session_id" cookies.txt; then
    echo -e "${GREEN}✓ PASS: Session cookie was set${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ FAIL: Session cookie was not set${NC}"
    ((TESTS_FAILED++))
fi

# Test 3: Get Current User (Protected)
echo -e "\n${YELLOW}=== Test 3: Get Current User (Protected Route) ===${NC}"
ME_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET http://localhost:3000/api/me \
  -H "Content-Type: application/json" \
  -b cookies.txt)

STATUS=$(echo "$ME_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$ME_RESPONSE" | head -n -1)

echo "Status: $STATUS"
echo "Response: $RESPONSE_BODY" | jq 2>/dev/null || echo "Response: $RESPONSE_BODY"

check_status "Get Current User Status" "$STATUS" "200"
if echo "$RESPONSE_BODY" | jq -e '.email == "test@example.com" and .id != null' >/dev/null 2>&1; then
    echo -e "${GREEN}✓ PASS: Current user response contains correct data${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ FAIL: Current user response validation${NC}"
    ((TESTS_FAILED++))
fi

# Test 4: Protected Route
echo -e "\n${YELLOW}=== Test 4: Access Protected Route ===${NC}"
PROTECTED_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET http://localhost:3000/api/protected \
  -H "Content-Type: application/json" \
  -b cookies.txt)

STATUS=$(echo "$PROTECTED_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$PROTECTED_RESPONSE" | head -n -1)

echo "Status: $STATUS"
echo "Response: $RESPONSE_BODY" | jq 2>/dev/null || echo "Response: $RESPONSE_BODY"

check_status "Protected Route Status" "$STATUS" "200"
if echo "$RESPONSE_BODY" | jq -e '.message == "This is a protected route" and .user_id != null' >/dev/null 2>&1; then
    echo -e "${GREEN}✓ PASS: Protected route response contains correct data${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ FAIL: Protected route response validation${NC}"
    ((TESTS_FAILED++))
fi

# Test 5: Logout
echo -e "\n${YELLOW}=== Test 5: User Logout ===${NC}"
LOGOUT_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST http://localhost:3000/api/logout \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -c cookies.txt)

STATUS=$(echo "$LOGOUT_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$LOGOUT_RESPONSE" | head -n -1)

echo "Status: $STATUS"
echo "Response: $RESPONSE_BODY" | jq 2>/dev/null || echo "Response: $RESPONSE_BODY"

check_status "Logout Status" "$STATUS" "200"
if echo "$RESPONSE_BODY" | jq -e '.message == "Logout successful"' >/dev/null 2>&1; then
    echo -e "${GREEN}✓ PASS: Logout response contains correct message${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ FAIL: Logout response validation${NC}"
    ((TESTS_FAILED++))
fi

# Test 6: Access After Logout (Should Fail)
echo -e "\n${YELLOW}=== Test 6: Access After Logout (Should Fail) ===${NC}"
UNAUTHORIZED_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET http://localhost:3000/api/me \
  -H "Content-Type: application/json" \
  -b cookies.txt)

STATUS=$(echo "$UNAUTHORIZED_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$UNAUTHORIZED_RESPONSE" | head -n -1)

echo "Status: $STATUS"
echo "Response: $RESPONSE_BODY" | jq 2>/dev/null || echo "Response: $RESPONSE_BODY"

check_status "Unauthorized Access Status" "$STATUS" "401"
if echo "$RESPONSE_BODY" | jq -e '.error == "Unauthorized"' >/dev/null 2>&1; then
    echo -e "${GREEN}✓ PASS: Unauthorized response contains correct error${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ FAIL: Unauthorized response validation${NC}"
    ((TESTS_FAILED++))
fi

# Test 7: Duplicate Registration (Should Fail)
echo -e "\n${YELLOW}=== Test 7: Duplicate Registration (Should Fail) ===${NC}"
DUPLICATE_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST http://localhost:3000/api/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "cheezy",
    "email": "test@example.com",
    "password": "testpass123"
  }')

STATUS=$(echo "$DUPLICATE_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$DUPLICATE_RESPONSE" | head -n -1)

echo "Status: $STATUS"
echo "Response: $RESPONSE_BODY" | jq 2>/dev/null || echo "Response: $RESPONSE_BODY"

check_status "Duplicate Registration Status" "$STATUS" "409"
if echo "$RESPONSE_BODY" | grep -q "already exists"; then
    echo -e "${GREEN}✓ PASS: Duplicate registration properly rejected${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ FAIL: Duplicate registration validation${NC}"
    ((TESTS_FAILED++))
fi

# Cleanup
rm -f cookies.txt

# Final Results
echo -e "\n${YELLOW}==================== TEST RESULTS ====================${NC}"
echo -e "${GREEN}Tests Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Tests Failed: $TESTS_FAILED${NC}"
echo -e "${YELLOW}Total Tests: $((TESTS_PASSED + TESTS_FAILED))${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 ALL TESTS PASSED! Your API is working correctly.${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed. Please check your API implementation.${NC}"
    exit 1
fi
