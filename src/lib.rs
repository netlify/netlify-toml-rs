extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use serde::de;
use serde::de::{value, Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Config represents the full configuration within a netlify.toml file.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Config {
    pub build: Option<Context>,
    pub context: Option<HashMap<String, Context>>,
    pub redirects: Option<Vec<Redirect>>,
    pub headers: Option<Vec<Header>>,
    pub template: Option<Template>,
}

/// Context holds the build variables Netlify uses to build a site before deploying it.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Context {
    pub base: Option<String>,
    pub publish: Option<String>,
    pub command: Option<String>,
    pub functions: Option<String>,
    pub environment: Option<HashMap<String, String>>,
}

/// Redirect holds information about a url redirect.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
    #[serde(alias = "params")]
    #[serde(alias = "parameters")]
    pub query: Option<HashMap<String, String>>,
    pub conditions: Option<HashMap<String, HashSet<String>>>,
    pub signed: Option<String>,
    #[serde(alias = "edge-handler")]
    pub edge_handler: Option<String>,
}

/// Header holds information to add response headers for a give url.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Header {
    #[serde(rename = "for")]
    pub path: String,
    #[serde(rename = "values")]
    pub headers: HashMap<String, HeaderValues>,
}

#[derive(Debug, PartialEq, Default)]
pub struct HeaderValues {
    pub values: Vec<String>,
}

/// Template holds information to turn a repository into a Netlify template.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Template {
    #[serde(rename = "incoming-hooks")]
    pub hooks: Option<Vec<String>>,
    pub environment: Option<HashMap<String, String>>,
}

/// Base crate error type
pub type Error = toml::de::Error;

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
pub fn from_str(io: &str) -> Result<Config, Error> {
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
