#!/bin/bash
set -e

KEYCLOAK_URL="http://localhost:8181"
REALM="Navigator"
ADMIN_USER="admin"
ADMIN_PASSWORD="admin"

echo "Fetching admin token..."
TOKEN=$(curl -s -X POST "$KEYCLOAK_URL/realms/master/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=admin-cli" \
  -d "username=$ADMIN_USER" \
  -d "password=$ADMIN_PASSWORD" \
  -d "grant_type=password" \
  | grep -o '"access_token":"[^"]*' \
  | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
  echo "Failed to obtain admin token. Is Keycloak running on $KEYCLOAK_URL?"
  exit 1
fi

echo "Creating user john.doe..."
HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$KEYCLOAK_URL/admin/realms/$REALM/users" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john.doe",
    "firstName": "John",
    "lastName": "Doe",
    "email": "john.doe@navigator.com",
    "enabled": true,
    "emailVerified": true,
    "credentials": [
      {
        "type": "password",
        "value": "password",
        "temporary": false
      }
    ]
  }')

if [ "$HTTP_STATUS" = "201" ]; then
  echo "User john.doe created successfully."
elif [ "$HTTP_STATUS" = "409" ]; then
  echo "User john.doe already exists."
else
  echo "Failed to create user. HTTP status: $HTTP_STATUS"
  exit 1
fi
