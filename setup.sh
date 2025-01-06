# JWT secret
openssl rand -hex 32 > jwt.hex

# # Genesis
# cat > genesis.json << EOL

# EOL

mkdir data
docker-compose up
