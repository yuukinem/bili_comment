// B站 API 模块

pub mod client;
pub mod login;
pub mod search;
pub mod comment;
pub mod error;

pub use client::BILI_CLIENT;
pub use error::BiliApiError;
