try {
  const rustAddon = require('./index.js');

  console.log('🎉 Testing Rust Node.js extension...');
  console.log('Available functions:', Object.keys(rustAddon));

  // 测试基本的Rust函数
  try {
    const helloResult = rustAddon.hello();
    console.log('hello():', helloResult);

    const addResult = rustAddon.add(3, 5);
    console.log('add(3, 5):', addResult);
  } catch (error) {
    console.log('Basic function error:', error.message);
  }

  // 测试文件相关函数
  const testFile = './src/test.js';
  console.log(`File exists (${testFile}):`, rustAddon.check_image_exists(testFile));

  // 测试获取文件大小
  try {
    const fileSize = rustAddon.get_file_size(testFile);
    console.log(`File size (${testFile}):`, fileSize, 'bytes');
  } catch (error) {
    console.log('File size error:', error.message);
  }

  // 测试获取文件扩展名
  try {
    const ext = rustAddon.get_image_extension('test.jpg');
    console.log('File extension:', ext);
  } catch (error) {
    console.log('Extension error:', error.message);
  }

  console.log('✅ Test completed successfully!');
} catch (error) {
  console.error('❌ Test failed:', error.message);
  console.error('Stack:', error.stack);
}