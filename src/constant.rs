// constants for HTTP responses
pub const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!";
pub const NOT_FOUND_RESPONSE: &str =
    "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\nNot Found";
pub const INTERNAL_SERVER_ERROR_RESPONSE: &str =
    "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\n\r\nInternal Server Error";