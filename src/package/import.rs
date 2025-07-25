use crate::package::types::convert_addr;

pub fn import_package(path: &str) {
    

    // 1. 解析地址
    let addr = convert_addr(path);

    // 2.更新到本地目路
    // 本地路径： ${HOME}/ds-build/

    // 3.获得sys pakage 的信息。

    // 4. 导入到 工作目录

    // 5. 提供系统包的信息， 包组所有组件。

}

#[cfg(test)]
mod test {
    use super::import_package;

    #[test]
    fn import_pkg() {
        let path = "${HOME}/ds-build/mac-devkit-0.1.5.tar.gz";
        import_package(path);
        //sys_setting();
    }
}
