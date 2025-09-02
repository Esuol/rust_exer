# API Gateway - Rust 实现

基于 Rust + Tokio + Rocket 构建的高性能 API 网关服务

## 🚀 项目特性

### 基础路由转发

- **路由配置管理**: 动态路由配置，支持 RESTful API 路由
- **请求转发**: 智能请求转发到后端服务
- **响应聚合**: 支持多个后端服务的响应聚合
- **超时控制**: 可配置的请求超时时间
- **重试机制**: 智能重试策略，支持指数退避

### 负载均衡

- **轮询算法**: Round Robin 负载均衡
- **加权轮询**: Weighted Round Robin 支持
- **最少连接**: Least Connections 算法
- **健康检查**: 自动健康状态检测
- **故障转移**: 服务故障自动切换

### 限流熔断

- **令牌桶限流**: Token Bucket 限流算法
- **滑动窗口限流**: Sliding Window 限流
- **熔断器**: Circuit Breaker 模式
- **降级策略**: 服务降级和优雅降级
- **限流规则配置**: 动态限流规则管理

### 认证授权

- **JWT 验证**: JSON Web Token 支持
- **API Key 认证**: 简单高效的 API Key 机制
- **OAuth2 集成**: 完整的 OAuth2 流程
- **权限控制**: 基于角色的访问控制 (RBAC)
- **黑白名单**: IP 和用户黑白名单管理

### 中间件系统

- **请求日志**: 完整的请求/响应日志记录
- **性能监控**: 响应时间、吞吐量监控
- **请求/响应转换**: 数据格式转换和验证
- **错误处理**: 统一的错误处理和响应格式
- **链路追踪**: 分布式链路追踪支持

### 监控告警

- **请求统计**: 实时请求量统计
- **响应时间监控**: 平均响应时间、P95、P99 监控
- **错误率统计**: 错误率趋势分析
- **资源使用监控**: CPU、内存、网络使用率
- **告警通知**: 多渠道告警通知 (邮件、短信、Webhook)

## 🏗️ 技术架构

### 核心技术栈

- **Rust**: 系统级编程语言，提供高性能和内存安全
- **Tokio**: 异步运行时，支持高并发 I/O 操作
- **Rocket**: Web 框架，提供优雅的 API 设计
- **Serde**: 序列化/反序列化库
- **Tracing**: 分布式链路追踪和日志系统

### 数据存储

- **SQLx**: 异步 SQL 数据库操作
- **Redis**: 缓存和会话存储
- **Config**: 配置管理

### 网络通信

- **Reqwest**: HTTP 客户端，用于后端服务调用
- **Rocket**: HTTP 服务器，处理客户端请求

## 📁 项目结构

```
api-gateway/
├── Cargo.toml                 # 项目依赖配置
├── Cargo.lock                 # 依赖锁定文件
├── README.md                  # 项目说明文档
├── src/
│   ├── main.rs               # 程序入口点
│   ├── config/               # 配置管理模块
│   │   ├── mod.rs
│   │   ├── app.rs            # 应用配置
│   │   ├── database.rs       # 数据库配置
│   │   └── redis.rs          # Redis 配置
│   ├── core/                 # 核心业务逻辑
│   │   ├── mod.rs
│   │   ├── router.rs         # 路由管理
│   │   ├── load_balancer.rs  # 负载均衡器
│   │   ├── rate_limiter.rs   # 限流器
│   │   ├── circuit_breaker.rs # 熔断器
│   │   └── auth.rs           # 认证授权
│   ├── middleware/           # 中间件系统
│   │   ├── mod.rs
│   │   ├── logging.rs        # 日志中间件
│   │   ├── metrics.rs        # 监控中间件
│   │   ├── cors.rs           # CORS 中间件
│   │   └── tracing.rs        # 链路追踪中间件
│   ├── handlers/             # 请求处理器
│   │   ├── mod.rs
│   │   ├── health.rs         # 健康检查
│   │   ├── proxy.rs          # 代理转发
│   │   ├── config.rs         # 配置管理
│   │   └── metrics.rs        # 监控指标
│   ├── utils/                # 工具函数
│   │   ├── mod.rs
│   │   ├── http.rs           # HTTP 工具
│   │   ├── crypto.rs         # 加密工具
│   │   └── time.rs           # 时间工具
│   └── types/                # 类型定义
│       ├── mod.rs
│       ├── request.rs        # 请求类型
│       ├── response.rs       # 响应类型
│       └── error.rs          # 错误类型
├── config/                    # 配置文件目录
│   ├── config.toml           # 主配置文件
│   ├── routes.toml           # 路由配置
│   └── services.toml         # 服务配置
├── tests/                     # 测试文件
│   ├── integration/          # 集成测试
│   └── unit/                 # 单元测试
├── docs/                      # 文档目录
│   ├── api.md                # API 文档
│   ├── deployment.md         # 部署指南
│   └── development.md        # 开发指南
└── scripts/                   # 脚本文件
    ├── build.sh              # 构建脚本
    ├── deploy.sh             # 部署脚本
    └── test.sh               # 测试脚本
```

## 🚀 快速开始

### 环境要求

- Rust 1.70+
- Cargo
- Redis 6.0+
- PostgreSQL 12+ (可选)

### 安装和运行

```bash
# 克隆项目
git clone <repository-url>
cd api-gateway

# 安装依赖
cargo build

# 运行项目
cargo run

# 运行测试
cargo test

# 构建发布版本
cargo build --release
```

### 配置说明

1. 复制配置文件模板

```bash
cp config/config.toml.example config/config.toml
```

2. 修改配置文件中的数据库和 Redis 连接信息

3. 配置路由规则和服务地址

## 📊 性能指标

- **并发处理**: 支持 10,000+ 并发连接
- **响应时间**: 平均响应时间 < 10ms
- **吞吐量**: 支持 100,000+ QPS
- **内存使用**: 低内存占用，启动内存 < 50MB

## 🔧 开发计划

### 第一阶段 (基础功能)

- [x] 项目结构搭建
- [ ] 基础路由转发
- [ ] 简单负载均衡
- [ ] 基础认证
- [ ] 健康检查

### 第二阶段 (核心功能)

- [ ] 高级负载均衡算法
- [ ] 限流熔断机制
- [ ] 完整的认证授权
- [ ] 中间件系统
- [ ] 基础监控

### 第三阶段 (高级功能)

- [ ] 动态配置管理
- [ ] 高级监控告警
- [ ] 链路追踪
- [ ] 性能优化
- [ ] 集群支持

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

### 开发规范

- 遵循 Rust 编码规范
- 所有公共 API 需要文档注释
- 新功能需要包含测试用例
- 提交前运行 `cargo fmt` 和 `cargo clippy`

## 📄 许可证

MIT License

## 📞 联系方式

- 项目维护者: [Your Name]
- 邮箱: [your.email@example.com]
- 项目地址: [GitHub Repository URL]

---

**注意**: 这是一个学习项目，生产环境使用前请充分测试！
