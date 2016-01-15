use program::raw::RawProgram;


const MASK_HAS_TESS_EVAL: u8 = 0b00000001;
const MASK_HAS_TESS_CONTROL: u8 = 0b00000010;
const MASK_HAS_GEOMETRY: u8 = 0b00000100;

/// Glium attaches internal information to a binary to be able to fully restore the program.
pub fn attach_glium_header(raw: &RawProgram, data: &mut Vec<u8>) {
    let mut header_byte = 0u8;
    if raw.has_tessellation_evaluation_shader() {
        header_byte = header_byte ^ MASK_HAS_TESS_EVAL;
    }
    if raw.has_tessellation_control_shader() {
        header_byte = header_byte ^ MASK_HAS_TESS_CONTROL;
    }
    if raw.has_geometry_shader() {
        header_byte = header_byte ^ MASK_HAS_GEOMETRY;
    }
    // TODO kind of inefficient.
    data.reserve(1);
    data.insert(0, header_byte);
}

/// Reads the first byte of the data (=glium header) and returns the corresponding shader flags.
/// If the header is not valid, returns None.
pub fn process_glium_header(data: &[u8]) -> Option<(bool, bool, bool)> {
    let header_byte = data[0];
    if header_byte >> 3 == 0 {
        let has_geometry_shader =                (header_byte & MASK_HAS_GEOMETRY) != 0;
        let has_tessellation_control_shader =    (header_byte & MASK_HAS_TESS_CONTROL) != 0;
        let has_tessellation_evaluation_shader = (header_byte & MASK_HAS_TESS_EVAL) != 0;
        Some((has_geometry_shader, has_tessellation_control_shader, has_tessellation_evaluation_shader))
    } else {
        None
    }
}
