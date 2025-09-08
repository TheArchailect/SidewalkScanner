/// DDS texture file writer with unified 32-bit float formats
use ddsfile::{AlphaMode, D3D10ResourceDimension, Dds, DxgiFormat, NewDxgiParams};

pub fn write_f32_texture(
    path: &str,
    size: usize,
    data: &[f32],
    format: DxgiFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::with_capacity(data.len() * 4);
    for &val in data {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    let params = NewDxgiParams {
        height: size as u32,
        width: size as u32,
        depth: None,
        format,
        mipmap_levels: Some(1),
        array_layers: Some(1),
        caps2: None,
        is_cubemap: false,
        resource_dimension: D3D10ResourceDimension::Texture2D,
        alpha_mode: AlphaMode::Unknown,
    };
    let mut dds = Dds::new_dxgi(params)?;
    dds.data = bytes;
    dds.write(&mut std::fs::File::create(path)?)?;
    Ok(())
}
