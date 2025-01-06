#!/bin/bash

SECRET=$(cat jwt.hex)
HEADER=$(echo -n '{"alg":"HS256","typ":"JWT"}' | base64 | tr -d '=')
PAYLOAD=$(echo -n "{\"iat\":$(date +%s)}" | base64 | tr -d '=')
UNSIGNED="$HEADER.$PAYLOAD"
SIGNATURE=$(echo -n "$UNSIGNED" | openssl dgst -binary -sha256 -mac HMAC -macopt hexkey:$SECRET | base64 -w 0 | tr -d '=' | tr '+/' '-_')
JWT="$HEADER.$PAYLOAD.$SIGNATURE"

while IFS= read -r request; do
    echo "Sending request:"
    echo "$request"
    curl -w "Time: %{time_total}s\n" -X POST http://localhost:8551 \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $JWT" \
    --data "$request"
    echo -e "\n---\n"
done < requests.txt