use engine::err;
use engine::util::Value;

pub enum Response<'a> {
    Status(err::StatusCode),
    SingleValue {
        status: err::StatusCode,
        value: Value
    },
    MultiKV {

    }
}

