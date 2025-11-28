/**
 * 计算器模块
 * 提供基本的数学计算功能
 */

/**
 * 执行基本四则运算
 * @param a 第一个操作数
 * @param b 第二个操作数
 * @param operator 运算符: '+', '-', '*', '/'
 * @returns 计算结果
 */
export function calculate(
  a: number,
  b: number,
  operator: '+' | '-' | '*' | '/'
): number {
  switch (operator) {
    case '+':
      return a + b;
    case '-':
      return a - b;
    case '*':
      return a * b;
    case '/':
      if (b === 0) {
        throw new Error('除数不能为零');
      }
      return a / b;
    default:
      throw new Error(`不支持的运算符: ${operator}`);
  }
}

/**
 * 计算表达式（支持简单的数学表达式）
 * @param expression 数学表达式字符串，例如: "2 + 3 * 4"
 * @returns 计算结果
 */
export function evaluateExpression(expression: string): number {
  // 移除所有空白字符
  const cleaned = expression.replace(/\s/g, '');

  // 使用 Function 构造函数安全地计算表达式
  // 注意：在生产环境中应该使用更安全的表达式解析器
  try {
    // 验证表达式只包含数字、运算符和括号
    if (!/^[0-9+\-*/().\s]+$/.test(cleaned)) {
      throw new Error('表达式包含非法字符');
    }

    // 使用 eval 计算（仅用于简单表达式，生产环境应使用解析器）
    const result = Function(`"use strict"; return (${cleaned})`)();

    if (typeof result !== 'number' || !isFinite(result)) {
      throw new Error('计算结果无效');
    }

    return result;
  } catch (error) {
    throw new Error(
      `表达式计算失败: ${
        error instanceof Error ? error.message : String(error)
      }`
    );
  }
}

/**
 * 计算百分比
 * @param part 部分值
 * @param total 总值
 * @param decimals 保留小数位数，默认2位
 * @returns 百分比值
 */
export function calculatePercentage(
  part: number,
  total: number,
  decimals: number = 2
): number {
  if (total === 0) {
    return 0;
  }
  const percentage = (part / total) * 100;
  return roundToDecimal(percentage, decimals);
}

/**
 * 四舍五入到指定小数位
 * @param num 要四舍五入的数字
 * @param decimals 小数位数
 * @returns 四舍五入后的数字
 */
export function roundToDecimal(num: number, decimals: number): number {
  const multiplier = Math.pow(10, decimals);
  return Math.round(num * multiplier) / multiplier;
}

/**
 * 计算平均值
 * @param numbers 数字数组
 * @returns 平均值
 */
export function calculateAverage(numbers: number[]): number {
  if (numbers.length === 0) {
    return 0;
  }
  const sum = numbers.reduce((acc, num) => acc + num, 0);
  return sum / numbers.length;
}

/**
 * 计算最大值
 * @param numbers 数字数组
 * @returns 最大值
 */
export function calculateMax(numbers: number[]): number {
  if (numbers.length === 0) {
    throw new Error('数组不能为空');
  }
  return Math.max(...numbers);
}

/**
 * 计算最小值
 * @param numbers 数字数组
 * @returns 最小值
 */
export function calculateMin(numbers: number[]): number {
  if (numbers.length === 0) {
    throw new Error('数组不能为空');
  }
  return Math.min(...numbers);
}

/**
 * 计算总和
 * @param numbers 数字数组
 * @returns 总和
 */
export function calculateSum(numbers: number[]): number {
  return numbers.reduce((acc, num) => acc + num, 0);
}

/**
 * 计算阶乘
 * @param n 非负整数
 * @returns 阶乘结果
 */
export function factorial(n: number): number {
  if (n < 0) {
    throw new Error('阶乘只能计算非负整数');
  }
  if (n === 0 || n === 1) {
    return 1;
  }
  let result = 1;
  for (let i = 2; i <= n; i++) {
    result *= i;
  }
  return result;
}

/**
 * 计算幂
 * @param base 底数
 * @param exponent 指数
 * @returns 幂运算结果
 */
export function power(base: number, exponent: number): number {
  return Math.pow(base, exponent);
}

/**
 * 计算平方根
 * @param num 数字
 * @returns 平方根
 */
export function sqrt(num: number): number {
  if (num < 0) {
    throw new Error('不能计算负数的平方根');
  }
  return Math.sqrt(num);
}

/**
 * 计算绝对值
 * @param num 数字
 * @returns 绝对值
 */
export function abs(num: number): number {
  return Math.abs(num);
}

/**
 * 将数字限制在指定范围内
 * @param num 数字
 * @param min 最小值
 * @param max 最大值
 * @returns 限制后的数字
 */
export function clamp(num: number, min: number, max: number): number {
  return Math.min(Math.max(num, min), max);
}

/**
 * 检查数字是否在指定范围内
 * @param num 数字
 * @param min 最小值
 * @param max 最大值
 * @returns 是否在范围内
 */
export function inRange(num: number, min: number, max: number): boolean {
  return num >= min && num <= max;
}

/**
 * 格式化数字，添加千分位分隔符
 * @param num 数字
 * @returns 格式化后的字符串
 */
export function formatNumber(num: number): string {
  return num.toLocaleString('zh-CN');
}

/**
 * 格式化浮点数，保留指定小数位并添加千分位分隔符
 * @param num 数字
 * @param decimals 小数位数
 * @returns 格式化后的字符串
 */
export function formatFloat(num: number, decimals: number = 2): string {
  return num.toLocaleString('zh-CN', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
}

// 默认导出所有计算函数
export default {
  calculate,
  evaluateExpression,
  calculatePercentage,
  roundToDecimal,
  calculateAverage,
  calculateMax,
  calculateMin,
  calculateSum,
  factorial,
  power,
  sqrt,
  abs,
  clamp,
  inRange,
  formatNumber,
  formatFloat,
};
