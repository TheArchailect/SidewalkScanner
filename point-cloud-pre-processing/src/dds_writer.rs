use ddsfile::{AlphaMode, D3D10ResourceDimension, Dds, DxgiFormat, NewDxgiParams};
use half::f16;

pub fn write_position_dds(
    path: &str,
    size: usize,
    data: &[f16],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::with_capacity(data.len() * 2);
    for &half_float in data {
        let bits = half_float.to_bits();
        bytes.extend_from_slice(&bits.to_le_bytes());
    }

    let params = NewDxgiParams {
        height: size as u32,
        width: size as u32,
        depth: None,
        format: DxgiFormat::R16G16B16A16_Float,
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

pub fn write_metadata_dds(
    path: &str,
    size: usize,
    data: &[f32],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::with_capacity(data.len() * 4);
    for &float_val in data {
        bytes.extend_from_slice(&float_val.to_le_bytes());
    }

    let params = NewDxgiParams {
        height: size as u32,
        width: size as u32,
        depth: None,
        format: DxgiFormat::R32G32B32A32_Float,
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

pub fn write_heightmap_dds(
    path: &str,
    size: usize,
    data: &[f32],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::with_capacity(data.len() * 4);
    for &height in data {
        bytes.extend_from_slice(&height.to_le_bytes());
    }

    let params = NewDxgiParams {
        height: size as u32,
        width: size as u32,
        depth: None,
        format: DxgiFormat::R32_Float,
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
