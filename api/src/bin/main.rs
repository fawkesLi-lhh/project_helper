use anyhow::{Context, Result};
use api::dot_parse::write_to_file;

#[auto_context::auto_context]
fn main() -> Result<()> {
    let cg = api::dot_parse::parse_from_dot("data/master.dot")?;
    println!("{:?}", cg);
    write_to_file(&serde_json::to_string(&cg)?, "data/master.json")?;
    Ok(())
}
