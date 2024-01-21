use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::Write;

use inquire::validator::Validation;
use inquire::{Confirm, MultiSelect, Text};
use lazy_static::lazy_static;
use string_builder::Builder;

use crate::CfxResult;

lazy_static! {
    static ref LIBRARIES: HashMap<&'static str, Library> = [
        (
            "es_extended",
            Library::new("@es_extended/imports.lua", ScriptRuntime::Shared)
        ),
        (
            "ox_lib",
            Library::new("@ox_lib/init.lua", ScriptRuntime::Shared)
        ),
        (
            "oxmysql",
            Library::new("@oxmysql/lib/MySQL.lua", ScriptRuntime::Server)
        )
    ]
    .iter()
    .cloned()
    .collect();
}

#[derive(Debug, Clone, PartialEq)]
enum ScriptRuntime {
    Server,
    Client,
    Shared,
}

#[derive(Debug, Clone)]
struct Library {
    import: String,
    runtime: ScriptRuntime,
}

impl Library {
    pub fn new(import: impl Into<String>, runtime: ScriptRuntime) -> Self {
        Self {
            import: import.into(),
            runtime,
        }
    }
}

struct ScriptSectionBuilder {
    name: String,
    scripts: Vec<String>,
}

impl ScriptSectionBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            scripts: vec![],
        }
    }

    pub fn append(&mut self, path: impl Into<String>) -> &mut Self {
        self.scripts.push(path.into());
        self
    }

    pub fn build(&self) -> CfxResult<String> {
        let mut string_builder = Builder::default();
        string_builder.append(format!("{}_scripts {{\n", self.name));

        for (i, script) in self.scripts.iter().enumerate() {
            string_builder.append(format!("    \"{script}\""));
            if i < self.scripts.len() - 1 {
                string_builder.append(",");
            }

            string_builder.append("\n")
        }

        string_builder.append("}");
        Ok(string_builder.string()?)
    }
}

struct ScriptManifest {
    author: String,
    use_data_files: bool,
    libraries: Vec<Library>,
}

impl ScriptManifest {
    pub fn new(author: impl Into<String>, use_data_files: bool, libraries: Vec<Library>) -> Self {
        Self {
            author: author.into(),
            use_data_files,
            libraries,
        }
    }

    pub fn build(&self) -> CfxResult<String> {
        let server_scripts = self.build_script_section("server", ScriptRuntime::Server)?;
        let client_scripts = self.build_script_section("client", ScriptRuntime::Client)?;
        let shared_scripts = self.build_script_section("shared", ScriptRuntime::Shared)?;

        let mut builder = Builder::default();
        builder.append(format!(
            r#"fx_version "cerulean"
game "gta5"
lua54 "yes"

author "{}"
version "0.0.0"

{server_scripts}

{client_scripts}

{shared_scripts}
        "#,
            self.author
        ));

        if self.use_data_files {
            builder.append(
                r#"
data_files {
    "data/*.lua"
}
            "#,
            )
        }

        let result = builder.string()?.trim().to_owned();
        Ok(result)
    }

    fn build_script_section(&self, name: &str, runtime: ScriptRuntime) -> CfxResult<String> {
        let mut builder = ScriptSectionBuilder::new(name);
        for library in self.get_runtime_libraries(&runtime) {
            builder.append(&library.import);
        }

        match &runtime {
            ScriptRuntime::Server => {
                builder.append("src/server/main.lua");
            }
            ScriptRuntime::Client => {
                builder.append("src/client/main.lua");
            }
            _ => {}
        }

        Ok(builder.build()?)
    }

    fn get_runtime_libraries(&self, runtime: &ScriptRuntime) -> Vec<&Library> {
        self.libraries
            .iter()
            .filter(|x| x.runtime == *runtime)
            .collect::<Vec<&Library>>()
    }
}

pub fn handle_create_command() -> CfxResult<()> {
    let library_names = LIBRARIES.keys().cloned().collect::<Vec<&str>>();

    let min_length_validator = |input: &str| {
        if input.chars().count() < 1 {
            Ok(Validation::Invalid("Invalid input".into()))
        } else {
            Ok(Validation::Valid)
        }
    };

    let project_name = Text::new("What is your project name?")
        .with_validator(min_length_validator)
        .prompt()?;

    let author_name = Text::new("What is the authors name?")
        .with_validator(min_length_validator)
        .prompt()?;

    let use_data_files = Confirm::new("Do you want to use data files?")
        .with_default(false)
        .prompt()?;

    let libraries: Vec<_> = MultiSelect::new(
        "What libraries/frameworks do you want to use?",
        library_names,
    )
    .prompt()?
    .iter()
    .map(|&name| LIBRARIES.get(name).expect("Invalid library").clone())
    .collect::<Vec<Library>>();

    let manifest = ScriptManifest::new(&author_name, use_data_files, libraries);
    let manifest_str = manifest.build()?;

    let base_path = format!("{project_name}");
    if use_data_files {
        create_dir_all(format!("{base_path}/data"))?;
    }

    create_dir_all(format!("{base_path}/src/client"))?;
    create_dir_all(format!("{base_path}/src/server"))?;
    create_dir_all(format!("{base_path}/src/shared"))?;

    File::create(format!("{base_path}/src/client/main.lua"))?;
    File::create(format!("{base_path}/src/server/main.lua"))?;

    let mut manifest_file = File::create(format!("{base_path}/fxmanifest.lua"))?;
    manifest_file.write_all(manifest_str.as_bytes())?;

    Ok(())
}
