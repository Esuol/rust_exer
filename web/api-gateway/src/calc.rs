/// 计算器模块
/// 提供基本的数学计算功能
use std::fmt;

/// 运算符枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Operator {
    /// 从字符创建运算符
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Operator::Add),
            '-' => Some(Operator::Subtract),
            '*' => Some(Operator::Multiply),
            '/' => Some(Operator::Divide),
            _ => None,
        }
    }
}

/// 计算错误类型
#[derive(Debug, Clone)]
pub enum CalcError {
    DivisionByZero,
    InvalidOperator(String),
    InvalidExpression(String),
    EmptyArray,
    NegativeNumber,
    InvalidNumber(String),
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalcError::DivisionByZero => write!(f, "除数不能为零"),
            CalcError::InvalidOperator(op) => write!(f, "不支持的运算符: {}", op),
            CalcError::InvalidExpression(expr) => {
                write!(f, "表达式计算失败: {}", expr)
            }
            CalcError::EmptyArray => write!(f, "数组不能为空"),
            CalcError::NegativeNumber => write!(f, "不能计算负数的平方根"),
            CalcError::InvalidNumber(msg) => write!(f, "无效的数字: {}", msg),
        }
    }
}

impl std::error::Error for CalcError {}

/// 执行基本四则运算
///
/// # 参数
/// * `a` - 第一个操作数
/// * `b` - 第二个操作数
/// * `operator` - 运算符
///
/// # 返回
/// 计算结果，如果除数为零则返回错误
///
/// # 示例
/// ```
/// use calc::calculate;
/// use calc::Operator;
///
/// let result = calculate(10.0, 5.0, Operator::Add).unwrap();
/// assert_eq!(result, 15.0);
/// ```
pub fn calculate(a: f64, b: f64, operator: Operator) -> Result<f64, CalcError> {
    match operator {
        Operator::Add => Ok(a + b),
        Operator::Subtract => Ok(a - b),
        Operator::Multiply => Ok(a * b),
        Operator::Divide => {
            if b == 0.0 {
                Err(CalcError::DivisionByZero)
            } else {
                Ok(a / b)
            }
        }
    }
}

/// 计算表达式（支持简单的数学表达式）
///
/// # 参数
/// * `expression` - 数学表达式字符串，例如: "2 + 3 * 4"
///
/// # 返回
/// 计算结果
///
/// # 注意
/// 这是一个简化版本，仅支持基本的四则运算和括号
/// 生产环境应使用更完善的表达式解析器
pub fn evaluate_expression(expression: &str) -> Result<f64, CalcError> {
    // 移除所有空白字符
    let cleaned: String = expression.chars().filter(|c| !c.is_whitespace()).collect();

    // 验证表达式只包含数字、运算符和括号
    if !cleaned
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '.'))
    {
        return Err(CalcError::InvalidExpression(
            "表达式包含非法字符".to_string(),
        ));
    }

    // 简单的表达式计算（使用递归下降解析）
    // 注意：这是一个简化实现，不支持运算符优先级
    // 实际应用中应使用更完善的解析器
    parse_expression(&cleaned)
}

/// 简单的表达式解析器（递归下降）
fn parse_expression(expr: &str) -> Result<f64, CalcError> {
    // 这是一个非常简化的实现
    // 实际应该实现完整的表达式解析器（支持运算符优先级、括号等）
    // 这里使用一个简单的栈来计算

    // 对于简单情况，我们可以尝试直接解析
    // 但为了安全，我们只处理简单的二元运算
    if let Some(pos) = expr.rfind('+') {
        let left = parse_expression(&expr[..pos])?;
        let right = parse_expression(&expr[pos + 1..])?;
        return Ok(left + right);
    }

    if let Some(pos) = expr.rfind('-') {
        if pos > 0 {
            let left = parse_expression(&expr[..pos])?;
            let right = parse_expression(&expr[pos + 1..])?;
            return Ok(left - right);
        }
    }

    if let Some(pos) = expr.rfind('*') {
        let left = parse_expression(&expr[..pos])?;
        let right = parse_expression(&expr[pos + 1..])?;
        return Ok(left * right);
    }

    if let Some(pos) = expr.rfind('/') {
        let left = parse_expression(&expr[..pos])?;
        let right = parse_expression(&expr[pos + 1..])?;
        if right == 0.0 {
            return Err(CalcError::DivisionByZero);
        }
        return Ok(left / right);
    }

    // 处理括号
    if expr.starts_with('(') && expr.ends_with(')') {
        return parse_expression(&expr[1..expr.len() - 1]);
    }

    // 解析数字
    expr.parse::<f64>()
        .map_err(|_| CalcError::InvalidExpression(format!("无法解析数字: {}", expr)))
}

/// 计算百分比
///
/// # 参数
/// * `part` - 部分值
/// * `total` - 总值
/// * `decimals` - 保留小数位数，默认2位
///
/// # 返回
/// 百分比值
pub fn calculate_percentage(part: f64, total: f64, decimals: usize) -> f64 {
    if total == 0.0 {
        return 0.0;
    }
    let percentage = (part / total) * 100.0;
    round_to_decimal(percentage, decimals)
}

/// 四舍五入到指定小数位
///
/// # 参数
/// * `num` - 要四舍五入的数字
/// * `decimals` - 小数位数
///
/// # 返回
/// 四舍五入后的数字
pub fn round_to_decimal(num: f64, decimals: usize) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (num * multiplier).round() / multiplier
}

/// 计算平均值
///
/// # 参数
/// * `numbers` - 数字数组
///
/// # 返回
/// 平均值，如果数组为空则返回0.0
pub fn calculate_average(numbers: &[f64]) -> f64 {
    if numbers.is_empty() {
        return 0.0;
    }
    let sum: f64 = numbers.iter().sum();
    sum / numbers.len() as f64
}

/// 计算最大值
///
/// # 参数
/// * `numbers` - 数字数组
///
/// # 返回
/// 最大值，如果数组为空则返回错误
pub fn calculate_max(numbers: &[f64]) -> Result<f64, CalcError> {
    numbers
        .iter()
        .copied()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .ok_or(CalcError::EmptyArray)
}

/// 计算最小值
///
/// # 参数
/// * `numbers` - 数字数组
///
/// # 返回
/// 最小值，如果数组为空则返回错误
pub fn calculate_min(numbers: &[f64]) -> Result<f64, CalcError> {
    numbers
        .iter()
        .copied()
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .ok_or(CalcError::EmptyArray)
}

/// 计算总和
///
/// # 参数
/// * `numbers` - 数字数组
///
/// # 返回
/// 总和
pub fn calculate_sum(numbers: &[f64]) -> f64 {
    numbers.iter().sum()
}

/// 计算阶乘
///
/// # 参数
/// * `n` - 非负整数
///
/// # 返回
/// 阶乘结果，如果n为负数则返回错误
pub fn factorial(n: u64) -> Result<u64, CalcError> {
    if n == 0 || n == 1 {
        return Ok(1);
    }

    let mut result = 1u64;
    for i in 2..=n {
        result = result
            .checked_mul(i)
            .ok_or_else(|| CalcError::InvalidNumber(format!("阶乘结果溢出: {}", n)))?;
    }
    Ok(result)
}

/// 计算幂
///
/// # 参数
/// * `base` - 底数
/// * `exponent` - 指数
///
/// # 返回
/// 幂运算结果
pub fn power(base: f64, exponent: f64) -> f64 {
    base.powf(exponent)
}

/// 计算平方根
///
/// # 参数
/// * `num` - 数字
///
/// # 返回
/// 平方根，如果数字为负数则返回错误
pub fn sqrt(num: f64) -> Result<f64, CalcError> {
    if num < 0.0 {
        return Err(CalcError::NegativeNumber);
    }
    Ok(num.sqrt())
}

/// 计算绝对值
///
/// # 参数
/// * `num` - 数字
///
/// # 返回
/// 绝对值
pub fn abs(num: f64) -> f64 {
    num.abs()
}

/// 将数字限制在指定范围内
///
/// # 参数
/// * `num` - 数字
/// * `min` - 最小值
/// * `max` - 最大值
///
/// # 返回
/// 限制后的数字
pub fn clamp(num: f64, min: f64, max: f64) -> f64 {
    num.max(min).min(max)
}

/// 检查数字是否在指定范围内
///
/// # 参数
/// * `num` - 数字
/// * `min` - 最小值
/// * `max` - 最大值
///
/// # 返回
/// 是否在范围内
pub fn in_range(num: f64, min: f64, max: f64) -> bool {
    num >= min && num <= max
}

/// 格式化数字，添加千分位分隔符
///
/// # 参数
/// * `num` - 数字
///
/// # 返回
/// 格式化后的字符串
pub fn format_number(num: i64) -> String {
    let num_str = num.to_string();
    let mut result = String::new();

    for (i, ch) in num_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }

    result.chars().rev().collect()
}

/// 格式化浮点数，保留指定小数位并添加千分位分隔符
///
/// # 参数
/// * `num` - 数字
/// * `decimals` - 小数位数
///
/// # 返回
/// 格式化后的字符串
pub fn format_float(num: f64, decimals: usize) -> String {
    let rounded = round_to_decimal(num, decimals);
    let parts: Vec<&str> = format!("{:.decimals$}", rounded, decimals = decimals)
        .split('.')
        .collect();

    if parts.len() == 2 {
        let int_part: i64 = parts[0].parse().unwrap_or(0);
        format!("{}.{}", format_number(int_part), parts[1])
    } else {
        let int_part: i64 = rounded as i64;
        format_number(int_part)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate() {
        assert_eq!(calculate(10.0, 5.0, Operator::Add).unwrap(), 15.0);
        assert_eq!(calculate(10.0, 5.0, Operator::Subtract).unwrap(), 5.0);
        assert_eq!(calculate(10.0, 5.0, Operator::Multiply).unwrap(), 50.0);
        assert_eq!(calculate(10.0, 5.0, Operator::Divide).unwrap(), 2.0);
        assert!(calculate(10.0, 0.0, Operator::Divide).is_err());
    }

    #[test]
    fn test_calculate_percentage() {
        assert_eq!(calculate_percentage(25.0, 100.0, 2), 25.0);
        assert_eq!(calculate_percentage(1.0, 3.0, 2), 33.33);
    }

    #[test]
    fn test_round_to_decimal() {
        assert_eq!(round_to_decimal(3.14159, 2), 3.14);
        assert_eq!(round_to_decimal(3.14159, 4), 3.1416);
    }

    #[test]
    fn test_calculate_average() {
        assert_eq!(calculate_average(&[1.0, 2.0, 3.0, 4.0, 5.0]), 3.0);
        assert_eq!(calculate_average(&[]), 0.0);
    }

    #[test]
    fn test_calculate_sum() {
        assert_eq!(calculate_sum(&[1.0, 2.0, 3.0]), 6.0);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0).unwrap(), 1);
        assert_eq!(factorial(1).unwrap(), 1);
        assert_eq!(factorial(5).unwrap(), 120);
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(sqrt(16.0).unwrap(), 4.0);
        assert!(sqrt(-1.0).is_err());
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn test_in_range() {
        assert!(in_range(5.0, 0.0, 10.0));
        assert!(!in_range(15.0, 0.0, 10.0));
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }
}
