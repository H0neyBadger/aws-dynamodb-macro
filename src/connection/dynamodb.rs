use aws_sdk_dynamodb::{error::QueryError, types::SdkError, Error as dynamodbError};

use crate::error::BackendError;

impl From<dynamodbError> for BackendError {
    fn from(error: dynamodbError) -> Self {
        BackendError::InternalServerError(format!("{}", error))
    }
}

impl From<SdkError<QueryError>> for BackendError {
    fn from(error: SdkError<QueryError>) -> Self {
        dbg!(&error);
        BackendError::InternalServerError(format!("{}", error))
    }
}
