/// an error returned from the snapcast server
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SnapcastError {
  #[error("Parse error: {0}")]
  ParseError(String),
  #[error("Invalid request: {0}")]
  InvalidRequest(String),
  #[error("Method not found: {0}")]
  MethodNotFound(String),
  #[error("Invalid params: {0}")]
  InvalidParams(String),
  #[error("Internal error: {0}")]
  InternalError(String),
  #[error("Unknown error: {0}")]
  Unknown(i64, String),
}

impl SnapcastError {
  pub fn code(&self) -> i64 {
    match self {
      SnapcastError::ParseError(_) => -32700,
      SnapcastError::InvalidRequest(_) => -32600,
      SnapcastError::MethodNotFound(_) => -32601,
      SnapcastError::InvalidParams(_) => -32602,
      SnapcastError::InternalError(_) => -32603,
      SnapcastError::Unknown(code, _) => *code,
    }
  }

  pub fn message(&self) -> &str {
    match self {
      SnapcastError::ParseError(message) => message,
      SnapcastError::InvalidRequest(message) => message,
      SnapcastError::MethodNotFound(message) => message,
      SnapcastError::InvalidParams(message) => message,
      SnapcastError::InternalError(message) => message,
      SnapcastError::Unknown(_, message) => message,
    }
  }
}

impl<'de> serde::Deserialize<'de> for SnapcastError {
  fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
    use serde::de::Error;
    use serde_json::Value;

    let value = Value::deserialize(d)?;
    let code = if let Some(code) = value.get("code").and_then(Value::as_i64) {
      code
    } else {
      return Err(D::Error::missing_field("code"));
    };
    let message = if let Some(message) = value.get("message").and_then(Value::as_str) {
      message.to_string()
    } else {
      return Err(D::Error::missing_field("message"));
    };

    Ok(match code {
      -32700 => SnapcastError::ParseError(message),
      -32600 => SnapcastError::InvalidRequest(message),
      -32601 => SnapcastError::MethodNotFound(message),
      -32602 => SnapcastError::InvalidParams(message),
      -32603 => SnapcastError::InternalError(message),
      code => SnapcastError::Unknown(code, message),
    })
  }
}

impl serde::Serialize for SnapcastError {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    use serde::ser::SerializeStruct;

    let mut state = serializer.serialize_struct("SnapcastError", 2)?;
    state.serialize_field("code", &self.code())?;
    state.serialize_field("message", &self.message())?;
    state.end()
  }
}

/// an error controlling a stream
#[derive(Debug)]
pub enum StreamControlError {
  NotFound(String),
  CannotBeControlled(String),
  CannotNext(String),
  CannotPrevious(String),
  CannotPlay(String),
  CannotPause(String),
  CannotSeek(String),
  CannotControl(String),
  InvalidParams(String),
  Unknown(i64, String),
}

impl StreamControlError {
  pub fn code(&self) -> i64 {
    match self {
      StreamControlError::NotFound(_) => -32603,
      StreamControlError::CannotBeControlled(_) => 1,
      StreamControlError::CannotNext(_) => 2,
      StreamControlError::CannotPrevious(_) => 3,
      StreamControlError::CannotPlay(_) => 4,
      StreamControlError::CannotPause(_) => 5,
      StreamControlError::CannotSeek(_) => 6,
      StreamControlError::CannotControl(_) => 7,
      StreamControlError::InvalidParams(_) => -32602,
      StreamControlError::Unknown(code, _) => *code,
    }
  }

  pub fn message(&self) -> &str {
    match self {
      StreamControlError::NotFound(message) => message,
      StreamControlError::CannotBeControlled(message) => message,
      StreamControlError::CannotNext(message) => message,
      StreamControlError::CannotPrevious(message) => message,
      StreamControlError::CannotPlay(message) => message,
      StreamControlError::CannotPause(message) => message,
      StreamControlError::CannotSeek(message) => message,
      StreamControlError::CannotControl(message) => message,
      StreamControlError::InvalidParams(message) => message,
      StreamControlError::Unknown(_, message) => message,
    }
  }
}

impl<'de> serde::Deserialize<'de> for StreamControlError {
  fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
    use serde::de::Error;
    use serde_json::Value;

    let value = Value::deserialize(d)?;
    let code = if let Some(code) = value.get("code").and_then(Value::as_i64) {
      code
    } else {
      return Err(D::Error::missing_field("code"));
    };
    let message = if let Some(message) = value.get("message").and_then(Value::as_str) {
      message.to_string()
    } else {
      return Err(D::Error::missing_field("message"));
    };

    Ok(match code {
      -32603 => StreamControlError::NotFound(message),
      1 => StreamControlError::CannotBeControlled(message),
      2 => StreamControlError::CannotNext(message),
      3 => StreamControlError::CannotPrevious(message),
      4 => StreamControlError::CannotPlay(message),
      5 => StreamControlError::CannotPause(message),
      6 => StreamControlError::CannotSeek(message),
      7 => StreamControlError::CannotControl(message),
      -32602 => StreamControlError::InvalidParams(message),
      code => StreamControlError::Unknown(code, message),
    })
  }
}

impl serde::Serialize for StreamControlError {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    use serde::ser::SerializeStruct;

    let mut state = serializer.serialize_struct("StreamControlError", 2)?;
    state.serialize_field("code", &self.code())?;
    state.serialize_field("message", &self.message())?;
    state.end()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn serialize_errors() {
    let error = SnapcastError::ParseError("Parse error".to_string());

    let json = serde_json::to_string(&error).unwrap();
    let expected = r#"{"code":-32700,"message":"Parse error"}"#;

    assert_eq!(json, expected);
  }

  #[test]
  fn deserialize_errors() {
    let json = r#"{"code": -32700, "message": "Parse error"}"#;
    let error: SnapcastError = serde_json::from_str(json).unwrap();
    assert_eq!(error, SnapcastError::ParseError("Parse error".to_string()));

    let json = r#"{"code": -32600, "message": "Invalid request"}"#;
    let error: SnapcastError = serde_json::from_str(json).unwrap();
    assert_eq!(error, SnapcastError::InvalidRequest("Invalid request".to_string()));

    let json = r#"{"code": -32601, "message": "Method not found"}"#;
    let error: SnapcastError = serde_json::from_str(json).unwrap();
    assert_eq!(error, SnapcastError::MethodNotFound("Method not found".to_string()));

    let json = r#"{"code": -32602, "message": "Invalid params"}"#;
    let error: SnapcastError = serde_json::from_str(json).unwrap();
    assert_eq!(error, SnapcastError::InvalidParams("Invalid params".to_string()));

    let json = r#"{"code": -32603, "message": "Internal error"}"#;
    let error: SnapcastError = serde_json::from_str(json).unwrap();
    assert_eq!(error, SnapcastError::InternalError("Internal error".to_string()));
  }
}
