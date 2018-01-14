#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::collections::HashMap;

/// Config represents the full configuration within a netlify.toml file.
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub build: Option<Context>,
    pub context: Option<HashMap<String, Context>>,
    pub redirects: Option<Vec<Redirect>>,
    pub headers: Option<Vec<Header>>,
    pub template: Option<Template>,
}

/// Context holds the build variables Netlify uses to build a site before deploying it.
#[derive(Serialize, Deserialize)]
pub struct Context {
    pub base: Option<String>,
    pub publish: Option<String>,
    pub command: Option<String>,
    pub functions: Option<String>,
    pub environment: Option<HashMap<String, String>>,
}

/// Redirect holds information about a url redirect.
#[derive(Serialize, Deserialize)]
pub struct Redirect {
    pub from: String,
    pub to: String,
    pub signed: Option<String>,
    pub status: Option<i64>,
    pub force: Option<bool>,
    pub headers: Option<HashMap<String, String>>,
}

/// Header olds information to add response headers for a give url.
#[derive(Serialize, Deserialize)]
pub struct Header {
    #[serde(rename="for")]
    pub path: String,
    #[serde(rename="values")]
    pub headers: HashMap<String, String>,
}

/// Template holds information to turn a repository into a Netlify template.
#[derive(Serialize, Deserialize)]
pub struct Template {
    #[serde(rename="incoming-hooks")]
    pub hooks: Option<Vec<String>>,
    pub environment: Option<HashMap<String, String>>,
}

/// Parses the contents of a netlify.toml file as a Config structure.
///
/// # Arguments
///
/// `io` - A string slice that holds the content of a toml file.
///
/// # Example
///
/// ```
/// let io = r#"
/// [build]
///   command = "make site"
/// "#;
///
/// let result = netlify_toml::from_str(io);
/// ```
pub fn from_str(io: &str) -> Result<Config, toml::de::Error> {
    toml::from_str::<Config>(io)
}

impl Config {
    /// Returns a HashMap that aggregates all environment variables for
    /// a context within a git branch.
    ///
    /// # Arguments
    ///
    /// `ctx` - The context name, for example `deploy-preview`, `branch-deploy` or `production`.
    /// `branch` - The deploy branch name, for example `make-changes-to-my-site`.
    ///
    /// # Example
    ///
    /// ```
    /// let io = r#"
    /// [build]
    ///   command = "make site"
    /// "#;
    ///
    /// let config = netlify_toml::from_str(io).unwrap();
    /// let env = config.context_env("deploy-preview", "new-styles");
    /// ```
    pub fn context_env(self, ctx: &str, branch: &str) -> HashMap<String, String> {
        let mut result = HashMap::<String, String>::new();

        // Read the env variables from the global "build" context.
        if let Some(c) = self.build {
            if let Some(ref env) = c.environment {
                for (k, v) in env {
                    result.insert(k.to_string(), v.to_string());
                }
            }
        }

        if let Some(c) = self.context {
            // Override with default context environment,
            // like `deploy-preview`, `branch-deploy` or `production`.
            if let Some(x) = c.get(ctx) {
                if let Some(ref env) = x.environment {
                    for (k, v) in env {
                        result.insert(k.to_string(), v.to_string());
                    }
                }
            }

            // Override with branch context environment,
            // like `deploy-preview`, `branch-deploy` or `production`.
            if let Some(x) = c.get(branch) {
                if let Some(ref env) = x.environment {
                    for (k, v) in env {
                        result.insert(k.to_string(), v.to_string());
                    }
                }
            }
        }

        result
    }
}
