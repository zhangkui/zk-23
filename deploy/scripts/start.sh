#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "========================================="
echo "启动索道塔架监测系统"
echo "========================================="

cd "$PROJECT_DIR"

# 检查 .env 文件
if [ ! -f .env ]; then
    echo "警告: .env 文件不存在，使用 .env.example 作为默认配置"
    cp .env.example .env
fi

# 加载环境变量
export $(grep -v '^#' .env | xargs)

# 创建必要的目录
mkdir -p deploy/nginx/ssl
mkdir -p deploy/clickhouse/data
mkdir -p deploy/nats/data
mkdir -p deploy/redis/data
mkdir -p logs

# 生成自签名证书（如果没有证书）
if [ ! -f deploy/nginx/ssl/fullchain.pem ] || [ ! -f deploy/nginx/ssl/privkey.pem ]; then
    echo "未找到 SSL 证书，正在生成自签名证书..."
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
        -keyout deploy/nginx/ssl/privkey.pem \
        -out deploy/nginx/ssl/fullchain.pem \
        -subj "/C=CN/ST=Sichuan/L=Chengdu/O=Cableway Monitor/OU=IT/CN=${DOMAIN_NAME:-localhost}"
    cp deploy/nginx/ssl/fullchain.pem deploy/nginx/ssl/chain.pem
    echo "自签名证书已生成，建议在生产环境使用正式证书"
fi

# 设置目录权限
chown -R 101:101 deploy/nginx/ssl 2>/dev/null || true
chmod -R 755 deploy

# 停止现有服务
echo "停止现有服务..."
docker-compose down || true

# 清理旧的镜像和容器（可选）
# docker system prune -f

# 构建并启动服务
echo "构建并启动服务..."
docker-compose up -d --build

# 等待服务启动
echo "等待服务启动..."
sleep 10

# 检查服务状态
echo "检查服务状态..."
docker-compose ps

echo ""
echo "========================================="
echo "服务启动完成！"
echo "========================================="
echo ""
echo "访问地址："
echo "  - 前端: https://${DOMAIN_NAME:-localhost}"
echo "  - 后端 API: https://${DOMAIN_NAME:-localhost}/api"
echo "  - WebSocket: wss://${DOMAIN_NAME:-localhost}/ws"
echo ""
echo "默认账号："
echo "  - 用户名: admin"
echo "  - 密码: admin123"
echo ""
echo "查看日志: docker-compose logs -f"
echo "停止服务: ./deploy/scripts/stop.sh"
echo "重启服务: ./deploy/scripts/restart.sh"
echo ""
