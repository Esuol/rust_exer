use std::collections::HashMap;
use std::collections::HashSet;

/// 数据生成器模块
/// 提供各种常用的数据生成功能，用于练习和测试
pub struct Generator;

impl Generator {
    /// 生成指定范围内的随机整数
    pub fn random_int(min: i32, max: i32) -> i32 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let mut x = seed;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;

        min + (x % (max - min + 1) as i32)
    }

    /// 生成随机浮点数
    pub fn random_float(min: f64, max: f64) -> f64 {
        let random_int = Self::random_int(0, 10000);
        min + (random_int as f64 / 10000.0) * (max - min)
    }

    /// 生成随机字符串
    pub fn random_string(length: usize) -> String {
        const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut result = String::with_capacity(length);

        for _ in 0..length {
            let index = Self::random_int(0, (CHARS.len() - 1) as i32) as usize;
            result.push(CHARS[index] as char);
        }

        result
    }

    /// 生成随机单词
    pub fn random_word() -> String {
        const WORDS: &[&str] = &[
            "hello",
            "world",
            "rust",
            "programming",
            "computer",
            "algorithm",
            "data",
            "structure",
            "function",
            "variable",
            "constant",
            "string",
            "integer",
            "float",
            "boolean",
            "array",
            "vector",
            "hashmap",
            "struct",
            "enum",
            "trait",
            "module",
            "crate",
            "package",
        ];

        let index = Self::random_int(0, (WORDS.len() - 1) as i32) as usize;
        WORDS[index].to_string()
    }

    /// 生成随机整数数组
    pub fn random_int_array(size: usize, min: i32, max: i32) -> Vec<i32> {
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            result.push(Self::random_int(min, max));
        }
        result
    }

    /// 生成斐波那契数列
    pub fn fibonacci(n: usize) -> Vec<u64> {
        if n == 0 {
            return vec![];
        }
        if n == 1 {
            return vec![0];
        }
        if n == 2 {
            return vec![0, 1];
        }

        let mut result = vec![0, 1];
        for i in 2..n {
            result.push(result[i - 1] + result[i - 2]);
        }
        result
    }

    /// 生成素数数组
    pub fn primes(max: usize) -> Vec<usize> {
        if max < 2 {
            return vec![];
        }

        let mut is_prime = vec![true; max + 1];
        is_prime[0] = false;
        is_prime[1] = false;

        for i in 2..=((max as f64).sqrt() as usize) {
            if is_prime[i] {
                for j in (i * i..=max).step_by(i) {
                    is_prime[j] = false;
                }
            }
        }

        is_prime
            .into_iter()
            .enumerate()
            .filter(|&(_, is_prime)| is_prime)
            .map(|(i, _)| i)
            .collect()
    }

    /// 生成随机浮点数数组
    pub fn random_float_array(size: usize, min: f64, max: f64) -> Vec<f64> {
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            result.push(Self::random_float(min, max));
        }
        result
    }

    /// 生成随机字符串数组
    pub fn random_string_array(size: usize, length: usize) -> Vec<String> {
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            result.push(Self::random_string(length));
        }
        result
    }

    /// 生成随机单词数组
    pub fn random_word_array(size: usize) -> Vec<String> {
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            result.push(Self::random_word());
        }
        result
    }

    /// 生成随机 HashMap
    pub fn random_hashmap(size: usize) -> HashMap<String, i32> {
        let mut map = HashMap::new();
        for _ in 0..size {
            let key = Self::random_string(5);
            let value = Self::random_int(1, 100);
            map.insert(key, value);
        }
        map
    }

    /// 生成随机 HashSet
    pub fn random_hashset(size: usize) -> HashSet<i32> {
        let mut set = HashSet::new();
        while set.len() < size {
            set.insert(Self::random_int(1, 1000));
        }
        set
    }

    /// 生成排序后的随机数组
    pub fn sorted_random_array(size: usize, min: i32, max: i32) -> Vec<i32> {
        let mut array = Self::random_int_array(size, min, max);
        array.sort();
        array
    }

    /// 生成逆序的随机数组
    pub fn reverse_sorted_random_array(size: usize, min: i32, max: i32) -> Vec<i32> {
        let mut array = Self::random_int_array(size, min, max);
        array.sort_by(|a, b| b.cmp(a));
        array
    }

    /// 生成重复元素的数组
    pub fn repeated_array(size: usize, value: i32) -> Vec<i32> {
        vec![value; size]
    }

    /// 生成等差数列
    pub fn arithmetic_sequence(start: i32, step: i32, size: usize) -> Vec<i32> {
        let mut result = Vec::with_capacity(size);
        for i in 0..size {
            result.push(start + (i as i32 * step));
        }
        result
    }

    /// 生成几何数列
    pub fn geometric_sequence(start: i32, ratio: i32, size: usize) -> Vec<i32> {
        let mut result = Vec::with_capacity(size);
        let mut current = start;
        for _ in 0..size {
            result.push(current);
            current *= ratio;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_int() {
        let num = Generator::random_int(1, 100);
        assert!(num >= 1 && num <= 100);
    }

    #[test]
    fn test_random_float() {
        let num = Generator::random_float(0.0, 1.0);
        assert!(num >= 0.0 && num <= 1.0);
    }

    #[test]
    fn test_random_string() {
        let s = Generator::random_string(10);
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn test_random_word() {
        let word = Generator::random_word();
        assert!(!word.is_empty());
    }

    #[test]
    fn test_random_int_array() {
        let array = Generator::random_int_array(5, 1, 100);
        assert_eq!(array.len(), 5);
        for &num in &array {
            assert!(num >= 1 && num <= 100);
        }
    }

    #[test]
    fn test_fibonacci() {
        let fib = Generator::fibonacci(10);
        assert_eq!(fib, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    }

    #[test]
    fn test_primes() {
        let primes = Generator::primes(20);
        assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19]);
    }

    #[test]
    fn test_random_float_array() {
        let array = Generator::random_float_array(5, 0.0, 1.0);
        assert_eq!(array.len(), 5);
        for &num in &array {
            assert!(num >= 0.0 && num <= 1.0);
        }
    }

    #[test]
    fn test_random_string_array() {
        let array = Generator::random_string_array(3, 5);
        assert_eq!(array.len(), 3);
        for s in &array {
            assert_eq!(s.len(), 5);
        }
    }

    #[test]
    fn test_random_word_array() {
        let array = Generator::random_word_array(5);
        assert_eq!(array.len(), 5);
        for word in &array {
            assert!(!word.is_empty());
        }
    }

    #[test]
    fn test_random_hashmap() {
        let map = Generator::random_hashmap(3);
        assert_eq!(map.len(), 3);
        for (_, &value) in &map {
            assert!(value >= 1 && value <= 100);
        }
    }

    #[test]
    fn test_random_hashset() {
        let set = Generator::random_hashset(5);
        assert_eq!(set.len(), 5);
        for &value in &set {
            assert!(value >= 1 && value <= 1000);
        }
    }

    #[test]
    fn test_sorted_random_array() {
        let array = Generator::sorted_random_array(5, 1, 100);
        assert_eq!(array.len(), 5);
        for i in 1..array.len() {
            assert!(array[i] >= array[i - 1]);
        }
    }

    #[test]
    fn test_reverse_sorted_random_array() {
        let array = Generator::reverse_sorted_random_array(5, 1, 100);
        assert_eq!(array.len(), 5);
        for i in 1..array.len() {
            assert!(array[i] <= array[i - 1]);
        }
    }

    #[test]
    fn test_repeated_array() {
        let array = Generator::repeated_array(5, 42);
        assert_eq!(array, vec![42, 42, 42, 42, 42]);
    }

    #[test]
    fn test_arithmetic_sequence() {
        let seq = Generator::arithmetic_sequence(1, 2, 5);
        assert_eq!(seq, vec![1, 3, 5, 7, 9]);
    }

    #[test]
    fn test_geometric_sequence() {
        let seq = Generator::geometric_sequence(1, 2, 5);
        assert_eq!(seq, vec![1, 2, 4, 8, 16]);
    }
}
