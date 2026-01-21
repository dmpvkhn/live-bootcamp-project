#!/bin/bash

# Configuration
BASE_URL="http://localhost:3000"
COOKIES_FILE="cookies.txt"

echo "Base URL: $BASE_URL"
echo "=================================="

# 1. Get Login/Signup UI
echo "1. Testing GET / (Login/Signup UI)"
curl -X GET "$BASE_URL/"
echo -e "\n"

# 2. Sign Up (without 2FA)
echo "2. Testing POST /signup (without 2FA)"
curl -X POST "$BASE_URL/signup" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePassword123!",
    "requires2FA": false
  }'
echo -e "\n"

# 3. Sign Up (with 2FA enabled)
echo "3. Testing POST /signup (with 2FA)"
curl -X POST "$BASE_URL/signup" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user2fa@example.com",
    "password": "SecurePassword123!",
    "requires2FA": true
  }'
echo -e "\n"

# 4. Login (without 2FA)
echo "4. Testing POST /login (without 2FA)"
curl -X POST "$BASE_URL/login" \
  -H "Content-Type: application/json" \
  -c "$COOKIES_FILE" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePassword123!"
  }'
echo -e "\n"

# 5. Login (with 2FA - returns loginAttemptId)
echo "5. Testing POST /login (with 2FA - should return loginAttemptId)"
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user2fa@example.com",
    "password": "SecurePassword123!"
  }')
echo "$LOGIN_RESPONSE"
echo -e "\n"

# 6. Verify 2FA (manual step - requires actual code)
echo "6. Testing POST /verify-2fa"
echo "Note: Replace LOGIN_ATTEMPT_ID and 2FA_CODE with actual values"
echo "Example command:"
echo "curl -X POST \"$BASE_URL/verify-2fa\" \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -c \"$COOKIES_FILE\" \\"
echo "  -d '{"
echo "    \"email\": \"user2fa@example.com\","
echo "    \"loginAttemptId\": \"LOGIN_ATTEMPT_ID\","
echo "    \"2FACode\": \"123456\""
echo "  }'"
echo -e "\n"

# Uncomment and fill in values to test:
# curl -X POST "$BASE_URL/verify-2fa" \
#   -H "Content-Type: application/json" \
#   -c "$COOKIES_FILE" \
#   -d '{
#     "email": "user2fa@example.com",
#     "loginAttemptId": "REPLACE_WITH_ACTUAL_ID",
#     "2FACode": "REPLACE_WITH_ACTUAL_CODE"
#   }'
# echo -e "\n"

# 7. Verify Token
echo "7. Testing POST /verify-token"
echo "Note: Replace YOUR_JWT_TOKEN with actual token"
echo "Example command:"
echo "curl -X POST \"$BASE_URL/verify-token\" \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -d '{\"token\": \"YOUR_JWT_TOKEN\"}'"
echo -e "\n"

# Uncomment and fill in token to test:
# curl -X POST "$BASE_URL/verify-token" \
#   -H "Content-Type: application/json" \
#   -d '{
#     "token": "REPLACE_WITH_ACTUAL_TOKEN"
#   }'
# echo -e "\n"

# 8. Logout
echo "8. Testing POST /logout (requires valid JWT cookie)"
curl -X POST "$BASE_URL/logout" \
  -b "$COOKIES_FILE" \
  -c "$COOKIES_FILE"
echo -e "\n"

echo "=================================="
echo "Testing complete!"
echo "Cookies saved to: $COOKIES_FILE"
