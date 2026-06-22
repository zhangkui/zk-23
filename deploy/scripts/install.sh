#!/bin/bash
set -e

echo "========================================="
echo "山地索道塔架监测系统 - 边缘服务器部署脚本"
echo "========================================="

# 检查是否为 root 用户
if [ "$EUID" -ne 0 ]; then 
    echo "请使用 root 用户运行此脚本"
    exit 1
fi

# 检查操作系统
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$NAME
    VER=$VERSION_ID
    echo "检测到操作系统: $OS $VER"
else
    echo "无法检测操作系统"
    exit 1
fi

# 更新系统
echo "正在更新系统..."
if [[ "$OS" == *"Ubuntu"* ]] || [[ "$OS" == *"Debian"* ]]; then
    apt-get update -y
    apt-get upgrade -y
    apt-get install -y ca-certificates curl gnupg lsb-release
elif [[ "$OS" == *"CentOS"* ]] || [[ "$OS" == *"Rocky"* ]] || [[ "$OS" == *"AlmaLinux"* ]]; then
    yum update -y
    yum install -y yum-utils ca-certificates curl
fi

# 安装 Docker
echo "正在安装 Docker..."
if [[ "$OS" == *"Ubuntu"* ]] || [[ "$OS" == *"Debian"* ]]; then
    install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
    chmod a+r /etc/apt/keyrings/docker.gpg
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
    apt-get update -y
    apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
elif [[ "$OS" == *"CentOS"* ]] || [[ "$OS" == *"Rocky"* ]] || [[ "$OS" == *"AlmaLinux"* ]]; then
    yum-config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo
    yum install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
fi

# 启动 Docker
systemctl enable docker
systemctl start docker

# 安装 Docker Compose
echo "正在安装 Docker Compose..."
curl -L "https://github.com/docker/compose/releases/download/v2.24.1/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
chmod +x /usr/local/bin/docker-compose

# 验证安装
echo "验证安装..."
docker --version
docker-compose --version

# 配置系统参数
echo "正在配置系统参数..."

# 增加文件描述符限制
cat >> /etc/security/limits.conf << EOF
* soft nofile 65535
* hard nofile 65535
root soft nofile 65535
root hard nofile 65535
EOF

# 增加最大连接数
cat >> /etc/sysctl.conf << EOF
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.tcp_syncookies = 1
net.ipv4.tcp_fin_timeout = 30
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_keepalive_intvl = 30
net.ipv4.tcp_keepalive_probes = 3
vm.swappiness = 0
vm.max_map_count = 262144
EOF

sysctl -p

# 创建应用目录
echo "正在创建应用目录..."
APP_DIR="/opt/cableway-monitor"
mkdir -p $APP_DIR
cd $APP_DIR

# 设置时区
timedatectl set-timezone Asia/Shanghai

echo ""
echo "========================================="
echo "系统环境配置完成！"
echo "========================================="
echo ""
echo "接下来请执行以下步骤："
echo "1. 将项目文件上传到 $APP_DIR"
echo "2. 复制 .env.example 为 .env 并修改密码"
echo "3. 运行 deploy/scripts/start.sh 启动服务"
echo ""
