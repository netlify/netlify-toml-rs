extern crate netlify_toml;

#[test]
fn test_it_parses_complete_example() {
    let io = r#"
[build]
command = "make site"
edge_handlers = "src/custom-edge-handlers"

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
    assert_eq!(
        build.edge_handlers.unwrap().as_str(),
        "src/custom-edge-handlers"
    );

    let context = config.context.unwrap();
    let ref prod = context.get("production").unwrap();
    if let Some(ref cmd) = prod.command {
        assert_eq!(cmd, &String::from("make prod"));
    }

    let headers = config.headers.unwrap();
    let header = &headers[0];
    assert_eq!("/foo", header.path);
    assert_eq!(1, header.headers["X-Foo"].values.len())
}

#[test]
fn test_it_loads_context_env() {
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

#[test]
fn test_it_fails_to_parse_invalid_headers() {
    let io = r#"
[[headers]]
for = "/foo"
[[headers.values]]
    X-Foo = "Bar"
    "#;

    let result = netlify_toml::from_str(&io);
    match result {
        Ok(v) => assert!(false, "unexpected config: {:?}", v),
        Err(e) => println!("error parsing headers: {:?}", e),
    }
}

#[test]
fn test_it_loads_headers_as_array() {
    let io = r#"
[[headers]]
for = "/foo"
[headers.values]
    X-Foo = [
        "Bar",
        "Baz",
        "Qux"
    ]
    "#;

    let config = netlify_toml::from_str(&io).unwrap();
    let mut headers = config.headers.unwrap();

    let header = headers.pop().unwrap();
    assert_eq!("/foo", header.path);
    assert_eq!(3, header.headers["X-Foo"].values.len())
}

#[test]
fn test_it_splits_strings_as_array() {
    let io = r#"
[[headers]]
for = "/foo"
[headers.values]
    Link = """
        </style1.css>; rel=preload; as=style,\
        </style2.css>; rel=preload; as=style,\
        </style3.css>; rel=preload; as=style"""
    "#;

    let config = netlify_toml::from_str(&io).unwrap();
    let mut headers = config.headers.unwrap();

    let header = headers.pop().unwrap();
    assert_eq!("/foo", header.path);
    assert_eq!(3, header.headers["Link"].values.len())
}

#[test]
fn test_full_redirect_rules() {
    let io = r#"
[[redirects]]
  from = "/old-path"
  to = "/new-path"
  status = 302
  force = true
  query = {path = ":path"}
  conditions = {Language = ["en"], Country = ["US"], Role = ["admin"]}
  headers = {X-From = "Netlify"}
  signed = "API_SIGNATURE_TOKEN"
  edge-handler = "hello-world"
    "#;

    let config = netlify_toml::from_str(&io).unwrap();
    let mut redirects = config.redirects.unwrap();
    assert_eq!(1, redirects.len());

    let redirect = redirects.pop().unwrap();
    assert_eq!("/old-path", redirect.from);
    assert_eq!(Some("/new-path".to_string()), redirect.to);
    assert_eq!("API_SIGNATURE_TOKEN", redirect.signed.unwrap());
    assert_eq!(302, redirect.status);
    assert_eq!(true, redirect.force);
    assert_eq!("hello-world", redirect.edge_handler.unwrap());

    let query = redirect.query.unwrap();
    assert_eq!(1, query.len());
    assert_eq!(":path", query.get("path").unwrap());

    let conditions = redirect.conditions.unwrap();
    assert_eq!(3, conditions.len());

    let headers = redirect.headers.unwrap();
    assert_eq!(1, headers.len());
    assert_eq!("Netlify", headers.get("X-From").unwrap());
}

#[test]
fn test_redirect_rule_with_defaults() {
    let io = r#"
[[redirects]]
  from = "/old-path"
  to = "/new-path"
    "#;

    let config = netlify_toml::from_str(&io).unwrap();
    let mut redirects = config.redirects.unwrap();
    assert_eq!(1, redirects.len());

    let redirect = redirects.pop().unwrap();
    assert_eq!("/old-path", redirect.from);
    assert_eq!(Some("/new-path".to_string()), redirect.to);
    assert_eq!(301, redirect.status);
    assert_eq!(false, redirect.force);
}

#[test]
fn test_unique_redirect_conditions() {
    let io = r#"
[[redirects]]
  from = "/old-path"
  to = "/new-path"
  status = 302
  conditions = {Language = ["en", "es", "en"]}
    "#;

    let config = netlify_toml::from_str(&io).unwrap();
    let mut redirects = config.redirects.unwrap();
    assert_eq!(1, redirects.len());

    let redirect = redirects.pop().unwrap();

    let conditions = redirect.conditions.unwrap();
    assert_eq!(1, conditions.len());

    let lang = conditions.get("Language").unwrap();
    assert_eq!(2, lang.len());
    assert!(lang.contains("en"));
    assert!(lang.contains("es"));
}

#[test]
fn parses_aliased_edge_handlers_name() {
    let io = r#"
[build]
edge-handlers = "src/custom-edge-handlers"
    "#;

    let config = netlify_toml::from_str(&io).unwrap();
    assert_eq!(
        config.build.unwrap().edge_handlers.unwrap(),
        "src/custom-edge-handlers"
    );
}
