# 山地索道塔架振动结冰联动监测与停运决策平台

## 项目概述

本平台是针对山地景区索道塔架设计的综合性智能监测系统，采用现代物联网、大数据分析和人工智能技术，实现对塔架结构健康状态的全方位监测和智能决策。

### 核心功能

1. **塔架点位建模** - 数字化塔架基础信息、地理位置、传感器配置
2. **振动与风速数据采集** - 实时采集振动振幅、频率、风速、风向等数据
3. **覆冰风险识别** - 基于多因素算法的覆冰厚度预测和风险等级评估
4. **视频联动复核** - AI 视频分析 + 人工复核的双重验证机制
5. **应急停运策略推送** - 基于风险评估的智能停运决策和行动方案
6. **巡检记录归档** - 完整的巡检流程管理和历史记录追溯
7. **恶劣天气影响分析** - 天气预报接入、灾害预警、影响评估

### 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 后端框架 | Rust Axum 0.7 | 高性能异步 Web 框架 |
| 前端框架 | Qwik 1.4 | 新一代高性能前端框架 |
| 时序数据库 | ClickHouse 24.1 | 海量传感器数据存储 |
| 消息队列 | NATS 2.10 | 高性能实时消息传输 |
| 实时通信 | WebSocket | 双向实时数据推送 |
| 缓存 | Redis 7.2 | 热点数据缓存和会话管理 |
| 图表库 | ECharts 5.4 | 数据可视化 |
| 地图组件 | Leaflet 1.9 | 地理信息展示 |
| 代理服务 | Nginx 1.25 | 反向代理和负载均衡 |
| 容器化 | Docker / Docker Compose | 应用部署和编排 |

---

## 系统架构

### 总体架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                        景区边缘服务器部署                            │
├─────────────┬───────────────────────────────────────────────────────┤
│             │  ┌─────────────────┐    ┌────────────────────────┐   │
│   浏览器    │  │   Nginx 代理    │    │   Qwik 前端 (SSR)      │   │
│             │  │  (SSL + 缓存)   │    │   (端口 3000)          │   │
│             │  └────────┬────────┘    └────────────────────────┘   │
└─────────────┘           │                                           │
                          │                                           │
┌─────────────────────────▼───────────────────────────────────────────┤
│                         后端服务层                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌────────────────────┐  ┌──────────────────────────────────────┐  │
│  │  Axum API 服务     │  │  WebSocket 实时推送                  │  │
│  │  (端口 8080)       │  │  (告警、状态、实时数据)              │  │
│  └─────────┬──────────┘  └──────────────────────────────────────┘  │
│            │                                                        │
│            ▼                                                        │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                       业务服务层                              │   │
│  ├───────────┬───────────┬───────────┬───────────┬──────────────┤   │
│  │ 塔架管理  │ 数据采集  │ 覆冰识别  │ 告警服务  │ 视频复核     │   │
│  ├───────────┼───────────┼───────────┼───────────┼──────────────┤   │
│  │ 停运策略  │ 巡检管理  │ 天气分析  │ 认证授权  │ 传感器管理   │   │
│  └───────────┴───────────┴───────────┴───────────┴──────────────┘   │
│            │                                                        │
└────────────┼────────────────────────────────────────────────────────┘
             │
┌────────────▼────────────────────────────────────────────────────────┐
│                       数据存储与消息层                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌────────────────┐  ┌───────────────┐  ┌──────────────────────┐   │
│  │  ClickHouse    │  │    NATS       │  │     Redis            │   │
│  │  (时序数据)    │  │  (消息队列)   │  │   (缓存/会话)        │   │
│  │  端口: 8123    │  │  端口: 4222   │  │   端口: 6379         │   │
│  └────────────────┘  └───────────────┘  └──────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 数据流

```
传感器 → NATS 消息队列 → 数据处理服务 → ClickHouse 存储
                          ↓
                    风险识别算法
                          ↓
                    告警触发 → 视频复核请求
                          ↓
                    停运策略评估 → 推送至前端
                          ↓
                    WebSocket 实时推送 → 用户界面
```

---

## 项目结构

```
zk-23/
├── backend/                    # Rust 后端
│   ├── src/
│   │   ├── config/            # 配置管理
│   │   ├── db/                # 数据库访问层
│   │   ├── handlers/          # API 请求处理器
│   │   ├── models/            # 数据模型
│   │   ├── mq/                # NATS 消息队列
│   │   ├── services/          # 业务服务
│   │   ├── websocket/         # WebSocket 服务
│   │   ├── lib.rs             # 库根文件
│   │   ├── main.rs            # 应用入口
│   │   └── routes.rs          # 路由配置
│   ├── Cargo.toml             # Rust 依赖
│   ├── Dockerfile             # 后端 Dockerfile
│   └── .env.example           # 环境变量示例
│
├── frontend/                   # Qwik 前端
│   ├── src/
│   │   ├── components/        # 通用组件
│   │   ├── routes/            # 页面路由
│   │   ├── services/          # API 和 WebSocket 服务
│   │   ├── stores/            # 状态管理
│   │   ├── types/             # TypeScript 类型定义
│   │   ├── global.css         # 全局样式 (Tailwind)
│   │   ├── root.tsx           # 根组件
│   │   ├── entry.ssr.tsx      # SSR 入口
│   │   └── entry.client.tsx   # 客户端入口
│   ├── package.json           # Node 依赖
│   ├── tailwind.config.js     # Tailwind 配置
│   ├── postcss.config.js      # PostCSS 配置
│   ├── vite.config.ts         # Vite 配置
│   ├── tsconfig.json          # TypeScript 配置
│   └── Dockerfile             # 前端 Dockerfile
│
├── deploy/                     # 部署配置
│   ├── clickhouse/            # ClickHouse 配置
│   ├── nats/                  # NATS 配置
│   ├── nginx/                 # Nginx 配置
│   │   ├── conf.d/
│   │   └── ssl/
│   ├── backend/               # 后端环境变量
│   └── scripts/               # 部署脚本
│       ├── install.sh         # 系统环境安装
│       ├── start.sh           # 启动服务
│       ├── stop.sh            # 停止服务
│       ├── restart.sh         # 重启服务
│       ├── backup.sh          # 数据备份
│       └── healthcheck.sh     # 健康检查
│
├── docker-compose.yml         # Docker Compose 编排
├── .env.example               # 环境变量示例
├── .gitignore                 # Git 忽略文件
└── README.md                  # 项目文档
```

---

## 快速开始

### 本地开发

#### 1. 环境要求

- **Rust**: 1.75+
- **Node.js**: 20+
- **Docker**: 24.0+
- **Docker Compose**: 2.20+

#### 2. 启动基础设施

```bash
# 启动 ClickHouse、NATS、Redis
docker-compose up -d clickhouse nats redis
```

#### 3. 配置环境变量

```bash
cp .env.example .env
# 修改 .env 中的密码和配置
```

#### 4. 启动后端服务

```bash
cd backend
cargo run -- --migrate --init-data
```

后端服务将在 `http://localhost:8080` 启动

#### 5. 启动前端服务

```bash
cd frontend
npm install
npm run dev
```

前端服务将在 `http://localhost:5173` 启动

#### 6. 访问系统

打开浏览器访问 `http://localhost:5173`

**默认账号**:
- 用户名: `admin`
- 密码: `admin123`

---

## 边缘服务器部署

### 系统要求

- **操作系统**: Ubuntu 20.04+ / CentOS 7+ / Rocky Linux 8+
- **CPU**: 4 核心及以上
- **内存**: 8GB 及以上（推荐 16GB）
- **磁盘**: 100GB 及以上 SSD（推荐 500GB）
- **网络**: 稳定的局域网连接，支持公网访问（可选）

### 自动化部署

#### 1. 上传项目文件

将项目文件上传到边缘服务器的 `/opt/cableway-monitor` 目录：

```bash
scp -r ./ zk-23 user@server:/opt/cableway-monitor
ssh user@server
cd /opt/cableway-monitor
```

#### 2. 执行系统环境安装

```bash
sudo su
chmod +x deploy/scripts/*.sh
./deploy/scripts/install.sh
```

该脚本将自动完成：
- 系统更新
- Docker 和 Docker Compose 安装
- 系统参数优化（文件描述符、网络参数等）
- 时区设置（Asia/Shanghai）
- 应用目录创建

#### 3. 配置环境变量

```bash
cp .env.example .env
vim .env
```

**必须修改的配置项**:

| 配置项 | 说明 | 示例 |
|--------|------|------|
| `CLICKHOUSE_PASSWORD` | ClickHouse 数据库密码 | 强随机字符串 |
| `NATS_PASSWORD` | NATS 消息队列密码 | 强随机字符串 |
| `REDIS_PASSWORD` | Redis 缓存密码 | 强随机字符串 |
| `JWT_SECRET` | JWT 签名密钥 | 至少 32 位随机字符串 |
| `DOMAIN_NAME` | 访问域名 | cableway.example.com |

#### 4. 启动服务

```bash
./deploy/scripts/start.sh
```

该脚本将自动完成：
- SSL 证书生成（自签名证书，生产环境请替换为正式证书）
- Docker 镜像构建
- 服务启动
- 健康检查

#### 5. 验证部署

运行健康检查脚本：

```bash
./deploy/scripts/healthcheck.sh
```

检查所有服务是否正常运行。

### 常用运维命令

```bash
# 启动服务
./deploy/scripts/start.sh

# 停止服务
./deploy/scripts/stop.sh

# 重启服务
./deploy/scripts/restart.sh

# 查看日志
docker-compose logs -f

# 查看特定服务日志
docker-compose logs -f backend
docker-compose logs -f frontend

# 数据备份
./deploy/scripts/backup.sh

# 健康检查
./deploy/scripts/healthcheck.sh

# 更新服务
git pull
./deploy/scripts/start.sh
```

### 生产环境部署建议

1. **SSL 证书**: 使用 Let's Encrypt 或购买正式 SSL 证书，替换 `deploy/nginx/ssl/` 中的自签名证书
2. **防火墙配置**: 仅开放必要端口（80, 443），关闭其他端口的公网访问
3. **定期备份**: 配置 cron 任务每天自动运行 `backup.sh`
4. **监控告警**: 接入 Prometheus + Grafana 进行系统监控
5. **日志管理**: 配置 ELK 或 Loki 进行日志集中管理
6. **高可用**: 对于重要景区，建议部署主备两台边缘服务器

---

## API 接口文档

### 认证接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/auth/login` | 用户登录 |
| POST | `/api/auth/refresh` | 刷新令牌 |
| POST | `/api/auth/change-password` | 修改密码 |
| GET | `/api/auth/me` | 获取当前用户信息 |

### 塔架管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/towers` | 获取塔架列表 |
| GET | `/api/towers/:id` | 获取塔架详情 |
| POST | `/api/towers` | 创建塔架 |
| PUT | `/api/towers/:id` | 更新塔架 |
| DELETE | `/api/towers/:id` | 删除塔架 |
| GET | `/api/towers/:id/status` | 获取塔架实时状态 |

### 传感器管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/sensors` | 获取传感器列表 |
| GET | `/api/sensors/:id` | 获取传感器详情 |
| POST | `/api/sensors` | 创建传感器 |
| PUT | `/api/sensors/:id` | 更新传感器 |
| DELETE | `/api/sensors/:id` | 删除传感器 |
| GET | `/api/sensors/tower/:towerId` | 获取塔架的传感器列表 |

### 数据采集

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/data/vibration/:towerId` | 获取振动历史数据 |
| GET | `/api/data/wind-speed/:towerId` | 获取风速历史数据 |
| GET | `/api/data/ice-detection/:towerId` | 获取覆冰历史数据 |
| POST | `/api/data/vibration` | 上报振动数据 |
| POST | `/api/data/wind-speed` | 上报风速数据 |
| POST | `/api/data/ice-detection` | 上报覆冰数据 |

### 告警管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/alerts` | 获取告警列表 |
| GET | `/api/alerts/:id` | 获取告警详情 |
| POST | `/api/alerts/:id/acknowledge` | 确认告警 |
| POST | `/api/alerts/:id/resolve` | 解决告警 |
| GET | `/api/alerts/summary` | 获取告警汇总统计 |

### 覆冰风险分析

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/ice-risk/:towerId` | 获取塔架覆冰风险分析 |
| GET | `/api/ice-risk/prediction/:towerId` | 获取覆冰预测 |
| GET | `/api/ice-risk/history` | 获取历史分析记录 |

### 停运策略

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/shutdown-strategies` | 获取停运策略列表 |
| GET | `/api/shutdown-strategies/:id` | 获取策略详情 |
| POST | `/api/shutdown-strategies/evaluate` | 评估停运策略 |
| POST | `/api/shutdown-strategies/:id/trigger` | 触发停运 |
| POST | `/api/shutdown-strategies/:id/cancel` | 取消停运 |

### 巡检管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/inspections` | 获取巡检记录列表 |
| GET | `/api/inspections/:id` | 获取巡检详情 |
| POST | `/api/inspections` | 创建巡检记录 |
| PUT | `/api/inspections/:id` | 更新巡检记录 |
| GET | `/api/inspections/:id/report` | 生成巡检报告 |

### 天气分析

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/weather/current` | 获取当前天气 |
| GET | `/api/weather/forecast` | 获取天气预报 |
| GET | `/api/weather/analysis` | 获取天气影响分析 |
| GET | `/api/weather/alerts` | 获取天气预警 |

### 视频复核

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/video/cameras` | 获取摄像头列表 |
| GET | `/api/video/stream/:cameraId` | 获取视频流地址 |
| POST | `/api/video/verification/request` | 申请视频复核 |
| POST | `/api/video/verification/:id/complete` | 完成复核 |
| GET | `/api/video/verification/results` | 获取复核结果 |

### 健康检查

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/health` | 系统健康检查 |
| GET | `/api/health/detailed` | 详细健康状态 |

### WebSocket 接口

连接地址: `ws://localhost:8080/ws`

#### 消息格式

```typescript
// 订阅消息
{
  "type": "subscribe",
  "channel": "alerts" // alerts | towers | sensors | system
}

// 取消订阅
{
  "type": "unsubscribe",
  "channel": "alerts"
}

// 心跳
{
  "type": "heartbeat"
}

// 告警消息
{
  "type": "alert",
  "data": {
    "id": "uuid",
    "title": "告警标题",
    "severity": "critical",
    "tower_id": "uuid",
    "message": "详细信息"
  }
}

// 塔架状态更新
{
  "type": "tower_status",
  "data": {
    "tower_id": "uuid",
    "status": "warning",
    "risk_score": 75,
    "updated_at": "2024-01-01T00:00:00Z"
  }
}

// 传感器数据推送
{
  "type": "sensor_data",
  "data": {
    "tower_id": "uuid",
    "sensor_type": "vibration",
    "value": 0.123,
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

---

## 核心算法

### 覆冰风险识别算法

采用多因素加权评分模型：

| 因素 | 权重 | 说明 |
|------|------|------|
| 覆冰厚度 | 40% | 直接测量或估算的覆冰厚度 |
| 振动振幅 | 25% | 塔架振动异常程度 |
| 风速大小 | 20% | 实时风速和蒲福风级 |
| 环境温度 | 10% | 低温促进覆冰形成 |
| 相对湿度 | 5% | 高湿度加速覆冰 |

风险等级划分：
- **低风险**: 0-60 分，正常运行
- **中风险**: 60-85 分，加强监测
- **高风险**: 85-100 分，考虑停运

### 停运策略评估

综合考虑以下因素：
1. 塔架风险评分
2. 历史故障记录
3. 当前天气状况
4. 天气预报
5. 运营时段和客流
6. 应急响应能力

### 蒲福风级转换

```
0级: <0.3 m/s - 无风
1级: 0.3-1.5 m/s - 软风
2级: 1.6-3.3 m/s - 轻风
3级: 3.4-5.4 m/s - 微风
4级: 5.5-7.9 m/s - 和风
5级: 8.0-10.7 m/s - 清劲风
6级: 10.8-13.8 m/s - 强风
7级: 13.9-17.1 m/s - 疾风
8级: 17.2-20.7 m/s - 大风
9级: 20.8-24.4 m/s - 烈风
10级: 24.5-28.4 m/s - 狂风
11级: 28.5-32.6 m/s - 暴风
12级: ≥32.7 m/s - 飓风
```

---

## 数据库设计

### ClickHouse 表结构

#### 核心时序表

1. **vibration_data** - 振动数据表
2. **wind_speed_data** - 风速数据表
3. **ice_detection_data** - 覆冰检测数据表
4. **weather_data** - 天气数据表
5. **sensor_readings** - 传感器读数表

#### 业务数据表

1. **towers** - 塔架信息表
2. **sensors** - 传感器信息表
3. **alerts** - 告警记录表
4. **shutdown_logs** - 停运记录表
5. **inspection_records** - 巡检记录表
6. **video_verification_results** - 视频复核结果表
7. **ice_analysis_results** - 覆冰分析结果表
8. **users** - 用户表

#### 表优化策略

- 使用 `MergeTree` 引擎，按日期分区
- 按 (tower_id, timestamp) 排序
- 配置 TTL 自动清理过期数据（默认保留 3 年）
- 设置适当的索引和跳数索引

---

## 配置说明

### 系统阈值配置

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `SYSTEM_ICE_THICKNESS_THRESHOLD` | 15 mm | 覆冰厚度告警阈值 |
| `SYSTEM_WIND_SPEED_THRESHOLD` | 20 m/s | 风速告警阈值（8级风） |
| `SYSTEM_VIBRATION_THRESHOLD` | 0.3 mm | 振动振幅告警阈值 |
| `SYSTEM_TEMPERATURE_THRESHOLD` | -5 °C | 低温告警阈值 |
| `SYSTEM_HUMIDITY_THRESHOLD` | 90 % | 高湿度告警阈值 |

### 数据采集间隔

| 数据类型 | 默认间隔 | 说明 |
|----------|----------|------|
| 振动数据 | 5000 ms | 每 5 秒采集一次 |
| 风速数据 | 5000 ms | 每 5 秒采集一次 |
| 天气数据 | 60000 ms | 每 60 秒采集一次 |
| 风险分析 | 30000 ms | 每 30 秒分析一次 |

---

## 安全说明

### 认证与授权

- 采用 JWT 无状态认证
- 密码使用 bcrypt 加密存储（cost = 12）
- 基于角色的权限控制（RBAC）
- 支持用户角色：管理员、运维人员、普通用户

### 网络安全

- 生产环境强制 HTTPS
- WebSocket 支持 wss 加密
- API 请求速率限制
- SQL 注入防护
- XSS 攻击防护

### 数据安全

- 敏感数据加密存储
- 操作日志完整记录
- 支持数据审计
- 定期数据备份

---

## 常见问题

### 1. 如何接入真实传感器数据？

系统提供标准的 HTTP API 和 NATS 消息接口，传感器网关可以通过以下方式接入：

- **HTTP 方式**: POST 数据到 `/api/data/*` 接口
- **NATS 方式**: 发布消息到 `sensor.vibration`, `sensor.wind_speed` 等主题

### 2. 如何集成真实视频监控？

修改 `deploy/backend/.env` 中的视频服务配置：

```env
VIDEO_SERVICE_URL=http://your-video-server:8000
VIDEO_API_KEY=your-api-key
VIDEO_ENABLED=true
```

### 3. 如何修改告警阈值？

修改 `deploy/backend/.env` 中的系统阈值配置，然后重启服务：

```bash
./deploy/scripts/restart.sh
```

### 4. 如何扩展塔架和传感器？

登录系统后，在"塔架管理"和"传感器管理"页面可以进行可视化配置。

### 5. 系统支持离线运行吗？

是的，本系统设计为边缘服务器部署，所有核心功能都可以在离线环境下运行。数据同步到云端为可选功能。

---

## 开发计划

### 已完成功能

- ✅ 后端基础框架 (Rust Axum)
- ✅ ClickHouse 数据库设计和集成
- ✅ NATS 消息队列集成
- ✅ WebSocket 实时推送
- ✅ 塔架点位建模
- ✅ 振动与风速数据采集（模拟）
- ✅ 覆冰风险识别算法
- ✅ 视频联动复核（模拟 AI 分析）
- ✅ 应急停运策略
- ✅ 巡检记录归档
- ✅ 恶劣天气影响分析
- ✅ 用户认证和权限管理
- ✅ 前端基础框架 (Qwik)
- ✅ 所有核心业务页面
- ✅ 数据可视化 (ECharts)
- ✅ 地图组件 (Leaflet)
- ✅ Docker 容器化部署
- ✅ 边缘服务器部署脚本

### 后续优化方向

- ⬜ 真实传感器数据接入
- ⬜ 真实视频流集成
- ⬜ AI 模型训练和优化
- ⬜ 云端数据同步
- ⬜ 移动端 APP 开发
- ⬜ 更多告警通知渠道（短信、邮件、钉钉、微信）
- ⬜ 单元测试和集成测试
- ⬜ 性能基准测试
- ⬜ 多语言支持

---

## 技术支持

如有问题或建议，请通过以下方式联系：

- 项目地址: `[项目仓库地址]`
- 问题反馈: `[Issue 页面]`
- 技术支持: `[联系邮箱]`

---

## 许可证

`[许可证类型]`

---

**祝您使用愉快！** 🎉
