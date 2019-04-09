use util::status::StatusCode;
use util::types::Value;

pub enum Response {
    Status(StatusCode),
    SingleValue { status: StatusCode, value: Value },
    MultiKV {},
}
