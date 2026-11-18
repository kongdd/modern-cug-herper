use crate::{copy_tree, home, unpack_zip};
use std::io;
use std::path::Path;

pub fn install(path: &Path) -> io::Result<String> {
    let extracted = unpack_zip(path)?;
    let extensions = extracted.path().join("kong-pi-extension-master/extensions");
    copy_tree(&extensions, &home()?.join(".pi/agent/extensions"), None)?;
    Ok("已安装".into())
}
