#![cfg_attr(all(not(test), target_arch = "wasm32"), no_std)]

pub const ABI_NAME: &str = "rouwdi.compiler-payload.mir-handoff";
pub const ABI_VERSION: u32 = 1;
pub const MIR_HANDOFF_STAGE: u32 = 1;
pub const ERROR_REAL_MIR_PAYLOAD_NOT_EXECUTABLE_YET: i32 = -1001;

pub const ABI_VERSION_SYMBOL: &str = "rouwdi_compiler_payload_abi_v1_version";
pub const ABI_STAGE_SYMBOL: &str = "rouwdi_compiler_payload_abi_v1_stage";
pub const ABI_DESCRIPTOR_PTR_SYMBOL: &str = "rouwdi_compiler_payload_abi_v1_descriptor_ptr";
pub const ABI_DESCRIPTOR_LEN_SYMBOL: &str = "rouwdi_compiler_payload_abi_v1_descriptor_len";
pub const MIR_EXECUTE_SYMBOL: &str = "rouwdi_mir_handoff_payload_v1_execute";
pub const MIR_LAST_ERROR_PTR_SYMBOL: &str = "rouwdi_mir_handoff_payload_v1_last_error_ptr";
pub const MIR_LAST_ERROR_LEN_SYMBOL: &str = "rouwdi_mir_handoff_payload_v1_last_error_len";

const ABI_DESCRIPTOR_JSON: &[u8] = br#"{"abi":"rouwdi.compiler-payload.mir-handoff","version":1,"stage":"mir_handoff","route":"wasm32-wasip1-module","status":"shim-only-bridge-attempted-blocked","bridge_blocker":"bootstrap_target_pack_missing_for_wasm_payload"}"#;
const LAST_ERROR: &[u8] = b"real MIR payload not executable yet: wasm ABI shim is present, rustc-private bridge attempt is blocked by missing wasm32-wasip1 bootstrap target pack";

#[no_mangle]
pub extern "C" fn rouwdi_compiler_payload_abi_v1_version() -> u32 {
    ABI_VERSION
}

#[no_mangle]
pub extern "C" fn rouwdi_compiler_payload_abi_v1_stage() -> u32 {
    MIR_HANDOFF_STAGE
}

#[no_mangle]
pub extern "C" fn rouwdi_compiler_payload_abi_v1_descriptor_ptr() -> usize {
    ABI_DESCRIPTOR_JSON.as_ptr() as usize
}

#[no_mangle]
pub extern "C" fn rouwdi_compiler_payload_abi_v1_descriptor_len() -> usize {
    ABI_DESCRIPTOR_JSON.len()
}

#[no_mangle]
pub extern "C" fn rouwdi_mir_handoff_payload_v1_execute(
    _input_ptr: usize,
    _input_len: usize,
    _output_ptr_out: usize,
    _output_len_out: usize,
    _error_ptr_out: usize,
    _error_len_out: usize,
) -> i32 {
    ERROR_REAL_MIR_PAYLOAD_NOT_EXECUTABLE_YET
}

#[no_mangle]
pub extern "C" fn rouwdi_mir_handoff_payload_v1_last_error_ptr() -> usize {
    LAST_ERROR.as_ptr() as usize
}

#[no_mangle]
pub extern "C" fn rouwdi_mir_handoff_payload_v1_last_error_len() -> usize {
    LAST_ERROR.len()
}

#[cfg(all(not(test), target_arch = "wasm32"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abi_shim_reports_the_explicit_mir_handoff_boundary() {
        assert_eq!(rouwdi_compiler_payload_abi_v1_version(), ABI_VERSION);
        assert_eq!(rouwdi_compiler_payload_abi_v1_stage(), MIR_HANDOFF_STAGE);
        assert_eq!(
            rouwdi_mir_handoff_payload_v1_execute(0, 0, 0, 0, 0, 0),
            ERROR_REAL_MIR_PAYLOAD_NOT_EXECUTABLE_YET
        );
        assert!(core::str::from_utf8(ABI_DESCRIPTOR_JSON)
            .unwrap()
            .contains("bridge-attempted-blocked"));
        assert!(core::str::from_utf8(LAST_ERROR)
            .unwrap()
            .contains("bootstrap target pack"));
    }
}
