use serde::{Deserialize, Serialize};
use std::num::NonZeroI32;
use std::path::Path;
use std::path::PathBuf;
use std::string::String;
use std::time::Duration;

// fn xxx() -> {

//     if let Some(config_dir) = config_file.map(Path::new).and_then(Path::parent) {
//         deserialised.allowed_failures = deserialised
//             .allowed_failures()
//             .iter()
//             .map(|p| config_dir.join(p))
//             .collect();
//     }
// }

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config<P> {
    elm_compiler: Option<String>,
    node: Option<String>,
    args: Option<Box<[String]>>,
    #[serde(bound(
        serialize = "P: AsRef<RelativePath> + Serialize",
        deserialize = "P: AsRef<RelativePath> + Deserialize<'de>"
    ))]
    allowed_failures: Option<Box<[P]>>,
    compiler_reruns: Option<NonZeroI32>,
    timeout: Option<Duration>,
}

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct RelativePath(PathBuf);

impl AsRef<Path> for RelativePath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<RelativePath> for RelativePath {
    fn as_ref(&self) -> &RelativePath {
        &self
    }
}

impl Config<RelativePath> {
    pub fn into_config<P>(self, config_file_location: P) -> Config<PathBuf>
    where
        P: AsRef<Path>,
    {
        let allowed_failures = self.allowed_failures.as_ref().map(|allowed_failures| {
            allowed_failures
                .iter()
                .map(|file_path| config_file_location.as_ref().join(file_path))
                .collect()
        });
        Config {
            elm_compiler: self.elm_compiler,
            node: self.node,
            args: self.args,
            allowed_failures,
            compiler_reruns: self.compiler_reruns,
            timeout: self.timeout,
        }
    }
}

impl<P: AsRef<Path>> Config<P> {
    pub fn serialize<P2: AsRef<Path>>(self, file_location: P2) -> impl Serialize {
        let allowed_failures = self.allowed_failures.as_ref().map(|allowed_failures| {
            allowed_failures
                .iter()
                .map(|file_path| {
                    RelativePath(
                        pathdiff::diff_paths(
                            file_path.as_ref(),
                            &file_location.as_ref().to_path_buf(),
                        )
                        .expect("Able to diff paths"),
                    )
                })
                .collect()
        });
        Config {
            elm_compiler: self.elm_compiler,
            node: self.node,
            args: self.args,
            allowed_failures,
            compiler_reruns: self.compiler_reruns,
            timeout: self.timeout,
        }
    }

    pub fn elm_compiler(&self) -> &str {
        self.elm_compiler
            .as_ref()
            .map_or_else(|| "elm", String::as_str)
    }

    pub fn node(&self) -> &str {
        self.node.as_ref().map_or_else(|| "node", String::as_str)
    }

    pub fn args(&self) -> &[String] {
        self.args.as_ref().map_or_else(|| &[][..], |x| &x[..])
    }

    pub fn allowed_failures(&self) -> &[impl AsRef<Path>] {
        self.allowed_failures
            .as_ref()
            .map_or_else(|| &[][..], |x| &x[..])
    }

    pub fn compiler_reruns(&self) -> NonZeroI32 {
        self.compiler_reruns
            .unwrap_or_else(|| NonZeroI32::new(1).unwrap())
    }

    // pub fn timeout(&self) -> Duration {
    //     self.timeout.unwrap_or_else(|| Duration::new(10, 0))
    // }
}
