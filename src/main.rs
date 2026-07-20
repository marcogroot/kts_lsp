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

fn decode_message(content: Vec<u8>) -> String {
    let split_index = content
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("could not find split");

    let body = &content[&split_index + 4..];

    String::from_utf8_lossy(body).into_owned()
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;

    #[test]
    fn test_encode_and_decode_message() {
        let content = "test!";

        let encoded_message = encode_message(content);
        let expected = "Content-Length: 5\r\n\r\ntest!";

        assert_eq!(encoded_message, expected);

        let bytes = encoded_message.into_bytes();
        let decoded_message = decode_message(bytes);

        assert_eq!(content, decoded_message);
    }
}
