use std::io::{Error, ErrorKind};
use std::pin::Pin;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, PartialEq)]
pub enum RespValue {
    String(String),
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    Array(Vec<RespValue>),
    Bool(bool),
    NullString,
    NullArray,
    Null,
}

impl Into<RespValue> for String {
    fn into(self) -> RespValue {
        RespValue::String(self)
    }
}

impl Into<RespValue> for &str {
    fn into(self) -> RespValue {
        RespValue::String(self.to_string())
    }
}

impl Into<RespValue> for i64 {
    fn into(self) -> RespValue {
        RespValue::Integer(self)
    }
}

impl Into<RespValue> for Vec<RespValue> {
    fn into(self) -> RespValue {
        RespValue::Array(self)
    }
}

pub struct Resp;

impl Resp {
    pub fn encode<'a, T: AsyncWriteExt + Unpin + Send + 'a>(
        value: RespValue,
        writer: &'a mut T,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>> {
        Box::pin(async move {
            match value {
                RespValue::String(s) => {
                    writer
                        .write_all(format!("${}\r\n{}\r\n", s.len(), s).as_bytes())
                        .await?;
                }
                RespValue::NullString => {
                    writer.write_all(b"$-1\r\n").await?;
                }
                RespValue::SimpleString(s) => {
                    writer.write_all(format!("+{}\r\n", s).as_bytes()).await?;
                }
                RespValue::SimpleError(s) => {
                    writer.write_all(format!("-{}\r\n", s).as_bytes()).await?;
                }
                RespValue::Integer(i) => {
                    writer.write_all(format!(":{}\r\n", i).as_bytes()).await?;
                }
                RespValue::Array(a) => {
                    writer
                        .write_all(format!("*{}\r\n", a.len()).as_bytes())
                        .await?;
                    for v in a {
                        Self::encode(v, writer).await?;
                    }
                }
                RespValue::Bool(b) => {
                    writer
                        .write_all(if b { b"#t\r\n" } else { b"#f\r\n" })
                        .await?;
                }
                RespValue::NullArray => {
                    writer.write_all(b"*-1\r\n").await?;
                }
                RespValue::Null => {
                    writer.write_all(b"_\r\n").await?;
                }
            }
            Ok(())
        })
    }

    pub fn decode<'a, T>(
        mut reader: &'a mut T,
    ) -> Pin<Box<dyn Future<Output = Result<RespValue, Error>> + Send + 'a>>
    where
        T: AsyncReadExt + Unpin + Send + 'a,
    {
        Box::pin(async move {
            let b = reader.read_u8().await?;
            match b {
                // Simple string
                b'+' | b'-' => {
                    let is_error = b == b'-';
                    let mut string = String::new();
                    let mut r_found = false;
                    loop {
                        let b = reader.read_u8().await?;
                        if r_found && b == b'\n' {
                            break;
                        }
                        if r_found {
                            string.push('\r');
                            r_found = false;
                        }
                        if b == b'\r' {
                            r_found = true;
                        } else {
                            string.push(char::from(b));
                        }
                    }
                    Ok(if is_error {
                        RespValue::SimpleError(string)
                    } else {
                        RespValue::SimpleString(string)
                    })
                }
                b':' => {
                    let mut n = 0i64;
                    let mut is_negative = false;
                    let b = reader.read_u8().await?;
                    match b {
                        b'+' => {}
                        b'-' => is_negative = true,
                        b'0'..=b'9' => {
                            n = (b - b'0') as i64;
                        }
                        _ => {
                            return Err(Error::new(
                                ErrorKind::InvalidInput,
                                "invalid character in integer",
                            ));
                        }
                    }
                    loop {
                        let b = reader.read_u8().await?;
                        match b {
                            b'0'..=b'9' => n = n * 10 + (b - b'0') as i64,
                            b'\r' => {
                                let b = reader.read_u8().await?;
                                if b != b'\n' {
                                    return Err(Error::new(
                                        ErrorKind::InvalidInput,
                                        "invalid int terminator",
                                    ));
                                }
                                break;
                            }
                            _ => {
                                return Err(Error::new(
                                    ErrorKind::InvalidInput,
                                    "invalid character in integer",
                                ));
                            }
                        }
                    }
                    Ok(RespValue::Integer(if is_negative { -n } else { n }))
                }
                // Bulk string
                b'$' => {
                    let len = Self::read_int(&mut reader).await?;
                    if len < 0 {
                        return if len == -1 {
                            Ok(RespValue::NullString)
                        } else {
                            Err(Error::new(
                                ErrorKind::InvalidInput,
                                "invalid bulk string length",
                            ))
                        };
                    }
                    let mut str_buffer = vec![0u8; len as usize];
                    reader.read_exact(&mut str_buffer).await?;
                    // consume '\n\r'
                    let mut crlf = [0u8; 2];
                    reader.read_exact(&mut crlf).await?;
                    if crlf != [b'\r', b'\n'] {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            "missing CRLF after bulk string",
                        ));
                    }
                    Ok(String::from_utf8(str_buffer)
                        .map_err(|e| Error::new(ErrorKind::InvalidInput, e))?
                        .into())
                }
                // Resp array
                b'*' => {
                    let len = Self::read_int(&mut reader).await?;
                    if len < 0 {
                        return if len == -1 {
                            Ok(RespValue::NullArray)
                        } else {
                            Err(Error::new(
                                ErrorKind::InvalidInput,
                                "invalid bulk array length",
                            ))
                        };
                    }
                    let mut list = Vec::<RespValue>::with_capacity(len as usize);
                    for _ in 0..len {
                        list.push(Self::decode(reader).await?);
                    }
                    Ok(RespValue::Array(list))
                }
                b'#' => {
                    let b = reader.read_u8().await?;
                    if b != b't' && b != b'f' {
                        return Err(Error::new(ErrorKind::InvalidInput, "invalid boolean value"));
                    }
                    // consume '\n\r'
                    let mut crlf = [0u8; 2];
                    reader.read_exact(&mut crlf).await?;
                    if crlf != [b'\r', b'\n'] {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            "missing CRLF after boolean",
                        ));
                    }
                    Ok(RespValue::Bool(b == b't'))
                }
                // Null
                b'_' => {
                    // consume '\n\r'
                    let mut crlf = [0u8; 2];
                    reader.read_exact(&mut crlf).await?;
                    if crlf != [b'\r', b'\n'] {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            "missing CRLF after null",
                        ));
                    }
                    Ok(RespValue::Null)
                }
                _ => Err(Error::new(ErrorKind::InvalidInput, "unhandled RESP type")),
            }
        })
    }

    pub async fn read_int<T: AsyncReadExt + Unpin>(mut reader: T) -> Result<i64, Error> {
        let mut n = 0i64;
        let mut negative = false;
        loop {
            let b = reader.read_u8().await?;
            match b {
                b'-' if n == 0 => negative = true,
                b'0'..=b'9' => n = n * 10 + (b - b'0') as i64,
                b'\r' => {
                    let b = reader.read_u8().await?;
                    if b != b'\n' {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            "invalid int terminator",
                        ));
                    }
                    break;
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        "invalid character in integer",
                    ));
                }
            }
        }
        Ok(if negative { -n } else { n })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    async fn decode_resp(data: &[u8]) -> Result<RespValue, Error> {
        let mut reader = Cursor::new(&data[..]);
        Resp::decode(&mut reader).await
    }

    /// Simple strings
    /// https://redis.io/docs/latest/develop/reference/protocol-spec/#simple-strings
    #[tokio::test]
    async fn test_simple_string() {
        let data = b"+OK\r\n";
        let result = decode_resp(&data[..]).await.unwrap();
        assert_eq!(result, RespValue::SimpleString("OK".to_string()));
    }

    /// Simple errors
    /// https://redis.io/docs/latest/develop/reference/protocol-spec/#simple-errors
    #[tokio::test]
    async fn test_simple_error() {
        let data = b"-Error message\r\n";
        let result = decode_resp(&data[..]).await.unwrap();
        assert_eq!(result, RespValue::SimpleError("Error message".to_string()));
    }

    /// Integers
    /// https://redis.io/docs/latest/develop/reference/protocol-spec/#integers
    #[tokio::test]
    async fn test_integers() {
        let data = b":-1000\r\n";
        let result = decode_resp(&data[..]).await.unwrap();
        assert_eq!(result, RespValue::Integer(-1000));
    }

    /// Bulk strings
    /// https://redis.io/docs/latest/develop/reference/protocol-spec/#bulk-strings
    #[tokio::test]
    async fn test_bulk_string() {
        let data = b"$5\r\nhello\r\n";
        let result = decode_resp(&data[..]).await.unwrap();
        assert_eq!(result, RespValue::String("hello".to_string()));
        let data = b"$0\r\n\r\n";
        let result = decode_resp(&data[..]).await.unwrap();
        assert_eq!(result, RespValue::String(String::new()));
    }

    /// Null bulk strings
    /// https://redis.io/docs/latest/develop/reference/protocol-spec/#null-bulk-strings
    #[tokio::test]
    async fn test_null_bulk_string() {
        let data = b"$-1\r\n";
        let result = decode_resp(&data[..]).await.unwrap();
        assert_eq!(result, RespValue::NullString);
    }

    /// Arrays
    /// https://redis.io/docs/latest/develop/reference/protocol-spec/#arrays
    #[tokio::test]
    async fn test_array() {
        let data = b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
        let result = decode_resp(&data[..]).await.unwrap();
        assert_eq!(
            result,
            RespValue::Array(vec![
                RespValue::String("hello".to_string()),
                RespValue::String("world".to_string())
            ])
        );
        let data = b"*0\r\n";
        let result = decode_resp(&data[..]).await.unwrap();
        assert_eq!(result, RespValue::Array(vec![]));
    }

    /// Null arrays
    /// https://redis.io/docs/latest/develop/reference/protocol-spec/#null-arrays
    #[tokio::test]
    async fn test_null_array() {
        let data = b"*-1\r\n";
        let result = decode_resp(&data[..]).await.unwrap();
        assert_eq!(result, RespValue::NullArray);
    }

    /// Nulls
    /// https://redis.io/docs/latest/develop/reference/protocol-spec/#nulls
    #[tokio::test]
    async fn test_null() {
        let data = b"_\r\n";
        let result = decode_resp(&data[..]).await.unwrap();
        assert_eq!(result, RespValue::Null);
    }

    /// Booleans
    /// https://redis.io/docs/latest/develop/reference/protocol-spec/#booleans
    #[tokio::test]
    async fn test_bool() {
        let data = b"#t\r\n";
        let result = decode_resp(&data[..]).await.unwrap();
        assert_eq!(result, RespValue::Bool(true));
    }
}
