#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "========================================="
echo "重启索道塔架监测系统"
echo "========================================="

cd "$PROJECT_DIR"

echo "重启服务..."
docker-compose restart

# 等待服务启动
echo "等待服务恢复..."
sleep 5

# 检查服务状态
echo "检查服务状态..."
docker-compose ps

echo "服务已重启"
