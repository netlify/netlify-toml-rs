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

    let headers = config.headers.unwrap();
    let header = &headers[0];
    assert_eq!("/foo", header.path);
    assert_eq!(1, header.headers["X-Foo"].values.len())
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

#[test]
fn it_fails_to_parse_invalid_headers() {
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
fn it_loads_headers_as_array() {
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
fn it_splits_strings_as_array() {
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
fn full_redirect_rules() {
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
    "#;

    let config = netlify_toml::from_str(&io).unwrap();
    let mut redirects = config.redirects.unwrap();
    assert_eq!(1, redirects.len());

    let redirect = redirects.pop().unwrap();
    assert_eq!("/old-path", redirect.from);
    assert_eq!("/new-path", redirect.to);
    assert_eq!("API_SIGNATURE_TOKEN", redirect.signed.unwrap());
    assert_eq!(302, redirect.status.unwrap());
    assert_eq!(true, redirect.force.unwrap());

    let query = redirect.query.unwrap();
    assert_eq!(1, query.len());
    assert_eq!(":path", query.get("path").unwrap());

    let conditions = redirect.conditions.unwrap();
    assert_eq!(3, conditions.len());

    let headers = redirect.headers.unwrap();
    assert_eq!(1, headers.len());
    assert_eq!("Netlify", headers.get("X-From").unwrap());
}