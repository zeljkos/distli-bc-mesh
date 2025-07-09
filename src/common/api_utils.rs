// Shared API utilities
use serde_json::json;
use warp::Reply;

pub fn success_response(message: &str) -> impl Reply {
    warp::reply::json(&json!({
        "status": "success",
        "message": message
    }))
}

pub fn error_response(message: &str) -> impl Reply {
    warp::reply::json(&json!({
        "status": "error", 
        "message": message
    }))
}

pub fn health_check() -> impl Reply {
    warp::reply::json(&json!({
        "status": "healthy",
        "timestamp": crate::common::time::current_timestamp()
    }))
}
