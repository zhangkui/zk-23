#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
BACKUP_DIR="/var/backups/cableway-monitor"
DATE=$(date +%Y%m%d_%H%M%S)

echo "========================================="
echo "备份索道塔架监测系统数据"
echo "========================================="

cd "$PROJECT_DIR"

# 创建备份目录
mkdir -p "$BACKUP_DIR"

# 加载环境变量
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

# 备份 ClickHouse 数据
echo "备份 ClickHouse 数据..."
docker exec cableway-clickhouse clickhouse-client \
    --user default \
    --password "${CLICKHOUSE_PASSWORD:-clickhouse_pass}" \
    --query "BACKUP DATABASE cableway_monitor TO Disk('backups', 'cableway_backup_$DATE')"

# 备份 Docker volumes
echo "备份 Docker volumes..."
docker run --rm \
    -v clickhouse_data:/data/clickhouse \
    -v nats_data:/data/nats \
    -v redis_data:/data/redis \
    -v "$BACKUP_DIR:/backup" \
    alpine tar czf /backup/volumes_$DATE.tar.gz -C /data .

# 备份配置文件
echo "备份配置文件..."
tar czf "$BACKUP_DIR/config_$DATE.tar.gz" \
    .env \
    docker-compose.yml \
    deploy/

# 清理旧备份（保留7天）
find "$BACKUP_DIR" -type f -name "*.tar.gz" -mtime +7 -delete
find "$BACKUP_DIR" -type f -name "*.sql" -mtime +7 -delete

echo ""
echo "========================================="
echo "备份完成！"
echo "========================================="
echo ""
echo "备份文件位置: $BACKUP_DIR"
echo "备份文件列表:"
ls -lh "$BACKUP_DIR"
echo ""
