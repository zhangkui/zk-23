#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "========================================="
echo "系统健康检查"
echo "========================================="

cd "$PROJECT_DIR"

# 检查 Docker 服务
echo "1. 检查 Docker 服务..."
if systemctl is-active --quiet docker; then
    echo "   ✓ Docker 服务运行正常"
else
    echo "   ✗ Docker 服务未运行"
    exit 1
fi

# 检查容器状态
echo ""
echo "2. 检查容器状态..."
CONTAINERS=("cableway-clickhouse" "cableway-nats" "cableway-redis" "cableway-backend" "cableway-frontend" "cableway-nginx")
ALL_RUNNING=true

for container in "${CONTAINERS[@]}"; do
    if [ "$(docker inspect -f '{{.State.Running}}' "$container" 2>/dev/null)" = "true" ]; then
        HEALTH=$(docker inspect -f '{{.State.Health.Status}}' "$container" 2>/dev/null || echo "unknown")
        echo "   ✓ $container 运行正常 (健康状态: $HEALTH)"
    else
        echo "   ✗ $container 未运行"
        ALL_RUNNING=false
    fi
done

if [ "$ALL_RUNNING" = false ]; then
    exit 1
fi

# 检查端口监听
echo ""
echo "3. 检查端口监听..."
PORTS=("80" "443" "8080" "3000" "8123" "4222" "6379")
for port in "${PORTS[@]}"; do
    if netstat -tlnp | grep -q ":$port "; then
        SERVICE_NAME=$(netstat -tlnp | grep ":$port " | awk '{print $NF}' | cut -d'/' -f2)
        echo "   ✓ 端口 $port 正常监听 ($SERVICE_NAME)"
    else
        echo "   ✗ 端口 $port 未监听"
    fi
done

# 检查 API 健康检查
echo ""
echo "4. 检查 API 健康状态..."
if curl -fsS http://localhost:8080/api/health > /dev/null 2>&1; then
    echo "   ✓ 后端 API 正常"
else
    echo "   ✗ 后端 API 异常"
fi

if curl -fsS http://localhost:3000/ > /dev/null 2>&1; then
    echo "   ✓ 前端服务正常"
else
    echo "   ✗ 前端服务异常"
fi

# 检查磁盘空间
echo ""
echo "5. 检查磁盘空间..."
DISK_USAGE=$(df -h / | awk 'NR==2 {print $5}' | sed 's/%//')
echo "   磁盘使用率: $DISK_USAGE%"
if [ "$DISK_USAGE" -gt 80 ]; then
    echo "   ⚠ 磁盘使用率超过 80%，请及时清理"
else
    echo "   ✓ 磁盘空间充足"
fi

# 检查内存使用
echo ""
echo "6. 检查内存使用..."
MEMORY_USAGE=$(free | awk '/Mem:/ {printf "%.0f", $3/$2 * 100}')
echo "   内存使用率: $MEMORY_USAGE%"
if [ "$MEMORY_USAGE" -gt 80 ]; then
    echo "   ⚠ 内存使用率超过 80%"
else
    echo "   ✓ 内存使用正常"
fi

echo ""
echo "========================================="
echo "健康检查完成"
echo "========================================="
