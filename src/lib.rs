use serde::{
    de::{self, value, Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};
use std::{
    collections::{HashMap, HashSet},
    fmt,
};
use toml::de::Error;

/// Config represents the full configuration within a netlify.toml file.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub build: Option<Build>,
    pub context: Option<HashMap<String, Context>>,
    #[serde(
        alias = "edgeHandlers",
        alias = "edge-handlers",
        alias = "edge_handlers",
        default
    )]
    pub edge_handlers: Vec<EdgeHandler>,
    pub functions: Option<Functions>,
    pub headers: Option<Vec<Header>>,
    pub redirects: Option<Vec<Redirect>>,
    pub template: Option<Template>,
}

/// Build configuration.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Build {
    pub base: Option<String>,
    pub command: Option<String>,
    pub functions: Option<String>,
    pub environment: Option<HashMap<String, String>>,
    #[serde(alias = "edge-handlers", alias = "edgeHandlers")]
    pub edge_handlers: Option<String>,
    pub publish: Option<String>,
}

/// Context overrides the build variables Netlify uses to build a site before deploying it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub base: Option<String>,
    pub command: Option<String>,
    #[serde(alias = "edge-handlers", alias = "edgeHandlers")]
    pub edge_handlers: Option<String>,
    pub environment: Option<HashMap<String, String>>,
    pub functions: Option<ContextFunctions>,
    pub publish: Option<String>,
}

/// Netlify Functions configuration.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Functions {
    pub directory: Option<String>,
    #[serde(default)]
    pub external_node_modules: Vec<String>,
    #[serde(default)]
    pub ignored_node_modules: Vec<String>,
    #[serde(default)]
    pub included_files: Vec<String>,
    pub node_bundler: Option<Bundler>,
}

/// Context-specific Netlify Functions configuration.
///
/// Disallows specifying the `directory` property.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ContextFunctions {
    #[serde(default)]
    pub external_node_modules: Vec<String>,
    #[serde(default)]
    pub ignored_node_modules: Vec<String>,
    #[serde(default)]
    pub included_files: Vec<String>,
    pub node_bundler: Option<Bundler>,
}

/// The netlify functions builder to use.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Bundler {
    Esbuild,
    Nft,
    Zisi,
}

/// Redirect holds information about a url redirect.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Redirect {
    #[serde(alias = "origin")]
    pub from: String,
    #[serde(alias = "destination")]
    pub to: Option<String>,
    #[serde(default = "default_status")]
    pub status: u32,
    #[serde(default)]
    pub force: bool,
    pub headers: Option<HashMap<String, String>>,
    #[serde(alias = "params", alias = "parameters")]
    pub query: Option<HashMap<String, String>>,
    pub conditions: Option<HashMap<String, HashSet<String>>>,
    pub signed: Option<String>,
    #[serde(alias = "edge-handler")]
    pub edge_handler: Option<String>,
}

/// Header holds information to add response headers for a give url.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Header {
    #[serde(rename = "for")]
    pub path: String,
    #[serde(rename = "values")]
    pub headers: HashMap<String, HeaderValues>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HeaderValues {
    pub values: Vec<String>,
}

/// Template holds information to turn a repository into a Netlify template.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Template {
    #[serde(rename = "incoming-hooks")]
    pub hooks: Option<Vec<String>>,
    pub environment: Option<HashMap<String, String>>,
}

/// A mount of an edge handler under a specific path.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EdgeHandler {
    /// The name of the edge handler to run.
    pub handler: String,
    /// The mount path of the edge handler.
    ///
    /// The system will select the first path that matches from top to bottom,
    /// if multiple apply.
    #[serde(alias = "for")]
    pub path: String,
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
#[inline]
pub fn from_str(io: &str) -> Result<Config, Error> {
    toml::from_str(io)
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
        let mut result = HashMap::new();

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

// This is the trait that informs Serde how to deserialize HeaderValues.
impl<'de> Deserialize<'de> for HeaderValues {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HeaderValuesVisitor;
        impl<'de> Visitor<'de> for HeaderValuesVisitor {
            type Value = HeaderValues;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string or vector")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v.contains(',') {
                    let values = v.split(',').map(|s| String::from(s.trim())).collect();
                    return Ok(HeaderValues { values });
                }

                Ok(HeaderValues {
                    values: vec![v.to_owned()],
                })
            }

            fn visit_seq<V>(self, v: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let items = Deserialize::deserialize(value::SeqAccessDeserializer::new(v))?;
                Ok(HeaderValues { values: items })
            }
        }
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of MyMap.
        deserializer.deserialize_any(HeaderValuesVisitor {})
    }
}

// This is the trait that informs Serde how to serialize HeaderValues.
impl Serialize for HeaderValues {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.values.len() > 1 {
            let mut seq = serializer.serialize_seq(Some(self.values.len()))?;
            for e in self.values.to_owned() {
                seq.serialize_element(&e)?;
            }
            seq.end()
        } else {
            serializer.serialize_str(&self.values[0])
        }
    }
}

fn default_status() -> u32 {
    301
}

impl Default for Redirect {
    fn default() -> Redirect {
        Redirect {
            from: String::new(),
            to: None,
            status: default_status(),
            force: false,
            signed: None,
            conditions: None,
            query: None,
            headers: None,
            edge_handler: None,
        }
    }
}

impl fmt::Display for Redirect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = toml::ser::to_string_pretty(&self);
        write!(f, "{:?}", string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_equal() {
        let r = Redirect {
            from: "/foo".to_string(),
            to: Some("/bar".to_string()),
            status: 301,
            force: false,
            headers: None,
            query: None,
            conditions: None,
            signed: None,
            edge_handler: None,
        };

        let r2 = Redirect {
            from: "/foo".to_string(),
            to: Some("/bar".to_string()),
            status: 301,
            force: false,
            headers: None,
            query: None,
            conditions: None,
            signed: None,
            edge_handler: None,
        };
        assert_eq!(r, r2)
    }

    #[test]
    fn test_default_redirect() {
        let r = Redirect {
            from: "/foo".to_string(),
            to: Some("/bar".to_string()),
            ..Default::default()
        };
        assert_eq!("/foo", r.from);
        assert_eq!(Some("/bar".to_string()), r.to);
        assert_eq!(301, r.status);
    }
}
