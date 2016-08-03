//! Various functions to detect whether texture types are supported.

use CapabilitiesSource;
use version::Api;
use version::Version;

/// Returns true is one-dimensional textures are supported.
#[inline]
pub fn is_texture_1d_supported<C: ?Sized>(context: &C) -> bool where C: CapabilitiesSource {
    context.get_version() >= &Version(Api::Gl, 1, 1)
}

/// Returns true is two-dimensional textures are supported.
///
/// This is a dummy function that always returns true, as 2d textures are always supported. This
/// function is just here for completeness.
#[inline]
pub fn is_texture_2d_supported<C: ?Sized>(_: &C) -> bool where C: CapabilitiesSource {
    true
}

/// Returns true is three-dimensional textures are supported.
#[inline]
pub fn is_texture_3d_supported<C: ?Sized>(context: &C) -> bool where C: CapabilitiesSource {
    context.get_version() >= &Version(Api::Gl, 1, 2) ||
    context.get_version() >= &Version(Api::GlEs, 3, 0) ||
    context.get_extensions().gl_ext_texture3d ||        // FIXME: functions have an EXT suffix, this isn't handled by glium
    context.get_extensions().gl_oes_texture_3d        // FIXME: functions have an OES suffix, this isn't handled by glium
}

/// Returns true is one-dimensional texture arrays are supported.
#[inline]
pub fn is_texture_1d_array_supported<C: ?Sized>(context: &C) -> bool where C: CapabilitiesSource {
    context.get_version() >= &Version(Api::Gl, 3, 0) ||
    context.get_extensions().gl_ext_texture_array
}

/// Returns true is two-dimensional texture arrays are supported.
#[inline]
pub fn is_texture_2d_array_supported<C: ?Sized>(context: &C) -> bool where C: CapabilitiesSource {
    context.get_version() >= &Version(Api::Gl, 3, 0) ||
    context.get_version() >= &Version(Api::GlEs, 3, 0) ||
    context.get_extensions().gl_ext_texture_array ||
    context.get_extensions().gl_nv_texture_array        // FIXME: functions have an NV suffix, this isn't handled by glium
}

/// Returns true is two-dimensional multisample textures are supported.
#[inline]
pub fn is_texture_2d_multisample_supported<C: ?Sized>(context: &C) -> bool where C: CapabilitiesSource {
    context.get_version() >= &Version(Api::Gl, 3, 2) ||
    context.get_version() >= &Version(Api::GlEs, 3, 1) ||
    context.get_extensions().gl_arb_texture_multisample
    // TODO: no gles extension?
}

/// Returns true is two-dimensional multisample texture arrays are supported.
#[inline]
pub fn is_texture_2d_multisample_array_supported<C: ?Sized>(context: &C) -> bool
                                                    where C: CapabilitiesSource
{
    context.get_version() >= &Version(Api::Gl, 3, 2) ||
    context.get_extensions().gl_arb_texture_multisample ||
    context.get_extensions().gl_oes_texture_storage_multisample_2d_array      // FIXME: functions have an OES suffix, this isn't handled by glium
}

/// Returns true is cubemaps are supported.
#[inline]
pub fn is_cubemaps_supported<C: ?Sized>(context: &C) -> bool where C: CapabilitiesSource {
    context.get_version() >= &Version(Api::Gl, 1, 3) ||
    context.get_version() >= &Version(Api::GlEs, 2, 0) ||
    context.get_extensions().gl_ext_texture_cube_map ||
    context.get_extensions().gl_arb_texture_cube_map
}

/// Returns true is cubemap arrays are supported.
#[inline]
pub fn is_cubemap_arrays_supported<C: ?Sized>(context: &C) -> bool where C: CapabilitiesSource {
    context.get_version() >= &Version(Api::Gl, 4, 0) ||
    context.get_extensions().gl_arb_texture_cube_map_array ||
    context.get_extensions().gl_ext_texture_cube_map_array ||
    context.get_extensions().gl_oes_texture_cube_map_array
}
