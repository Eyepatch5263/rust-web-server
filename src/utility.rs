use crate::User;

// get_id functions
pub fn get_user_id(req: &str) -> &str {
    req.split("/").nth(2).unwrap_or_default().split_whitespace().next().unwrap_or_default()
}

// deserialize user data from request body
pub fn get_user_request_body(req: &str) -> Result<User, serde_json::Error> {
    serde_json::from_str(req.split("\r\n\r\n").last().unwrap_or_default())
}
