extern "C" {
    pub fn http_call(call_data_offset: *const u8, call_data_length: i32);
    pub fn call_result_size() -> u32;
    pub fn call_result_copy(result_offset: *const u8);
    pub fn gas_used() -> i64;
}