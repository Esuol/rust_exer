#![deny(clippy::all)]

use napi::{Error, Result, Status};
use napi_derive::napi;
use std::fs;
use std::path::Path;

/// 获取图片的基本信息
#[napi]
pub fn get_image_info(image_path: String) -> Result<String> {
    let path = Path::new(&image_path);

    // 检查文件是否存在
    if !path.exists() {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Image file not found: {}", image_path),
        ));
    }

    // 获取文件大小
    let metadata = fs::metadata(&path).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to read file metadata: {}", e),
        )
    })?;

    let file_size = metadata.len();

    // 返回JSON格式的字符串
    let result = format!(
        r#"{{"path":"{}","size":{},"exists":true}}"#,
        image_path, file_size
    );

    Ok(result)
}

/// 简单的图片重命名功能
#[napi]
pub fn rename_image(old_path: String, new_path: String) -> Result<bool> {
    fs::rename(&old_path, &new_path).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to rename image: {}", e),
        )
    })?;

    Ok(true)
}

/// 检查图片文件扩展名
#[napi]
pub fn get_image_extension(image_path: String) -> Result<String> {
    let path = Path::new(&image_path);

    match path.extension() {
        Some(ext) => Ok(ext.to_string_lossy().to_string()),
        None => Err(Error::new(
            Status::GenericFailure,
            "No file extension found",
        )),
    }
}

/// 检查文件是否存在
#[napi]
pub fn check_image_exists(image_path: String) -> bool {
    Path::new(&image_path).exists()
}

/// 获取文件大小（字节）
#[napi]
pub fn get_file_size(image_path: String) -> Result<u64> {
    let metadata = fs::metadata(&image_path).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to read file metadata: {}", e),
        )
    })?;

    Ok(metadata.len())
}

/// 检查文件是否是图片
#[napi]
pub fn check_image_file(image_path: String) -> Result<bool> {
    let path = Path::new(&image_path);
    let extension = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_string());
    match extension {
        Some(ext) => {
            Ok(ext == "jpg" || ext == "jpeg" || ext == "png" || ext == "gif" || ext == "webp")
        }
        None => Ok(false),
    }
}

/// Hello World函数
#[napi]
pub fn hello() -> String {
    "Hello from Rust!".to_string()
}

/// 简单的加法函数
#[napi]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
