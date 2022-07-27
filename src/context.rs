use template::{self, Template};
use compiler;
use {Result, Error};

use std::fmt;
use std::fs::File;
use std::str;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Represents the shared metadata needed to compile and render a mustache
/// template.
#[derive(Debug, Clone)]
pub struct Context<P: PartialLoader> {
    pub partial_loader: P,
}

impl Context<DefaultLoader> {
    /// Configures a mustache context the specified path to the templates.
    pub fn new(path: PathBuf) -> Self {
        Context {
            // template_path: path.clone(),
            // template_extension: "mustache".to_string(),
            partial_loader: DefaultLoader::new(path, "mustache".to_string()),
        }
    }

    /// Configures a mustache context the specified path and extension to the templates.
    pub fn with_extension(path: PathBuf, extension: String) -> Self {
        Context {
            partial_loader: DefaultLoader::new(path, extension),
        }
    }
}

impl<P: PartialLoader> Context<P> {
    /// Configures a mustache context to use a custom loader
    pub fn with_loader(loader: P) -> Self {
        Self {
            partial_loader: loader
        }
    }

    /// Compiles a template from a string
    pub fn compile<IT: Iterator<Item = char>>(&self, reader: IT) -> Result<Template<P>> {
        let compiler = compiler::Compiler::new(self.clone(), reader);
        let (tokens, partials) = compiler.compile()?;

        Ok(template::new(self.clone(), tokens, partials))
    }

    /// Compiles a template from a path.
    pub fn compile_path(&self, path: impl AsRef<Path>) -> Result<Template<P>> {
        let template = self.partial_loader.load(path)?;

        self.compile(template.chars())
    }
}

/// A trait that defines how partials should be loaded.
/// Types implementing this trait must also implement [`Clone`],
/// and must provide the [`PartialLoader::load`] method.
///
/// Its default implementation, [`DefaultLoader`], simply loads the corresponding file from the disk.
///
/// # Example
///
/// ```
/// use mustache::{PartialLoader, Error};
/// use std::path::Path;
///
/// // A simple loader, that returns the name of the partial as the partial's body
/// #[derive(Clone, Debug)]
/// pub struct MyLoader {}
///
/// impl PartialLoader for MyLoader {
///     fn load(&self, name: impl AsRef<Path>) -> Result<String, Error> {
///         let name = name.as_ref().to_str().ok_or(Error::InvalidStr)?;
///         Ok(name.to_string())
///     }
/// }
/// ```
pub trait PartialLoader: Clone {
    fn load(&self, name: impl AsRef<Path>) -> Result<String>;
}

/// Default [`PartialLoader`].
///
/// For a given partial with `name`, loads `{template_path}/{name}.{template_extension}`.
/// Uses `set_extension` to set the extension.
#[derive(Clone, Debug, PartialEq)]
pub struct DefaultLoader {
    pub template_path: PathBuf,
    pub template_extension: String,
}

impl DefaultLoader {
    pub fn new(
        template_path: PathBuf,
        template_extension: String
    ) -> Self {
        Self {
            template_path,
            template_extension,
        }
    }
}

impl PartialLoader for DefaultLoader {
    fn load(&self, name: impl AsRef<Path>) -> Result<String> {
        let mut path = self.template_path.join(name.as_ref());
        path.set_extension(&self.template_extension);

        // FIXME(#6164): This should use the file decoding tools when they are
        // written. For now we'll just read the file and treat it as UTF-8file.

        match File::open(path) {
            Ok(mut file) => {
                let mut string = String::new();
                file.read_to_string(&mut string)?;

                Ok(string)
            }

            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(String::new())
            },
            Err(e) => return Err(e.into()),
        }
    }
}
