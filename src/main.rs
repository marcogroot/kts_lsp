use std::collections::HashMap;

fn main() {
    println!("Hello, world!");
}

fn encode_message(content: &str) -> String {
    let content_length_message = format!("Content-Length: {}", content.len());

    let mut message = String::new();
    message.push_str(&content_length_message);
    message.push_str("\r\n\r\n");
    message.push_str(content);

    message
}

#[derive(Debug, PartialEq)]
struct LspMessage {
    json_rpc: String,
    id: String,
    method: String,
    params: Vec<String>,
}

#[derive(Debug)]
struct JsonMessage {
    headers: HashMap<String, String>,
    body: LspMessage,
}

fn decode_message(content: Vec<u8>) -> JsonMessage {
    let split_index = content
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("could not find split");

    let header_section = &content[..split_index];
    let header_map: HashMap<String, String> = String::from_utf8_lossy(header_section)
        .splitn(2, "\r\n")
        .map(|val| {
            let a: Vec<&str> = val.split(":").collect();
            assert!(a.len() == 2);
            let res = (
                String::from(a.first().unwrap().trim()),
                String::from(a.last().unwrap().trim()),
            );
            res
        })
        .collect();

    let body = &content[split_index + 4..];
    let body_string = String::from_utf8_lossy(body).into_owned();

    JsonMessage {
        headers: header_map,
        body: body_string,
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;

    #[test]
    fn test_encode_and_decode_message() {
        let content = "
        {
            \"jsonrpc\": \"2.0\",
            \"id\": 1,
            \"method\": \"textDocument/completion\",
            \"params\": {}
        }";

        let lsp_message = LspMessage {
            json_rpc: "2.0".to_string(),
            id: "1".to_string(),
            method: "textDocument/completion".to_string(),
            params: vec![],
        };

        let encoded_message = encode_message(content);
        let expected = "Content-Length: 5\r\n\r\ntest!";

        assert_eq!(encoded_message, expected);

        let bytes = encoded_message.into_bytes();
        let decoded_message = decode_message(bytes);

        assert_eq!(decoded_message.body, lsp_message);
        assert_eq!(decoded_message.headers.get("Content-Length").unwrap(), "5");
    }
}
