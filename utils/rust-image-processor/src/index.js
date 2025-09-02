const { platform, arch } = require("process");
const { path } = require("path");

// 根据平台确定编译产物的路径
let nativeBinding;

try {
  // 开发环境直接加载target目录下的文件
  const libName = `rust_image_processor.${getLibExtension()}`;
  const libPath = path.join(__dirname, 'target', 'debug', libName);
  nativeBinding = require(libPath);
} catch (error) {
  // 生产环境从包路径加载
  throw new Error(`Failed to load native module: ${e.message}`);
}

function getLibExtension() {
  switch (platform) {
    case 'win32': return 'dll';
    case 'darwin': return 'dylib';
    case 'linux': return 'so';
    default: throw new Error(`Unsupported platform: ${platform}`);
  }
}

module.exports = nativeBinding;