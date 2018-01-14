extern crate netlify_toml;

#[test]
fn it_parses_complete_example() {
    let io = r#"
[build]
command = "make site"

[context.production] # this is an alias for build
command = "make prod"

[context.deploy-preview]
command = "make dp"

[context.branch-deploy]
command = "make bd"

[[redirects]]
from = "/foo"
to = "/bar"

[[headers]]
for = "/foo"
[headers.values]
    X-Foo = "Bar"

[template]
    incoming-hooks = ["foo", "bar"]
    environment = {FOO = "BAR", BAZ = "QUX"}
    "#;

    let config = netlify_toml::from_str(&io).unwrap();
    let build = config.build.unwrap();
    assert_eq!(build.command.unwrap(), String::from("make site"));

    let context = config.context.unwrap();
    let ref prod = context.get("production").unwrap();
    if let Some(ref cmd) = prod.command {
       assert_eq!(cmd, &String::from("make prod"));
    }
}

#[test]
fn it_loads_context_env() {
    let io = r#"
[build]
environment = {BUILD = "true", OVERRIDE = "1"}

[context.deploy-preview]
environment = {DP = "true", OVERRIDE = "2"}

[context.branch]
environment = {BRANCH = "true"}
    "#;

    let config = netlify_toml::from_str(&io).unwrap();
    let env = config.context_env("deploy-preview", "branch");
    assert!(env.contains_key("BUILD"));
    assert!(env.contains_key("DP"));
    assert!(env.contains_key("BRANCH"));

    let e = String::from("2");
    assert_eq!(env.get("OVERRIDE"), Some(&e));
}
