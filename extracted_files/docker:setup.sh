#!/bin/bash

echo "🚀 Setting up Enterprise Blockchain in Docker..."

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Navigate to docker directory
cd "$(dirname "$0")"

# Build and start the services
echo "🔨 Building Docker images..."
docker-compose build

echo "🚀 Starting enterprise blockchain services..."
docker-compose up -d

echo "⏳ Waiting for services to start..."
sleep 15

# Check service health
echo "🔍 Checking service health..."

for i in {1..3}; do
    echo "Checking validator$i..."
    if curl -s http://localhost:808$i/health > /dev/null; then
        echo "✅ Validator $i is healthy"
    else
        echo "❌ Validator $i is not responding"
    fi
done

echo "Checking load balancer..."
if curl -s http://localhost:8080/health > /dev/null; then
    echo "✅ Load balancer is healthy"
else
    echo "❌ Load balancer is not responding"
fi

echo ""
echo "🎉 Enterprise Blockchain is ready!"
echo ""
echo "🔗 API Endpoint: http://localhost:8080"
echo "⚙️  Individual Validators:"
echo "   - Validator 1: http://localhost:8081"
echo "   - Validator 2: http://localhost:8082" 
echo "   - Validator 3: http://localhost:8083"
echo ""
echo "📊 To start the dashboard separately:"
echo "   cargo run --bin enterprise-dashboard -- --port 9090"
echo "   Then visit: http://localhost:9090"
echo ""
echo "📋 Useful commands:"
echo "   docker-compose logs -f          # View logs"
echo "   docker-compose stop             # Stop services"
echo "   docker-compose down             # Stop and remove containers"
echo "   docker-compose ps               # View running services"
echo ""
echo "🧪 Test the API:"
echo "   curl http://localhost:8080/api/status"
echo "   curl http://localhost:8080/health"
echo ""
echo "🔗 Now start your tracker with:"
echo "   ENTERPRISE_BC_URL=\"http://192.168.200.133:8080\" cargo run --bin tracker"
