#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "========================================="
echo "停止索道塔架监测系统"
echo "========================================="

cd "$PROJECT_DIR"

echo "停止所有服务..."
docker-compose down

echo "服务已停止"
echo ""
echo "如需清理数据，请手动执行："
echo "  docker-compose down -v"
echo "  注意：这将删除所有数据库数据！"
