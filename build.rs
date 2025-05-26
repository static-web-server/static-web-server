use shadow_rs::{SdResult, ShadowBuilder};

fn main() -> SdResult<()> {
    ShadowBuilder::builder().build()?;
    Ok(())
}
