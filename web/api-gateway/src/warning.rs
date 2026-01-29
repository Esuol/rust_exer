use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 警告级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WarningLevel {
    /// 低级别警告 - 信息性提示
    Low,
    /// 中级别警告 - 需要注意但不影响功能
    Medium,
    /// 高级别警告 - 可能影响性能或稳定性
    High,
    /// 严重警告 - 需要立即关注
    Critical,
}

impl WarningLevel {
    /// 获取警告级别的显示名称
    pub fn as_str(&self) -> &'static str {
        match self {
            WarningLevel::Low => "low",
            WarningLevel::Medium => "medium",
            WarningLevel::High => "high",
            WarningLevel::Critical => "critical",
        }
    }

    /// 从字符串创建警告级别
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(WarningLevel::Low),
            "medium" => Some(WarningLevel::Medium),
            "high" => Some(WarningLevel::High),
            "critical" => Some(WarningLevel::Critical),
            _ => None,
        }
    }
}

/// 警告信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    /// 警告ID（唯一标识）
    pub id: String,
    /// 警告级别
    pub level: WarningLevel,
    /// 警告消息
    pub message: String,
    /// 警告来源（如模块名、组件名）
    pub source: String,
    /// 警告时间戳
    pub timestamp: String,
    /// 警告详情（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, String>>,
    /// 警告计数（相同警告的重复次数）
    pub count: u64,
}

/// 警告配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningConfig {
    /// 是否启用警告收集
    pub enabled: bool,
    /// 最大警告数量（超过此数量会清理旧警告）
    pub max_warnings: usize,
    /// 警告保留时间（秒）
    pub retention_seconds: u64,
    /// 是否自动清理过期警告
    pub auto_cleanup: bool,
    /// 最低记录级别（低于此级别的警告不会被记录）
    pub min_level: WarningLevel,
}

impl Default for WarningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_warnings: 1000,
            retention_seconds: 3600, // 1小时
            auto_cleanup: true,
            min_level: WarningLevel::Low,
        }
    }
}

/// 警告统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningStats {
    /// 总警告数
    pub total_warnings: u64,
    /// 按级别统计
    pub by_level: HashMap<String, u64>,
    /// 按来源统计
    pub by_source: HashMap<String, u64>,
    /// 当前存储的警告数量
    pub current_count: usize,
}

/// 警告查询过滤器
#[derive(Debug, Clone)]
pub struct WarningFilter {
    /// 级别过滤
    pub level: Option<WarningLevel>,
    /// 来源过滤
    pub source: Option<String>,
    /// 时间范围（秒）
    pub time_range: Option<Duration>,
    /// 最大返回数量
    pub limit: Option<usize>,
}

impl Default for WarningFilter {
    fn default() -> Self {
        Self {
            level: None,
            source: None,
            time_range: None,
            limit: Some(100),
        }
    }
}

/// 警告管理器
pub struct WarningManager {
    config: WarningConfig,
    warnings: Arc<Mutex<Vec<Warning>>>,
    stats: Arc<Mutex<WarningStats>>,
    start_time: Instant,
    warning_counter: Arc<Mutex<u64>>,
}

impl WarningManager {
    /// 创建新的警告管理器
    pub fn new(config: WarningConfig) -> Self {
        Self {
            config,
            warnings: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(WarningStats {
                total_warnings: 0,
                by_level: HashMap::new(),
                by_source: HashMap::new(),
                current_count: 0,
            })),
            start_time: Instant::now(),
            warning_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// 记录警告
    pub fn record_warning(
        &self,
        level: WarningLevel,
        message: String,
        source: String,
        details: Option<HashMap<String, String>>,
    ) {
        if !self.config.enabled {
            return;
        }

        // 检查最低级别
        if level < self.config.min_level {
            return;
        }

        let timestamp = chrono::Utc::now().to_rfc3339();
        let id = {
            let mut counter = self.warning_counter.lock().unwrap();
            *counter += 1;
            format!("WARN-{:08}", *counter)
        };

        // 检查是否有相同的警告（基于消息和来源）
        let mut warnings = self.warnings.lock().unwrap();
        if let Some(existing_warning) = warnings
            .iter_mut()
            .find(|w| w.message == message && w.source == source && w.level == level)
        {
            // 更新现有警告的计数和时间戳
            existing_warning.count += 1;
            existing_warning.timestamp = timestamp.clone();
            if let Some(ref new_details) = details {
                if let Some(ref mut existing_details) = existing_warning.details {
                    existing_details.extend(new_details.clone());
                } else {
                    existing_warning.details = Some(new_details.clone());
                }
            }
            return;
        }

        // 创建新警告
        let warning = Warning {
            id,
            level,
            message: message.clone(),
            source: source.clone(),
            timestamp,
            details,
            count: 1,
        };

        // 检查是否需要清理
        if warnings.len() >= self.config.max_warnings {
            if self.config.auto_cleanup {
                self.cleanup_old_warnings();
            } else {
                // 移除最旧的警告
                warnings.remove(0);
            }
        }

        warnings.push(warning.clone());

        // 更新统计信息
        let mut stats = self.stats.lock().unwrap();
        stats.total_warnings += 1;
        stats.current_count = warnings.len();
        *stats
            .by_level
            .entry(level.as_str().to_string())
            .or_insert(0) += 1;
        *stats.by_source.entry(source).or_insert(0) += 1;
    }

    /// 获取所有警告（带过滤）
    pub fn get_warnings(&self, filter: Option<WarningFilter>) -> Vec<Warning> {
        let warnings = self.warnings.lock().unwrap();
        let filter = filter.unwrap_or_default();
        let now = chrono::Utc::now();

        let mut result: Vec<Warning> = warnings
            .iter()
            .filter(|w| {
                // 级别过滤
                if let Some(ref level) = filter.level {
                    if w.level != *level {
                        return false;
                    }
                }

                // 来源过滤
                if let Some(ref source) = filter.source {
                    if !w.source.contains(source) {
                        return false;
                    }
                }

                // 时间范围过滤
                if let Some(ref time_range) = filter.time_range {
                    if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&w.timestamp) {
                        let warning_time = timestamp.with_timezone(&chrono::Utc);
                        let elapsed = now.signed_duration_since(warning_time);
                        if elapsed > chrono::Duration::from_std(*time_range).unwrap() {
                            return false;
                        }
                    }
                }

                true
            })
            .cloned()
            .collect();

        // 按时间戳倒序排序（最新的在前）
        result.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // 应用限制
        if let Some(limit) = filter.limit {
            result.truncate(limit);
        }

        result
    }

    /// 获取警告统计信息
    pub fn get_stats(&self) -> WarningStats {
        self.stats.lock().unwrap().clone()
    }

    /// 清理过期警告
    pub fn cleanup_old_warnings(&self) {
        if !self.config.auto_cleanup {
            return;
        }

        let now = chrono::Utc::now();
        let retention_duration = chrono::Duration::seconds(self.config.retention_seconds as i64);

        let mut warnings = self.warnings.lock().unwrap();
        warnings.retain(|w| {
            if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&w.timestamp) {
                let warning_time = timestamp.with_timezone(&chrono::Utc);
                let elapsed = now.signed_duration_since(warning_time);
                elapsed <= retention_duration
            } else {
                true // 保留无法解析时间戳的警告
            }
        });

        // 更新统计
        let mut stats = self.stats.lock().unwrap();
        stats.current_count = warnings.len();
    }

    /// 清除所有警告
    pub fn clear_all(&self) {
        let mut warnings = self.warnings.lock().unwrap();
        warnings.clear();

        let mut stats = self.stats.lock().unwrap();
        stats.current_count = 0;
    }

    /// 清除特定级别的警告
    pub fn clear_by_level(&self, level: WarningLevel) {
        let mut warnings = self.warnings.lock().unwrap();
        warnings.retain(|w| w.level != level);

        let mut stats = self.stats.lock().unwrap();
        stats.current_count = warnings.len();
        stats.by_level.remove(level.as_str());
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// 更新配置
    pub fn update_config(&mut self, config: WarningConfig) {
        self.config = config;
    }

    /// 获取配置
    pub fn get_config(&self) -> &WarningConfig {
        &self.config
    }
}

// 实现 PartialOrd 和 Ord 用于 WarningLevel 比较
impl PartialOrd for WarningLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_val = match self {
            WarningLevel::Low => 1,
            WarningLevel::Medium => 2,
            WarningLevel::High => 3,
            WarningLevel::Critical => 4,
        };
        let other_val = match other {
            WarningLevel::Low => 1,
            WarningLevel::Medium => 2,
            WarningLevel::High => 3,
            WarningLevel::Critical => 4,
        };
        self_val.partial_cmp(&other_val)
    }
}

impl Ord for WarningLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// 记录警告的便捷宏
#[macro_export]
macro_rules! record_warning {
    ($manager:expr, $level:expr, $source:expr, $msg:expr) => {
        $manager.record_warning($level, $msg.to_string(), $source.to_string(), None);
    };
    ($manager:expr, $level:expr, $source:expr, $msg:expr, $details:expr) => {
        $manager.record_warning(
            $level,
            $msg.to_string(),
            $source.to_string(),
            Some($details),
        );
    };
}

/// 快速记录低级别警告的宏
#[macro_export]
macro_rules! warn_low {
    ($manager:expr, $source:expr, $msg:expr) => {
        $crate::record_warning!($manager, $crate::warning::WarningLevel::Low, $source, $msg);
    };
}

/// 快速记录中级别警告的宏
#[macro_export]
macro_rules! warn_medium {
    ($manager:expr, $source:expr, $msg:expr) => {
        $crate::record_warning!(
            $manager,
            $crate::warning::WarningLevel::Medium,
            $source,
            $msg
        );
    };
}

/// 快速记录高级别警告的宏
#[macro_export]
macro_rules! warn_high {
    ($manager:expr, $source:expr, $msg:expr) => {
        $crate::record_warning!($manager, $crate::warning::WarningLevel::High, $source, $msg);
    };
}

/// 快速记录严重警告的宏
#[macro_export]
macro_rules! warn_critical {
    ($manager:expr, $source:expr, $msg:expr) => {
        $crate::record_warning!(
            $manager,
            $crate::warning::WarningLevel::Critical,
            $source,
            $msg
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warning_level_from_str() {
        assert_eq!(WarningLevel::from_str("low"), Some(WarningLevel::Low));
        assert_eq!(WarningLevel::from_str("HIGH"), Some(WarningLevel::High));
        assert_eq!(WarningLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_warning_manager_creation() {
        let config = WarningConfig::default();
        let manager = WarningManager::new(config);
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_record_warning() {
        let config = WarningConfig {
            enabled: true,
            max_warnings: 100,
            retention_seconds: 3600,
            auto_cleanup: false,
            min_level: WarningLevel::Low,
        };
        let manager = WarningManager::new(config);

        manager.record_warning(
            WarningLevel::Medium,
            "Test warning".to_string(),
            "test_source".to_string(),
            None,
        );

        let warnings = manager.get_warnings(None);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].message, "Test warning");
        assert_eq!(warnings[0].level, WarningLevel::Medium);
    }

    #[test]
    fn test_warning_deduplication() {
        let config = WarningConfig::default();
        let manager = WarningManager::new(config);

        manager.record_warning(
            WarningLevel::Low,
            "Duplicate warning".to_string(),
            "source".to_string(),
            None,
        );
        manager.record_warning(
            WarningLevel::Low,
            "Duplicate warning".to_string(),
            "source".to_string(),
            None,
        );

        let warnings = manager.get_warnings(None);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].count, 2);
    }

    #[test]
    fn test_warning_filter() {
        let config = WarningConfig::default();
        let manager = WarningManager::new(config);

        manager.record_warning(
            WarningLevel::Low,
            "Low warning".to_string(),
            "source1".to_string(),
            None,
        );
        manager.record_warning(
            WarningLevel::High,
            "High warning".to_string(),
            "source2".to_string(),
            None,
        );

        let filter = WarningFilter {
            level: Some(WarningLevel::High),
            ..Default::default()
        };
        let warnings = manager.get_warnings(Some(filter));
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].level, WarningLevel::High);
    }

    #[test]
    fn test_warning_stats() {
        let config = WarningConfig::default();
        let manager = WarningManager::new(config);

        manager.record_warning(
            WarningLevel::Low,
            "Warning 1".to_string(),
            "source1".to_string(),
            None,
        );
        manager.record_warning(
            WarningLevel::High,
            "Warning 2".to_string(),
            "source2".to_string(),
            None,
        );

        let stats = manager.get_stats();
        assert_eq!(stats.total_warnings, 2);
        assert_eq!(stats.current_count, 2);
    }

    #[test]
    fn test_clear_warnings() {
        let config = WarningConfig::default();
        let manager = WarningManager::new(config);

        manager.record_warning(
            WarningLevel::Low,
            "Warning".to_string(),
            "source".to_string(),
            None,
        );
        manager.clear_all();

        let warnings = manager.get_warnings(None);
        assert_eq!(warnings.len(), 0);
    }

    #[test]
    fn test_clear_by_level() {
        let config = WarningConfig::default();
        let manager = WarningManager::new(config);

        manager.record_warning(
            WarningLevel::Low,
            "Warning".to_string(),
            "source".to_string(),
            None,
        );
        manager.clear_by_level(WarningLevel::Low);

        let warnings = manager.get_warnings(None);
        assert_eq!(warnings.len(), 0);
    }

    #[test]
    fn test_get_stats() {
        let config = WarningConfig::default();
        let manager = WarningManager::new(config);

        manager.record_warning(
            WarningLevel::Low,
            "Warning".to_string(),
            "source".to_string(),
            None,
        );
    }

    #[test]
    fn test_get_config() {
        let config = WarningConfig::default();
        let manager = WarningManager::new(config);
        assert_eq!(manager.get_config(), &config);
    }

    #[test]
    fn test_update_config() {
        let mut config = WarningConfig::default();
        config.enabled = false;
        let manager = WarningManager::new(config);
        assert_eq!(manager.get_config().enabled, false);
    }

    #[test]
    fn test_is_enabled() {
        let config = WarningConfig::default();
        let manager = WarningManager::new(config);
        assert_eq!(manager.is_enabled(), true);
    }

    #[test]
    fn test_warning_stats_v2() {
        let config = WarningConfig::default();
        let manager = WarningManager::new(config);

        manager.record_warning(
            WarningLevel::Low,
            "Warning 1".to_string(),
            "source1".to_string(),
            None,
        );
        manager.record_warning(
            WarningLevel::High,
            "Warning 2".to_string(),
            "source2".to_string(),
            None,
        );

        let stats = manager.get_stats();
        assert_eq!(stats.total_warnings, 2);
        assert_eq!(stats.current_count, 2);
    }

    #[test]
    fn test_warning_deduplication_v3() {
        let config = WarningConfig::default();
        let manager = WarningManager::new(config);

        manager.record_warning(
            WarningLevel::Low,
            "Duplicate warning".to_string(),
            "source".to_string(),
            None,
        );
        manager.record_warning(
            WarningLevel::Low,
            "Duplicate warning".to_string(),
            "source".to_string(),
            None,
        );

        let warnings = manager.get_warnings(None);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].count, 2);
    }

    #[test]
    fn test_warning_stats_v4() {
        let config = WarningConfig::default();
        let manager = WarningManager::new(config);

        manager.record_warning(
            WarningLevel::Low,
            "Warning 1".to_string(),
            "source1".to_string(),
            None,
        );
        manager.record_warning(
            WarningLevel::High,
            "Warning 2".to_string(),
            "source2".to_string(),
            None,
        );

        let stats = manager.get_stats();
        assert_eq!(stats.total_warnings, 2);
        assert_eq!(stats.current_count, 2);
    }
}
