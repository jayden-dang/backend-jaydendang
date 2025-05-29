#!/bin/bash

# deploy.sh - Deploy script using Docker Compose

set -e

# Parse arguments
export DOCKER_IMAGE="$1"
export CIRCLE_SHA1="$2"
export DOCKER_USERNAME="$3"
export DOCKER_PASSWORD="$4"

# Validate arguments
if [ -z "$DOCKER_IMAGE" ] || [ -z "$CIRCLE_SHA1" ] || [ -z "$DOCKER_USERNAME" ] || [ -z "$DOCKER_PASSWORD" ]; then
    echo "Usage: $0 <docker_image> <image_tag> <docker_username> <docker_password>"
    exit 1
fi

echo "=== Starting deployment ==="
echo "Image: $DOCKER_IMAGE:$CIRCLE_SHA1"

# Login to Docker Hub
echo "Logging in to Docker Hub..."
echo "$DOCKER_PASSWORD" | docker login -u "$DOCKER_USERNAME" --password-stdin

# Stop existing containers
echo "Stopping existing containers..."
docker-compose -f docker-compose.postgres.yml down || true
docker stop jaydenblog || true
docker rm jaydenblog || true

# Start databases with Docker Compose
echo "Starting databases..."
docker-compose -f docker-compose.postgres.yml up -d

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL to be ready..."
for i in {1..30}; do
    if docker exec jd-postgres pg_isready -U jayden -d jaydenblog &>/dev/null; then
        echo "PostgreSQL is ready!"
        break
    fi
    echo "Waiting for PostgreSQL... ($i/30)"
    sleep 2
done

# Check if PostgreSQL connection works
echo "Testing database connection..."
if ! docker exec jd-postgres psql -U jayden -d jaydenblog -c "SELECT 1;" &>/dev/null; then
    echo "❌ Database connection failed!"
    echo "PostgreSQL logs:"
    docker logs jd-postgres --tail 20
    exit 1
fi

# Run database migrations if SQL files exist
if [ -d "sql" ]; then
    echo "Running database migrations..."
    for sql_file in sql/*.sql; do
        if [ -f "$sql_file" ]; then
            echo "Running $(basename "$sql_file")..."
            docker exec -i jd-postgres psql -U jayden -d jaydenblog < "$sql_file" || {
                echo "⚠️  Warning: Failed to run $sql_file"
            }
        fi
    done
else
    echo "No SQL migration files found, skipping migrations."
fi

# Pull and run application
echo "Deploying application..."
docker pull $DOCKER_IMAGE:$CIRCLE_SHA1

docker run -d \
    --name jaydenblog \
    --restart unless-stopped \
    -p 8080:8080 \
    --network host \
    -e DATABASE_URL=postgresql://jayden:postgres@localhost:5432/jaydenblog \
    -e REDIS_URL=redis://localhost:6379 \
    -e RUST_LOG=info \
    -e RUST_BACKTRACE=1 \
    $DOCKER_IMAGE:$CIRCLE_SHA1

# Wait for app to start
echo "Waiting for application to start..."
sleep 15

# Show deployment status
echo "=== Deployment Status ==="
echo "Containers:"
docker ps -a | grep -E "(jd-postgres|jd-redis|jaydenblog)"

echo -e "\nApplication logs (last 20 lines):"
docker logs jaydenblog --tail 20

# Health check
echo -e "\nHealth check:"
for i in {1..10}; do
    if curl -f -s http://localhost:8080/health &>/dev/null; then
        echo "✅ Application is healthy!"
        echo "🚀 Deployment completed successfully!"
        exit 0
    fi
    echo "Waiting for health check... ($i/10)"
    sleep 3
done

echo "❌ Application health check failed after 30 seconds"
echo "Application logs (last 50 lines):"
docker logs jaydenblog --tail 50

echo "Database logs:"
docker logs jd-postgres --tail 20

exit 1
