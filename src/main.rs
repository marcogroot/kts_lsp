use std::collections::HashMap;

use crate::ExpectedJsonValues::EndOrStartOfKey;

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
    let _body_string = String::from_utf8_lossy(body).into_owned();
    let temp = LspMessage {
        json_rpc: "2.0".to_string(),
        id: "1".to_string(),
        method: "textDocument/completion".to_string(),
        params: vec![],
    };

    JsonMessage {
        headers: header_map,
        body: temp,
    }
}

#[derive(Debug)]
enum JsonValue {
    Nested(HashMap<String, JsonValue>),
    Text(String),
    Number(i32),
    Repeated(Vec<JsonValue>),
    Boolean(bool),
}

enum ExpectedJsonValues {
    Start,
    EndOrStartOfKey,
    Colon,
    StartOfValueOrNestedOrList,
    CommaOrEnd,
    StartOfKey,
}

fn parse_word(bytes: &[u8], start_index: usize) -> (usize, String) {
    let mut i = start_index;
    let mut escaped = false;

    loop {
        let curr = bytes[i];
        println!("Curr: {}", curr as char);
        if curr == b'"' && !escaped {
            let word = bytes[start_index..i].to_vec();
            return (i + 1, String::from_utf8(word).unwrap());
        }
        if escaped {
            escaped = false;
        }
        if curr == b'\\' {
            escaped = true;
        }
        i += 1;
    }
}

fn parse_number(bytes: &[u8], start_index: usize) -> (usize, i32) {
    let mut i = start_index;
    loop {
        let curr = bytes[i];
        let next = bytes[i + 1];
        println!("curr: {}, next: {}", curr as char, next as char);
        if !next.is_ascii_digit() {
            let num_string = String::from_utf8(bytes[start_index..i + 1].to_vec()).unwrap();
            return (i + 1, num_string.parse::<i32>().unwrap());
        }
        i += 1;
    }
}

fn parse(content: &str) -> (usize, HashMap<String, JsonValue>) {
    let mut values = HashMap::new();
    println!("content: {content}");

    let mut expected = ExpectedJsonValues::Start;
    let mut i = 0;

    let bytes = content.as_bytes();

    let mut key = String::new();
    let mut value;

    loop {
        let curr = bytes[i];
        if curr == b' ' || curr == b'\n' {
            i += 1;
            continue;
        }
        println!("Curr: {}", curr as char);
        match expected {
            ExpectedJsonValues::Start => {
                assert!(curr == b'{');
                expected = EndOrStartOfKey;
                i += 1;
            }
            ExpectedJsonValues::EndOrStartOfKey => match curr {
                b'}' => {
                    return (i + 1, values);
                }
                b'"' => {
                    let (new_i, new_key) = parse_word(bytes, i + 1);
                    key = new_key;
                    expected = ExpectedJsonValues::Colon;
                    i = new_i;
                }
                _ => {
                    panic!("got {curr}")
                }
            },
            ExpectedJsonValues::Colon => {
                assert!(curr == b':');
                expected = ExpectedJsonValues::StartOfValueOrNestedOrList;
                i += 1;
            }
            ExpectedJsonValues::StartOfValueOrNestedOrList => {
                match curr {
                    b'"' => {
                        let (new_i, new_value) = parse_word(bytes, i + 1);
                        value = new_value;
                        values.insert(key.clone(), JsonValue::Text(value.clone()));
                        //println!("Inserted {} : {}", key.clone(), value.clone());
                        expected = ExpectedJsonValues::CommaOrEnd;
                        i = new_i;
                    }
                    b'{' => {
                        println!("abc");
                        let (i_offset, nested_json) = parse(&content[i..]);
                        println!("cde");
                        values.insert(key.clone(), JsonValue::Nested(nested_json));
                        expected = ExpectedJsonValues::CommaOrEnd;
                        i = i + i_offset;
                    }
                    b'[' => {
                        // TODO: implement support for list
                        panic!("Not supported list");
                    }
                    b'f' => {
                        // TODO: Check its valid
                        values.insert(key.clone(), JsonValue::Boolean(false));
                        i += 5;
                        expected = ExpectedJsonValues::CommaOrEnd;
                    }
                    b't' => {
                        // TODO: Check its valid
                        values.insert(key.clone(), JsonValue::Boolean(true));
                        i += 4;
                        expected = ExpectedJsonValues::CommaOrEnd;
                    }
                    _ => {
                        if curr.is_ascii_digit() {
                            let (new_i, num) = parse_number(bytes, i);
                            values.insert(key.clone(), JsonValue::Number(num));
                            expected = ExpectedJsonValues::CommaOrEnd;
                            i = new_i;
                        } else {
                            panic!("Not expected {}", curr as char);
                        }
                    }
                }
            }
            ExpectedJsonValues::CommaOrEnd => match curr {
                b'}' => {
                    return (i + 1, values);
                }
                b',' => {
                    expected = ExpectedJsonValues::StartOfKey;
                    i += 1;
                }
                _ => {
                    panic!("no no no no no {}", curr as char);
                }
            },
            ExpectedJsonValues::StartOfKey => {
                assert!(curr == b'"');
                let (new_i, new_key) = parse_word(bytes, i + 1);
                key = new_key;
                expected = ExpectedJsonValues::Colon;
                i = new_i;
            }
        }
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
        let expected = "Content-Length: 145\r\n\r\n\n        {\n            \"jsonrpc\": \"2.0\",\n            \"id\": 1,\n            \"method\": \"textDocument/completion\",\n            \"params\": {}\n        }";

        assert_eq!(encoded_message, expected);

        let bytes = encoded_message.into_bytes();
        let decoded_message = decode_message(bytes);

        assert_eq!(decoded_message.body, lsp_message);
        assert_eq!(
            decoded_message.headers.get("Content-Length").unwrap(),
            "145"
        );
    }

    #[test]
    fn test_json_parsing() {
        let content = "
                {
                    \"jsonrpc\": \"2.0\",
                    \"id\": 1,
                    \"method\": \"textDocument/completion\",
                    \"is_real\": true,
                    \"is_fake\": false,
                    \"params\": {}
        }";

        let res = parse(content);
        res.1.iter().for_each(|e| println!("{}: {:?}", e.0, e.1));

        assert_eq!(1, 2);
    }
}
