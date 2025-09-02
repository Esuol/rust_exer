const { platform } = process;
const path = require('path');

// 根据平台确定编译产物的路径和扩展名
function getLibExtension() {
  switch (platform) {
    case 'win32': return 'dll';
    case 'darwin': return 'dylib';
    case 'linux': return 'so';
    default: throw new Error(`Unsupported platform: ${platform}`);
  }
}

// 加载真正的NAPI模块
console.log('Loading Rust image processor...');

let nativeBinding;
try {
  // 尝试加载生成的.node文件
  const projectRoot = path.resolve(__dirname, '..');
  const nodePath = path.join(projectRoot, 'rust-image-processor.node');

  console.log('Loading from:', nodePath);

  if (!require('fs').existsSync(nodePath)) {
    throw new Error(`Node addon not found at: ${nodePath}`);
  }

  nativeBinding = require(nodePath);
  console.log('Real NAPI module loaded successfully!');
  console.log('Available functions:', Object.keys(nativeBinding));

} catch (e) {
  console.error('Failed to load real module, falling back to mock:', e.message);

  // Fallback to mock functions for testing
  nativeBinding = {
    hello: () => "Hello from mock Rust!",
    add: (a, b) => a + b,
    check_image_exists: (path) => {
      try {
        return require('fs').existsSync(path);
      } catch (e) {
        return false;
      }
    },
    get_file_size: (path) => {
      try {
        const stats = require('fs').statSync(path);
        return stats.size;
      } catch (e) {
        throw new Error(`Failed to get file size: ${e.message}`);
      }
    },
    get_image_extension: (path) => {
      const ext = require('path').extname(path);
      return ext ? ext.substring(1) : '';
    }
  };

  console.log('Mock functions loaded as fallback');
}

module.exports = nativeBinding;