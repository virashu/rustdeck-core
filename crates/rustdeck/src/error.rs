#[allow(dead_code, reason = "WIP")]
enum ErrorKind {
    NotALibrary = 193,
}

pub fn report_libloading_error(e: &libloading::Error) {
    println!("Libloading error!");

    match e {
        libloading::Error::LoadLibraryExW { source } => {
            // source.0.raw_os_error();
            let err_code_1 = std::io::Error::last_os_error().raw_os_error();

            // This freaking thing cannot just give me the number, it only has format as public...
            let f = format!("{:?}", &source);
            let err_code_2: i32 = f
                .split_once(",")
                .unwrap()
                .0
                .strip_prefix("Os { code: ")
                .unwrap()
                .parse()
                .unwrap();

            let err_code = err_code_1.unwrap_or(err_code_2);

            match err_code {
                193 => {
                    println!("Not a library");
                }
                _ => {
                    println!("{:?}", source);
                }
            }
        }
        _ => {
            println!("{:?}", e);
        }
    }
}

#[derive(Debug)]
pub struct PluginLoadingError {
    pub detail: Option<String>,
}

impl std::fmt::Display for PluginLoadingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.detail {
            Some(d) => write!(f, "Plugin loading error: {}", d),
            None => write!(f, "Plugin loading error"),
        }
    }
}

impl std::error::Error for PluginLoadingError {}
