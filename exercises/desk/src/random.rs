use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// 随机数生成器模块
///
/// 提供多种随机数生成功能，包括整数、浮点数、字符串等
pub struct RandomGenerator {
    seed: u64,
}

impl RandomGenerator {
    /// 创建一个新的随机数生成器实例
    ///
    /// # Arguments
    /// * `seed` - 随机数种子
    ///
    /// # Returns
    /// 返回一个新的RandomGenerator实例
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// 使用当前时间作为种子创建随机数生成器
    pub fn new_with_time() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self::new(time)
    }

    /// 生成下一个随机数
    fn next(&mut self) -> u64 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        self.seed
    }

    /// 生成指定范围内的随机整数
    ///
    /// # Arguments
    /// * `min` - 最小值（包含）
    /// * `max` - 最大值（包含）
    ///
    /// # Returns
    /// 返回指定范围内的随机整数
    pub fn random_int(&mut self, min: i32, max: i32) -> i32 {
        if min > max {
            return min;
        }
        let range = (max - min + 1) as u64;
        let random = self.next() % range;
        min + random as i32
    }

    /// 生成0到1之间的随机浮点数
    ///
    /// # Returns
    /// 返回0.0到1.0之间的随机浮点数
    pub fn random_float(&mut self) -> f64 {
        let random = self.next();
        (random as f64) / (u64::MAX as f64)
    }

    /// 生成指定范围内的随机浮点数
    ///
    /// # Arguments
    /// * `min` - 最小值（包含）
    /// * `max` - 最大值（包含）
    ///
    /// # Returns
    /// 返回指定范围内的随机浮点数
    pub fn random_float_range(&mut self, min: f64, max: f64) -> f64 {
        if min > max {
            return min;
        }
        min + self.random_float() * (max - min)
    }

    /// 从数组中随机选择一个元素
    ///
    /// # Arguments
    /// * `items` - 要选择的数组
    ///
    /// # Returns
    /// 返回随机选择的元素，如果数组为空则返回None
    pub fn random_choice<T>(&mut self, items: &[T]) -> Option<&T> {
        if items.is_empty() {
            return None;
        }
        let index = self.random_int(0, (items.len() - 1) as i32) as usize;
        Some(&items[index])
    }

    /// 生成指定长度的随机字符串
    ///
    /// # Arguments
    /// * `length` - 字符串长度
    ///
    /// # Returns
    /// 返回指定长度的随机字符串
    pub fn random_string(&mut self, length: usize) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut result = String::with_capacity(length);

        for _ in 0..length {
            let index = self.random_int(0, (CHARS.len() - 1) as i32) as usize;
            result.push(CHARS[index] as char);
        }

        result
    }

    /// 生成随机布尔值
    ///
    /// # Arguments
    /// * `probability` - 返回true的概率（0.0到1.0之间）
    ///
    /// # Returns
    /// 根据概率返回随机布尔值
    pub fn random_bool(&mut self, probability: f64) -> bool {
        self.random_float() < probability
    }

    /// 生成50%概率的随机布尔值
    ///
    /// # Returns
    /// 返回随机布尔值
    pub fn random_bool_50(&mut self) -> bool {
        self.random_bool(0.5)
    }

    /// 打乱数组顺序
    ///
    /// # Arguments
    /// * `items` - 要打乱的数组
    ///
    /// # Returns
    /// 返回打乱后的新数组
    pub fn shuffle<T: Clone>(&mut self, items: &[T]) -> Vec<T> {
        let mut result: Vec<T> = items.to_vec();
        let len = result.len();

        for i in 0..len {
            let j = self.random_int(i as i32, (len - 1) as i32) as usize;
            result.swap(i, j);
        }

        result
    }
}

/// 便捷函数：生成指定范围内的随机整数
pub fn random_int(seed: u64, min: i32, max: i32) -> i32 {
    let mut generator = RandomGenerator::new(seed);
    generator.random_int(min, max)
}

/// 便捷函数：生成随机浮点数
pub fn random_float(seed: u64) -> f64 {
    let mut generator = RandomGenerator::new(seed);
    generator.random_float()
}

/// 便捷函数：从数组中随机选择元素
pub fn random_choice<T>(seed: u64, items: &[T]) -> Option<&T> {
    let mut generator = RandomGenerator::new(seed);
    generator.random_choice(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_generator_new() {
        let generator = RandomGenerator::new(12345);
        assert_eq!(generator.seed, 12345);
    }

    #[test]
    fn test_random_generator_with_time() {
        let generator = RandomGenerator::new_with_time();
        assert!(generator.seed > 0);
    }

    #[test]
    fn test_random_int() {
        let mut generator = RandomGenerator::new(12345);
        let result = generator.random_int(1, 10);
        assert!(result >= 1 && result <= 10);
    }

    #[test]
    fn test_random_int_same_seed() {
        let mut generator1 = RandomGenerator::new(12345);
        let mut generator2 = RandomGenerator::new(12345);

        let result1 = generator1.random_int(1, 100);
        let result2 = generator2.random_int(1, 100);

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_random_int_invalid_range() {
        let mut generator = RandomGenerator::new(12345);
        let result = generator.random_int(10, 1);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_random_float() {
        let mut generator = RandomGenerator::new(12345);
        let result = generator.random_float();
        assert!(result >= 0.0 && result <= 1.0);
    }

    #[test]
    fn test_random_float_range() {
        let mut generator = RandomGenerator::new(12345);
        let result = generator.random_float_range(5.0, 10.0);
        assert!(result >= 5.0 && result <= 10.0);
    }

    #[test]
    fn test_random_choice() {
        let mut generator = RandomGenerator::new(12345);
        let items = vec![1, 2, 3, 4, 5];
        let result = generator.random_choice(&items);
        assert!(result.is_some());
        assert!(items.contains(result.unwrap()));
    }

    #[test]
    fn test_random_choice_empty() {
        let mut generator = RandomGenerator::new(12345);
        let items: Vec<i32> = vec![];
        let result = generator.random_choice(&items);
        assert!(result.is_none());
    }

    #[test]
    fn test_random_string() {
        let mut generator = RandomGenerator::new(12345);
        let result = generator.random_string(10);
        assert_eq!(result.len(), 10);
        assert!(result.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_random_string_zero_length() {
        let mut generator = RandomGenerator::new(12345);
        let result = generator.random_string(0);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_random_bool() {
        let mut generator = RandomGenerator::new(12345);
        let result = generator.random_bool(0.5);
        assert!(result == true || result == false);
    }

    #[test]
    fn test_random_bool_50() {
        let mut generator = RandomGenerator::new(12345);
        let result = generator.random_bool_50();
        assert!(result == true || result == false);
    }

    #[test]
    fn test_shuffle() {
        let mut generator = RandomGenerator::new(12345);
        let original = vec![1, 2, 3, 4, 5];
        let shuffled = generator.shuffle(&original);

        assert_eq!(shuffled.len(), original.len());
        assert!(shuffled.iter().all(|&x| original.contains(&x)));
    }

    #[test]
    fn test_shuffle_empty() {
        let mut generator = RandomGenerator::new(12345);
        let original: Vec<i32> = vec![];
        let shuffled = generator.shuffle(&original);
        assert_eq!(shuffled.len(), 0);
    }

    #[test]
    fn test_convenience_functions() {
        let seed = 12345;

        let int_result = random_int(seed, 1, 10);
        assert!(int_result >= 1 && int_result <= 10);

        let float_result = random_float(seed);
        assert!(float_result >= 0.0 && float_result <= 1.0);

        let items = vec![1, 2, 3];
        let choice_result = random_choice(seed, &items);
        assert!(choice_result.is_some());
        assert!(items.contains(choice_result.unwrap()));
    }

    #[test]
    fn test_deterministic_behavior() {
        let seed = 12345;
        let mut generator1 = RandomGenerator::new(seed);
        let mut generator2 = RandomGenerator::new(seed);

        // 测试多个随机数生成的一致性
        for _ in 0..10 {
            assert_eq!(generator1.random_int(1, 100), generator2.random_int(1, 100));
            assert_eq!(generator1.random_float(), generator2.random_float());
        }
    }

    #[test]
    fn test_edge_cases() {
        let mut generator = RandomGenerator::new(12345);

        // 测试边界值
        assert_eq!(generator.random_int(5, 5), 5);
        assert_eq!(generator.random_float_range(3.0, 3.0), 3.0);

        // 测试负数范围
        let negative_result = generator.random_int(-10, -1);
        assert!(negative_result >= -10 && negative_result <= -1);
    }
}
